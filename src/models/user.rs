use crate::models::auth::UserRole;

#[derive(sqlx::FromRow, serde::Deserialize, serde::Serialize)]
pub struct User {
    pub id: String,
    pub username: String,
    pub email: String,
    pub role: UserRole,
}

impl From<UserWithPassword> for User {
    fn from(value: UserWithPassword) -> Self {
        Self {
            id: value.id,
            username: value.username,
            email: value.email,
            role: value.role,
        }
    }
}

#[derive(sqlx::FromRow, serde::Deserialize, serde::Serialize)]
pub struct UserWithPassword {
    pub id: String,
    pub username: String,
    pub password: String,
    pub email: String,
    pub role: UserRole,
}
