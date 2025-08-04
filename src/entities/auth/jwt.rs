use crate::{
	entities::auth::{
			constants::{ACCESS_TOKEN_EXPIRATION, REFRESH_TOKEN_EXPIRATION},
			dto::{Tokens},
	},
	common::{AppState, errors::api_error::ApiError},
	models::{
			auth::{Claims, UserRole},
	},
};
use actix_web::{
	HttpRequest, HttpResponse,
	cookie::{Cookie, SameSite, time},
	web,
};
use jsonwebtoken::{DecodingKey, EncodingKey, Header, Validation, decode, encode};

pub fn create_jwt(
	user_id: &str,
	is_premium: &bool,
	role: &UserRole,
	expires_after: i64,
) -> Result<String, String> {
	use chrono::Utc;

	let secret = std::env::var("JWT_SECRET").unwrap_or("roscript-backend".to_string());

	let claims = Claims {
			sub: user_id.to_owned(),
			role: *role,
			is_premium: *is_premium,
			exp: (Utc::now().timestamp() + expires_after) as usize, // two weeks
	};

	match encode(
			&Header::default(),
			&claims,
			&EncodingKey::from_secret(secret.as_bytes()),
	) {
			Ok(token) => Ok(token),
			Err(e) => Err(format!("Ошибка при попытке создания токена, {:?}", e)),
	}
}

pub fn verify_jwt(token: &str) -> Option<Claims> {
	let secret = std::env::var("JWT_SECRET").unwrap_or("roscript-backend".to_string());

	let validation = Validation::default();

	decode::<Claims>(
			token,
			&DecodingKey::from_secret(secret.as_bytes()),
			&validation,
	)
	.map(|data| data.claims)
	.ok()
}

pub fn generate_tokens(
	user_id: &str,
	is_premium: &bool,
	role: &UserRole,
) -> Result<Tokens, ApiError> {
	let access_token =
			create_jwt(user_id, is_premium, role, ACCESS_TOKEN_EXPIRATION).map_err(|e| {
					ApiError::Other(format!(
							"Error when trying to generate access token {:?}",
							e
					))
			})?;
	let refresh_token =
			create_jwt(user_id, is_premium, role, REFRESH_TOKEN_EXPIRATION).map_err(|e| {
					ApiError::Other(format!(
							"Error when trying to generate refresh token {:?}",
							e
					))
			})?;

	Ok(Tokens {
			access_token,
			refresh_token,
	})
}

pub fn refresh_token(
	req: HttpRequest,
	app_state: web::Data<AppState>,
) -> Result<HttpResponse, ApiError> {
	// get refresh token cookie value from request
	let cookie = req
			.cookie("refresh_token")
			.ok_or_else(|| ApiError::Unauthorized("Refresh token cookie not found".into()))?;
	let refresh_token = cookie.value();

	// getting claims if refresh token is valid
	let claims = verify_jwt(refresh_token)
			.ok_or_else(|| ApiError::Unauthorized("Invalid Refresh Token".into()))?;

	// if no drop (no api errors above)
	// then generate new tokens
	let new_tokens = generate_tokens(&claims.sub, &claims.is_premium, &claims.role)?;

	let new_refresh_cookie = Cookie::build("refresh_token", new_tokens.refresh_token)
			.http_only(true)
			.secure(app_state.is_production)
			.path("/")
			.same_site(SameSite::Strict)
			.max_age(time::Duration::days(30))
			.finish();

	Ok(HttpResponse::Ok()
			.cookie(new_refresh_cookie)
			.json(serde_json::json!({"access_token": new_tokens.access_token})))
}
