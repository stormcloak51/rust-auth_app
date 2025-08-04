use actix_web::{HttpResponse, ResponseError, http::StatusCode};
use sqlx::postgres::PgDatabaseError;
use std::{collections::HashMap, fmt};
use validator::ValidationErrors;

#[derive(Debug, serde::Serialize)]
pub struct ErrorResponseBody {
    code: u16,
    error: String,
    message: String,
    details: Option<HashMap<String, Vec<String>>>,
}

#[derive(Debug)]
pub enum ApiError {
    UniqueViolation { field: String },
    NotFound(String),
    Unauthorized(String),
    Validation(ValidationErrors), // errors
    InternalServer(String),
    Other(String),
}

fn extract_field_from_constraint(constraint: &str) -> String {
    use regex::Regex;
    let re = Regex::new(r".*_(.+)_key$").unwrap();
    if let Some(captures) = re.captures(constraint) {
        return captures[1].to_string();
    }

    return "field".to_string();
}

impl fmt::Display for ApiError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            // CUSTOM ERRORS
            ApiError::UniqueViolation { field } => {
                write!(f, "Entity with such {} already exists", field)
            }
            ApiError::NotFound(msg) => write!(f, "{}", msg),
            ApiError::Unauthorized(msg) => write!(f, "{}", msg),
            ApiError::Validation(_) => write!(f, "Validation Error"),

            // DATABASE / BACKEND ERRORS
            ApiError::InternalServer(e) => write!(f, "Database error: {e}"),
            ApiError::Other(msg) => write!(f, "{}", msg),
        }
    }
}

impl ResponseError for ApiError {
    fn status_code(&self) -> StatusCode {
        match self {
            ApiError::UniqueViolation { .. } => StatusCode::CONFLICT,
            ApiError::NotFound(_) => StatusCode::NOT_FOUND,
            ApiError::Unauthorized(_) => StatusCode::UNAUTHORIZED,
            ApiError::Validation(..) => StatusCode::BAD_REQUEST,

            ApiError::InternalServer(_) => StatusCode::INTERNAL_SERVER_ERROR,
            ApiError::Other(_) => StatusCode::BAD_REQUEST,
        }
    }

    fn error_response(&self) -> HttpResponse {
        let status = self.status_code();
        let details = match self {
            ApiError::Validation(errors) => {
                let mut validation_details: HashMap<String, Vec<String>> = HashMap::new();

                for (field, field_errors) in errors.field_errors() {
                    let messages: Vec<String> = field_errors
                        .into_iter()
                        .map(|e| {
                            e.message
                                .as_ref()
                                .map(|s| s.to_string())
                                .unwrap_or_else(|| e.code.to_string())
                        })
                        .collect();
                    validation_details.insert(field.to_string(), messages);
                }

                Some(validation_details)
            }
            _ => None,
        };

        let body = ErrorResponseBody {
            code: status.as_u16(),
            error: status.canonical_reason().unwrap_or("Error").to_string(),
            message: self.to_string(),
            details,
        };

        HttpResponse::build(status).json(body)
    }
}

// database errors
impl From<sqlx::Error> for ApiError {
    fn from(e: sqlx::Error) -> Self {
        let sqlx::Error::Database(db_err) = &e else {
            return ApiError::InternalServer(e.to_string());
        };

        let pg_err = db_err.downcast_ref::<PgDatabaseError>();
        let code = pg_err.code();

        // check for unique violation
        if code == "23505" {
            if let Some(constraint) = pg_err.constraint() {
                let field = extract_field_from_constraint(constraint);
                return ApiError::UniqueViolation { field };
            }
        }

        ApiError::InternalServer(e.to_string())
    }
}
