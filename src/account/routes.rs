use actix_web::web;
use crate::account::view;

pub fn configure(cfg: &mut web::ServiceConfig) {
    cfg.route("/signup", web::get().to(view::sign_up))
        .route("/signup", web::post().to(view::sign_up_post))
        .route("/signin", web::get().to(view::sign_in))
        .route("/signin", web::post().to(view::sign_in_post))
        .route("/signout", web::post().to(view::sign_out))
        .route("/profile", web::get().to(view::profile))
        .route("/profile/edit", web::get().to(view::profile_edit))
        .route("/profile/edit", web::post().to(view::profile_edit_post))
        .route("/profile/family", web::post().to(view::family_create_post));
}
