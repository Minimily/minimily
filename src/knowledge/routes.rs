use crate::knowledge::view;
use actix_web::web;

pub fn configure(cfg: &mut web::ServiceConfig) {
    cfg.service(web::scope("/knowledge").route("", web::get().to(view::knowledge)));
}
