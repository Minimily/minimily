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

pub async fn sign_in(state: web::Data<AppState>, session: Session) -> Either<web::Redirect, HttpResponse> {
    let num_user_accounts = match repository::num_user_accounts(&state.pool).await {
        Ok(num) => num,
        Err(e) => {
            log::error!("Error checking the number of user accounts: {}", e);
            0
        }
    };

    if num_user_accounts == 0 {
        return Either::Left(web::Redirect::to("/signup").see_other());
    }
    
    let context = template::create_context(&session);
    let (context, _user_account) = handle_sign_in(state.get_ref(), None, context).await;
    Either::Right(respond_with_template(state, context, "signin.html"))
}

pub async fn sign_in_post(state: web::Data<AppState>, form: web::Form<SignInForm>, session: Session) -> impl Responder {
    let context = template::create_context(&session);
    let (mut context, user_account) = handle_sign_in(state.get_ref(), Some(form.into_inner()), context).await;

    match user_account {
        Some(ua) => {
            let _ = session.insert("full_name", ua.full_name());
            let _ = session.insert("email", ua.email);
            let _ = session.insert("id", ua.id);
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
        let profile = repository::get_profile(&state.pool, email).await;
        match profile {
            Ok(p) => {
                context.insert("user_account", &p.user_account);

                let family = repository::get_family_profile(&state.pool, p).await;
                match family {
                    Ok(f) => context.insert("family", &f),
                    Err(e) => log::error!("Error retrieving family profile: {}", e)
                }
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