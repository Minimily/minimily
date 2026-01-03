use actix_files as fs;
use actix_session::SessionMiddleware;
use actix_session::storage::CookieSessionStore;
use actix_web::{web, App, HttpServer};
use actix_web::cookie::Key;
use actix_web::middleware::Logger;
use crate::config::Config;
use crate::model::AppState;
use crate::view;

pub struct Server {
    port: u16,
    session_secret_key: Key,
    session_secure: bool,
}

impl Server {
    pub fn new(config: Config) -> Self {
        let session_key = config.session.secrete_key.as_bytes();

        let secret_key = if session_key.len() < 64 {
            log::warn!("SESSION_SECRET_KEY is too short (min 64 bytes). Generating a temporary random key.");
            Key::generate()
        } else {
            Key::from(session_key)
        };

        Server {
            port: config.server_port,
            session_secret_key: secret_key,
            session_secure: config.session.secure,
        }
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
        let secret_key = self.session_secret_key.clone();
        let secure = self.session_secure.clone();

        /*
        * actix-web will spin up a worker process for each available core. Each
        * worker runs its own copy of the application.
        */
        HttpServer::new(move || {
            let worker_key = secret_key.clone();
            let worker_secure = secure.clone();

            App::new()
                .wrap(Logger::default())
                .wrap(
                    SessionMiddleware::builder(CookieSessionStore::default(), worker_key)
                        .cookie_name("minimily-session".to_string())
                        .cookie_secure(worker_secure)
                        .cookie_http_only(true)
                        .build()
                )
                .service(fs::Files::new("/assets", "./content/static/assets").show_files_listing())
                .route("/", web::get().to(view::home))
                .route("/signin", web::get().to(view::sign_in))
                .route("/signin", web::post().to(view::sign_in_post))
                .route("/signup", web::get().to(view::sign_up))
                .route("/signup", web::post().to(view::sign_up_post))
                .route("/robots.txt", web::get().to(view::robots))
                .route("/sitemap.xml", web::get().to(view::sitemap))
                .route("/health", web::get().to(view::health_check))
                .default_service(web::route().to(view::not_found))
                .app_data(app_state.clone())
        }).bind(("0.0.0.0", self.port))?.run().await
    }
}