
use serde::{Deserialize, Serialize};
use sqlx::Type;

#[derive(Debug, Serialize, Deserialize, Clone, Copy, PartialEq, Eq, Type)]
#[sqlx(type_name = "user_role", rename_all = "UPPERCASE")]
pub enum UserRole {
	// GUEST, // guest is unavailable on database
	User,
	Admin
}

impl UserRole {
	pub fn is_admin(&self) -> bool {
		matches!(self, UserRole::Admin)
	}

	pub fn is_user(&self) -> bool {
		matches!(self, UserRole::User)
	}
}

#[derive(Serialize, Deserialize)]
pub struct Claims {
	pub sub: String,
	pub role: UserRole,
	pub is_premium: bool,
	pub exp: usize,
}