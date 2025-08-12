use actix_web::{web, get, post, put,  error};
use actix_web::web::{Json, Path};
use crate::models::property::{Property, PropertyNew};
use crate::repository::property_repo::PropertyRepo;

#[derive(Clone)]
pub struct AppState {
    pub repository: PropertyRepo,
}

#[get("")]
pub async fn get_all_properties(state: web::Data<AppState>) -> actix_web::Result<Json<Vec<Property>>> {
    log::info!("Requesting all properties");
    
    match state.repository.find_all_properties().await {
        Ok(properties) => Ok(Json(properties)),
        _ => Ok(Json(vec![]))
    }
}

#[get("/{id}")]
pub async fn get_property_by_id(path: Path<String>, state: web::Data<AppState>) -> actix_web::Result<Json<Property>> {
    let id = path.into_inner();

    log::info!("Requesting Property with ID: {id}");

    match state.repository.find_property_by_id(id.clone()).await {
        Ok(property) => Ok(Json(property)),
        Err(sqlx::Error::RowNotFound) => {
            Err(error::ErrorNotFound(format!("Property with id {} not found", id)))
        }
        Err(e) => {
            // This will show in Shuttle's logs
            log::error!("Database error: {}", e);
            Err(error::ErrorInternalServerError("Database error"))
        }
    }
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

#[put("/{id}")]
pub async fn put_property(path: Path<String>, property: Json<PropertyNew>, state: web::Data<AppState>) -> actix_web::Result<Json<Property>> {
    let id = path.into_inner();
    // let oldProperty = state.repository.find_property_by_id(id.clone()).await.unwrap();

    log::info!("Updating Property with ID: {id}");

    match state.repository.update_property_by_id(id.clone(), property).await {
        Ok(property) => Ok(Json(property)),
        Err(sqlx::Error::RowNotFound) => {
            Err(error::ErrorNotFound(format!("Property with id {} not found", id)))
        }
        Err(e) => {
            // This will show in Shuttle's logs
            log::error!("Database error: {}", e);
            Err(error::ErrorInternalServerError("Database error"))
        }
    }

}