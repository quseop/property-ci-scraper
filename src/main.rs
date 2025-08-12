mod repository;
mod models;
mod controller;

use actix_web::{
    get, middleware::Logger, post,
    web::{self, Json, ServiceConfig},
    Result,
};

use shuttle_actix_web::ShuttleActixWeb;
use sqlx::{ PgPool};
use crate::controller::controller::{get_all_properties, get_property_by_id, post_property, put_property, AppState};
use crate::repository::property_repo::PropertyRepo;

#[shuttle_runtime::main]
async fn main(
    #[shuttle_shared_db::Postgres] pool: PgPool,
) -> ShuttleActixWeb<impl FnOnce(&mut ServiceConfig) + Send + Clone + 'static> {
    sqlx::migrate!()
        .run(&pool)
        .await
        .expect("Failed to run migrations");


    let state = web::Data::new(AppState { repository: PropertyRepo::new(pool) });

    let config = move |cfg: &mut ServiceConfig| {
        cfg.service(
            web::scope("/properties")
                .wrap(Logger::default())
                .service(get_all_properties)
                .service(get_property_by_id)
                .service(post_property)
                .service(put_property)
                .app_data(state),
        );
    };

    Ok(config.into())
}
