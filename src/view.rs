use actix_web::{web, HttpResponse, Responder};
use crate::handler::handle_signup;
use crate::model::AppState;
use crate::form::UserAccountForm;
use crate::template;
use crate::template::respond_with_template;

pub async fn home(state: web::Data<AppState>) -> impl Responder {
    let context = template::create_context(state.get_ref());

    HttpResponse::Ok()
        .content_type("text/html; charset=utf-8")
        .body(
            state.get_ref().template.render("index.html", &context).unwrap()
        )
}

pub async fn signup(state: web::Data<AppState>) -> impl Responder {
    let context = handle_signup(state.get_ref(), None).await;
    respond_with_template(state, context, "signup.html")
}

pub async fn signup_post(state: web::Data<AppState>, form: web::Form<UserAccountForm>) -> impl Responder {
    let context = handle_signup(state.get_ref(), Some(form.into_inner())).await;
    if context.get("next").is_some() && context.get("next").unwrap() == "signup_ok.html" {
        return respond_with_template(state, context, "signup_ok.html")
    }
    
    respond_with_template(state, context, "signup.html")
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