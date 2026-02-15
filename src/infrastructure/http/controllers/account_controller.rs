use actix_web::{get, HttpResponse, Responder};

#[get("/accounts/{id}")]
async fn get_account() -> impl Responder {
    HttpResponse::Ok().json("Hello world !")
}