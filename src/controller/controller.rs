use actix_web::{web, get, post, HttpResponse, Responder, error};
use actix_web::web::{Json, Path};
use crate::models::property::{Property, PropertyNew};
use crate::repository::property_repo::PropertyRepo;

#[derive(Clone)]
pub struct AppState {
    pub repository: PropertyRepo,
}

#[get("/{id}")]
pub async fn get_property_by_id(path: Path<String>, state: web::Data<AppState>) -> impl Responder {
    let id = path.to_string();
    HttpResponse::Ok().body("Here's the property")
}

#[post("")]
pub async fn post_property(property: Json<PropertyNew>, state: web::Data<AppState>) -> actix_web::Result<Json<Property>> {
    let property = state
        .repository
        .create_property(property)
        .await
        .map_err(|e| error::ErrorBadRequest(e.to_string()))?;

    Ok(Json(property))
}