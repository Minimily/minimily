use actix_files as fs;
use actix_web::{web, App, HttpResponse, HttpServer, Responder};
use actix_web::middleware::Logger;
use sqlx::PgPool;

pub struct AppState {
    pub pool: PgPool,
}

impl AppState {
    pub fn new(pool: PgPool) -> Self {
        AppState { pool }
    }
}

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
                .route("/", web::get().to(home))
                .default_service(web::route().to(http_404))
                .app_data(app_state.clone())
        })
            .bind(("0.0.0.0", self.port))?
            .run()
            .await
    }
}

async fn home() -> impl Responder {
    HttpResponse::Ok().body("Minimily")
}

async fn http_404() -> HttpResponse {
    HttpResponse::NotFound()
        .content_type("text/html; charset=utf-8")
        .body("Page not found!")
}