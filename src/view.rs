use actix_web::{web, HttpResponse, Responder};
use crate::model::AppState;
use crate::template;

pub async fn home(state: web::Data<AppState>) -> impl Responder {
    let context = template::create_context(state.get_ref());

    HttpResponse::Ok()
        .content_type("text/html; charset=utf-8")
        .body(
            state.get_ref().template.render("index.html", &context).unwrap()
        )
}

pub async fn robots(state: web::Data<AppState>) -> HttpResponse {
    let context = template::create_context(state.get_ref());

    HttpResponse::Ok()
        .content_type("text/plain; charset=utf-8")
        .body(
            state.get_ref().template.render("robots.html", &context).unwrap()
        )
}

pub async fn sitemap(state: web::Data<AppState>) -> HttpResponse {
    let context = template::create_context(state.get_ref());

    HttpResponse::Ok()
        .content_type("text/xml; charset=utf-8")
        .body(
            state.get_ref().template.render("sitemap.html", &context).unwrap()
        )
}

pub async fn not_found(state: web::Data<AppState>) -> HttpResponse {
    let context = template::create_context(state.get_ref());

    HttpResponse::NotFound()
        .content_type("text/html; charset=utf-8")
        .body(
            state.get_ref().template.render("404.html", &context).unwrap()
        )
}

pub async fn health_check() -> HttpResponse {
    HttpResponse::Ok().into()
}