use actix_web::{web, HttpResponse, Responder, Either};
use crate::handler::{handle_signin, handle_signup};
use crate::model::AppState;
use crate::form::{SignInForm, SignUpForm};
use crate::template;
use crate::template::respond_with_template;

pub async fn home(state: web::Data<AppState>) -> impl Responder {
    let context = template::create_context(state.get_ref());
    respond_with_template(state, context, "index.html")
}

pub async fn sign_up(state: web::Data<AppState>) -> impl Responder {
    let context = handle_signup(state.get_ref(), None).await;
    respond_with_template(state, context, "signup.html")
}

pub async fn sign_up_post(state: web::Data<AppState>, form: web::Form<SignUpForm>) -> impl Responder {
    let context = handle_signup(state.get_ref(), Some(form.into_inner())).await;
    if let Some(next_value) = context.get("next") {
        if let Some(next) = next_value.as_str() {
            return respond_with_template(state, context.clone(), next)
        }
    }

    respond_with_template(state, context, "signup.html")
}

pub async fn sign_in(state: web::Data<AppState>) -> impl Responder {
    let context = handle_signin(state.get_ref(), None).await;
    respond_with_template(state, context, "signin.html")
}

pub async fn sign_in_post(state: web::Data<AppState>, form: web::Form<SignInForm>) -> impl Responder {
    let context = handle_signin(state.get_ref(), Some(form.into_inner())).await;
    if let Some(redirect_val) = context.get("redirect") {
        if let Some(redirect) = redirect_val.as_str() {
            return Either::Left(web::Redirect::to(redirect.to_string()).see_other());
        }
    }
    Either::Right(respond_with_template(state, context.clone(), "signin.html"))
}

pub async fn robots(state: web::Data<AppState>) -> HttpResponse {
    let context = template::create_context(state.get_ref());
    respond_with_template(state, context, "robots.html")
}

pub async fn sitemap(state: web::Data<AppState>) -> HttpResponse {
    let context = template::create_context(state.get_ref());
    respond_with_template(state, context, "sitemap.html")
}

pub async fn not_found(state: web::Data<AppState>) -> HttpResponse {
    let context = template::create_context(state.get_ref());
    respond_with_template(state, context, "404.html")
}

pub async fn health_check() -> HttpResponse {
    HttpResponse::Ok().into()
}