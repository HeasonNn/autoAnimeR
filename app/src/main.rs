use routers::*;
use actix_web::{App, HttpServer, web};
use diesel::r2d2::{self, ConnectionManager};
use diesel::SqliteConnection;
use tera::Tera;
use actix_files::Files;

mod routers;
mod schema;
mod api;
mod models;
mod dao;
mod mods;

#[macro_use]
extern crate diesel;

pub type Pool = r2d2::Pool<ConnectionManager<SqliteConnection>>;

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    dotenv::dotenv().ok();
    let database_url = std::env::var("DATABASE_URL")
        .expect("DATABASE_URL must be set");
    let database_pool = Pool::builder()
        .build(ConnectionManager::<SqliteConnection>::new(database_url))
        .expect("Failed to create pool.");

    let tera = Tera::new("templates/**/*").expect("Failed to load templates");

    HttpServer::new(move || {
        App::new()
            .app_data(web::Data::new(database_pool.clone()))
            .app_data(web::Data::new(tera.clone()))
            .service(Files::new("/static", "./static").show_files_listing())
            .configure(anime_routes)
    })
    .bind(("0.0.0.0", 8080))?
    .run()
    .await
}