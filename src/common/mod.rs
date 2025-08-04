

use sqlx::PgPool;

pub mod database;
pub mod errors;

pub struct AppState {
	pub pool: PgPool,
	pub is_production: bool,
}