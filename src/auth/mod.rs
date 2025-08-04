pub mod constants;
pub mod dto;
pub mod jwt;
pub mod middlewares;

use crate::{
    auth::dto::{AuthResponse, CreateUserDto, LoginDto},
    common::{AppState, errors::api_error::ApiError},
    models::user::User,
    user::{check_user_exists, dto::CheckUserExistsDto},
};
use actix_web::{
    HttpResponse,
    cookie::{Cookie, SameSite, time},
    web,
};
use bcrypt::{DEFAULT_COST, hash, verify};
use uuid::Uuid;
use validator::Validate;

pub async fn register(
    new_user: web::Json<CreateUserDto>,
    app_state: web::Data<AppState>,
) -> Result<HttpResponse, ApiError> {
    match new_user.validate() {
        Ok(_) => (),
        Err(e) => {
            println!("validata {:#?}", e.field_errors().);
            Err(ApiError::Validation(e))
        }?,
    }

    // hash password
    let password_hash = match hash(&new_user.password, DEFAULT_COST) {
        Ok(h) => h,
        Err(_) => {
            return Err(ApiError::InternalServer(
                "Something went wrong on server side".to_string(),
            ));
        }
    };

    // generating uuid for new user
    let new_user_id = Uuid::new_v4();

    let query = "
			INSERT INTO users (username, email, password, id)
			VALUES ($1, $2, $3, $4)
			RETURNING id, username, email, role, created_at
		";

    // creating new user and after that fetching it
    let result = sqlx::query_as::<_, User>(query)
        .bind(&new_user.username)
        .bind(&new_user.email)
        .bind(&password_hash)
        .bind(new_user_id.to_string())
        .fetch_one(&app_state.pool)
        .await?;

    // generating access and refresh tokens
    let tokens = jwt::generate_tokens(&result.id, &false, &result.role)?;

    // build cookie for refresh token that stands by httpOnly parameter
    let refresh_cookie = Cookie::build("refresh_token", tokens.refresh_token)
        .http_only(true)
        .secure(app_state.is_production)
        .path("/")
        .same_site(SameSite::Strict)
        .max_age(time::Duration::days(30))
        .finish();

    Ok(HttpResponse::Ok()
        .cookie(refresh_cookie)
        .json(AuthResponse {
            user: result,
            access_token: tokens.access_token,
        }))
}

pub async fn login(
    dto: web::Json<LoginDto>,
    app_state: web::Data<AppState>,
) -> Result<HttpResponse, ApiError> {
    // is user existing
    let user = check_user_exists(
        CheckUserExistsDto {
            username_or_email: dto.username_or_email.clone(),
        },
        &app_state.pool,
    )
    .await?;

    // is password valid
    let is_password_valid = verify(&dto.password, &user.password)
        .map_err(|_| ApiError::Other("Error when tried to compare passwords".into()))?;
    if !is_password_valid {
        return Err(ApiError::Other("Incorrect Password".into()));
    }

    // generating access and refresh tokens
    let tokens = jwt::generate_tokens(&user.id, &false, &user.role)?;

    let refresh_cookie = Cookie::build("refresh_token", tokens.refresh_token)
        .http_only(true)
        .secure(app_state.is_production)
        .path("/")
        .same_site(SameSite::Strict)
        .max_age(time::Duration::days(30))
        .finish();

    Ok(HttpResponse::Ok()
        .cookie(refresh_cookie)
        .json(AuthResponse {
            user: User::from(user),
            access_token: tokens.access_token,
        }))
}
