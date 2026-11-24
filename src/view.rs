use actix_session::Session;
use actix_web::{web, HttpResponse, Responder};
use crate::state::AppState;
use crate::template;
use crate::template::respond_with_template;

pub async fn home(state: web::Data<AppState>, session: Session) -> impl Responder {
    let context = template::create_context(&session);
    respond_with_template(state, context, "index.html")
}

pub async fn robots(state: web::Data<AppState>, session: Session) -> HttpResponse {
    let context = template::create_context(&session);
    respond_with_template(state, context, "robots.html")
}

pub async fn not_found(state: web::Data<AppState>, session: Session) -> HttpResponse {
    let context = template::create_context(&session);
    respond_with_template(state, context, "404.html")
}

pub async fn health_check() -> HttpResponse {
    HttpResponse::Ok().into()
}