use actix_files as fs;
use actix_web::{web, App, HttpServer};
use actix_web::middleware::Logger;
use crate::model::AppState;
use crate::view;

pub struct Server {
    pub port: u16,
}

impl Server {
    pub fn new(port: u16) -> Self {
        Server { port }
    }

    pub async fn run(&self, state: AppState) -> std::io::Result<()> {
        let app_state = web::Data::new(state);

        HttpServer::new(move || {
            App::new()
                .wrap(Logger::default())
                .service(fs::Files::new("/assets", "./content/static/assets").show_files_listing())
                .route("/", web::get().to(view::home))
                .route("/robots.txt", web::get().to(view::robots))
                .route("/sitemap.xml", web::get().to(view::sitemap))
                .route("/health", web::get().to(view::health_check))
                .default_service(web::route().to(view::not_found))
                .app_data(app_state.clone())
        }).bind(("0.0.0.0", self.port))?.run().await
    }
}