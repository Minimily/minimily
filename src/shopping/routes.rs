use actix_web::web;
use crate::shopping::view;

pub fn configure(cfg: &mut web::ServiceConfig) {
    cfg.service(
        web::scope("/shopping")
            .route("", web::get().to(view::shopping))
            .route("/goods", web::post().to(view::good_create_post))
            .route("/goods/{id}", web::get().to(view::good))
            .route("/goods/{id}/edit", web::post().to(view::good_edit_post))
            .route("/goods/{id}/delete", web::post().to(view::good_delete_post))
            .route("/locations", web::post().to(view::location_create_post))
            .route("/locations/{id}/edit", web::post().to(view::location_edit_post))
            .route("/locations/{id}/delete", web::post().to(view::location_delete_post))
    );
}
