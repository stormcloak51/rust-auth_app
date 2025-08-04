pub mod dto;

use dto::CheckUserExistsDto;
use sqlx::PgPool;

use crate::{common::errors::api_error::ApiError, models::user::UserWithPassword};

pub async fn check_user_exists(
    dto: CheckUserExistsDto,
    pool: &PgPool,
) -> Result<UserWithPassword, ApiError> {
    let query = r#"
		SELECT id, username, email, password, role, created_at
		FROM users 
		WHERE username = $1 OR email = $1
		LIMIT 1
	"#;

    match sqlx::query_as::<_, UserWithPassword>(query)
        .bind(&dto.username_or_email)
        .fetch_one(pool)
        .await
    {
        Ok(user) => Ok(user),
        Err(sqlx::Error::RowNotFound) => Err(ApiError::NotFound(format!(
            "User {} not found",
            dto.username_or_email
        ))),
        Err(e) => Err(ApiError::InternalServer(e.to_string())),
    }
}
