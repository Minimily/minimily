use actix_session::Session;
use actix_web::{web, HttpResponse, Responder, Either};
use crate::handler::{handle_sign_in, handle_sign_up};
use crate::model::AppState;
use crate::form::{SignInForm, SignUpForm};
use crate::{repository, template};
use crate::template::respond_with_template;

pub async fn home(state: web::Data<AppState>, session: Session) -> impl Responder {
    let context = template::create_context(&session);
    respond_with_template(state, context, "index.html")
}

pub async fn sign_up(state: web::Data<AppState>, session: Session) -> impl Responder {
    let context = template::create_context(&session);
    let (context, _user_account) = handle_sign_up(state.get_ref(), None, context).await;
    respond_with_template(state, context, "signup.html")
}

pub async fn sign_up_post(state: web::Data<AppState>, form: web::Form<SignUpForm>, session: Session) -> impl Responder {
    let context = template::create_context(&session);
    let (context, user_account) = handle_sign_up(state.get_ref(), Some(form.into_inner()), context).await;

    match user_account {
        Some(_) => respond_with_template(state, context, "signup_ok.html"),
        None => respond_with_template(state, context, "signup.html")
    }
}

pub async fn sign_in(state: web::Data<AppState>, session: Session) -> impl Responder {
    let context = template::create_context(&session);
    let (context, _user_account) = handle_sign_in(state.get_ref(), None, context).await;
    respond_with_template(state, context, "signin.html")
}

pub async fn sign_in_post(state: web::Data<AppState>, form: web::Form<SignInForm>, session: Session) -> impl Responder {
    let context = template::create_context(&session);
    let (mut context, user_account) = handle_sign_in(state.get_ref(), Some(form.into_inner()), context).await;

    match user_account {
        Some(ua) => {
            let _ = session.insert("full_name", ua.full_name());
            let _ = session.insert("email", ua.email);
            return Either::Left(web::Redirect::to("/").see_other())
        },
        None => context.insert("error", "These credentials don't match your account. Please, try again."),
    }

    Either::Right(respond_with_template(state, context.clone(), "signin.html"))
}

pub async fn sign_out(session: Session) -> impl Responder {
    session.purge();
    web::Redirect::to("/").see_other()
}

pub async fn profile(state: web::Data<AppState>, session: Session) -> impl Responder {
    let mut context = template::create_context(&session);

    if let Ok(Some(email)) = session.get::<String>("email") {
        let user_account = repository::get_user_account_by_email(&state.pool, email).await;
        match user_account {
            Ok(ua) => {
                context.insert("first_name", &ua.first_name);
                context.insert("last_name", &ua.last_name);
                context.insert("birth_date", &ua.birth_date);
                context.insert("email", &ua.email);
                context.insert("created", &ua.created);
                context.insert("modified", &ua.modified);
            },
            Err(e) => {
                log::error!("Error retrieving user account: {}", e);
                return Either::Left(web::Redirect::to("/").see_other())
            }
        }
    }
    else {
        return Either::Left(web::Redirect::to("/").see_other())
    }
    Either::Right(respond_with_template(state, context, "profile.html"))
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