use actix_web::{
    App, HttpResponse, HttpServer,
    web::{self},
};
use dotenv::dotenv;
use rust_backend::{
    common::{
        AppState,
        database::{create_db_pool, run_migrations},
    },
    entities::{
        auth::{guards::role_guard::RoleGuard, login, middlewares::jwt_auth::JwtAuth, register},
        post::{get_book, get_secret_book},
    },
    models::auth::UserRole,
};
use std::io::Result as IoResult;

// #[derive(Debug, serde::Serialize, serde::Deserialize, Clone)]
// struct Book {
//     id: u32,
//     title: String,
//     author: String,
// }

// async fn create_book(book: web::Json<Book>, data: web::Data<AppState>) -> impl Responder {
//     let mut books = data.books.lock().unwrap();
//     let book_title = book.title.clone();
//     books.insert(book.id, book.into_inner());

//     HttpResponse::Ok().json(format!("Succesfully created a book with title {}", book_title))
// }

// async fn edit_book(path: web::Path<u32>, book: web::Json<Book>, data: web::Data<AppState>) -> impl Responder {
//     let book_id = path.into_inner();
//     let mut books = data.books.lock().unwrap();

//     if books.contains_key(&book_id) {
//         books.insert(book_id, book.0);
//         return HttpResponse::Ok().json("Success!");
//     }

//     HttpResponse::NotFound().body("Not found book")
// }

// async fn delete_book(path: web::Path<u32>, data: web::Data<AppState>) -> impl Responder {
//     let book_id = path.into_inner();
//     let mut books = data.books.lock().unwrap();

//     if books.contains_key(&book_id) {
//         books.remove(&book_id);
//         return HttpResponse::Ok().json("Success!");
//     }

//     HttpResponse::NotFound().body("Not found book")
// }

// async fn get_books(data: web::Data<AppState>) -> impl Responder {
//     let books = data.books.lock().unwrap();

//     HttpResponse::Ok().json(&*books)
// }

async fn test_handler() -> HttpResponse {
    HttpResponse::Ok().body("Pong!")
}

// #[get("/")]
// async fn hello_world() -> impl Responder {
//     HttpResponse::Ok().body("Hello fucking world, im starting learning actix")
// }

#[actix_web::main]
async fn main() -> IoResult<()> {
    dotenv().ok();

    // Database Init
    let pg_pool =
        create_db_pool(&std::env::var("DATABASE_URL").expect("Database Url must be set")).await;
    run_migrations(&pg_pool).await;

    // Is production?
    let is_prod = std::env::var("APP_ENV").expect("Database Url must be set") == "production";

    // Setting app data
    let app_data = web::Data::new(AppState {
        pool: pg_pool,
        is_production: is_prod,
    });

    HttpServer::new(move || {
        App::new()
            .app_data(app_data.clone())
            // .service(
            // web::scope("")
            .route("/register", web::post().to(register))
            .route("/login", web::post().to(login))
            // )
            .service(
                web::scope("")
                    .wrap(JwtAuth)
                    .route("/book", web::get().to(get_book))
                    .route(
                        "/book-secret",
                        web::get().to(get_secret_book),
                    ),
            )
    })
    .bind("127.0.0.1:8080")?
    .run()
    .await
}
