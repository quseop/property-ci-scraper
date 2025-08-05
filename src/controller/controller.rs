use actix_web::{web, get, HttpResponse, Responder};
use actix_web::web::Path;
use crate::repository::property_repo::PropertyRepo;

#[derive(Clone)]
pub struct AppState {
    pub repository: PropertyRepo,
}

#[get("/{id}")]
pub async fn get_property_by_id(path: Path<&str>, state: web::Data<AppState>) -> impl Responder {
    let id = *path;
    HttpResponse::Ok().body("Here's the property")
}