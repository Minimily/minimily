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
        /*
        * app_state holds a PgConnection instance that is not cloneable, but
        * actix-web requires it to be cloneable. To solve this problem we use
        * web::Data to wrap the app_state in an Atomic Reference Counted
        * pointer, so each instance of the application gets a pointer, not a
        * raw copy.
        */
        let app_state = web::Data::new(state);

        /*
        * actix-web will spin up a worker process for each available core. Each
        * worker runs its own copy of the application.
        */
        HttpServer::new(move || {
            App::new()
                .wrap(Logger::default())
                .service(fs::Files::new("/assets", "./content/static/assets").show_files_listing())
                .route("/", web::get().to(view::home))
                .route("/signup", web::get().to(view::signup))
                .route("/signup", web::post().to(view::signup_post))
                .route("/robots.txt", web::get().to(view::robots))
                .route("/sitemap.xml", web::get().to(view::sitemap))
                .route("/health", web::get().to(view::health_check))
                .default_service(web::route().to(view::not_found))
                .app_data(app_state.clone())
        }).bind(("0.0.0.0", self.port))?.run().await
    }
}