use crate::models::user::User;
use validator::Validate;

#[derive(serde::Deserialize, Validate)]
pub struct CreateUserDto {
    #[validate(length(min = 3, max = 20, message = "Name length must be more than 3 chars, but less than 20 chars"))]
    pub username: String,

		#[validate(length(min = 6, message = "Password length must be more than 6 chars"))]
    pub password: String,

		#[validate(email(message = "Incorrect email"))]
    pub email: String,
}

#[derive(serde::Deserialize)]
pub struct LoginDto {
    pub username_or_email: String,
    pub password: String,
}

#[derive(serde::Deserialize, serde::Serialize)]
pub struct AuthResponse {
    // то есть и Login и Register  - общий response у них
    pub user: User,
    pub access_token: String,
}

#[derive(serde::Serialize)]
pub struct Tokens {
    pub access_token: String,
    pub refresh_token: String,
}
