use sqlx::{PgPool, migrate};

pub async fn create_db_pool(database_url: &str) -> PgPool {
    PgPool::connect(database_url)
        .await
        .expect(&"Cannot connect to database {database_url}".to_owned())
}

pub async fn run_migrations(pool: &PgPool) {
    println!("[MIGRATIONS]: Running...");
    match migrate!("./migrations")
    .run(pool)
    .await {
        Ok(_) => println!("[MIGRATIONS]: Successfully applied"),
        Err(e) => eprintln!("[MIGRATIONS]: Something went wrong: {}", e)
    }
}