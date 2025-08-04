use actix_web::{HttpResponse, Responder};



pub async fn get_book() -> impl Responder {
	HttpResponse::Ok().body("Success you got it!")
}