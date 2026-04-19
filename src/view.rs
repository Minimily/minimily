use actix_session::Session;
use actix_web::{web, HttpResponse, Responder, Either};
use chrono::NaiveDate;
use crate::handler::{handle_sign_in, handle_sign_up};
use crate::model::AppState;
use crate::form::{EditProfileForm, SignInForm, SignUpForm};
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

pub async fn profile_edit(state: web::Data<AppState>, session: Session) -> impl Responder {
    let mut context = template::create_context(&session);

    let session_email = match session.get::<String>("email") {
        Ok(Some(email)) => email,
        _ => return Either::Left(web::Redirect::to("/").see_other()),
    };

    match repository::get_user_account_by_email(&state.pool, session_email).await {
        Ok(ua) => {
            let form = EditProfileForm {
                first_name: ua.first_name,
                last_name: ua.last_name,
                birth_date: ua.birth_date.map(|d| d.format("%Y-%m-%d").to_string()),
                email: ua.email.unwrap_or_default(),
            };
            let errors = form.get_errors();
            context.insert("form", &form);
            context.insert("errors", &errors);
        },
        Err(e) => {
            log::error!("Error retrieving user account for edit: {}", e);
            return Either::Left(web::Redirect::to("/").see_other());
        }
    }

    Either::Right(respond_with_template(state, context, "profile_edit.html"))
}

pub async fn profile_edit_post(state: web::Data<AppState>, form: web::Form<EditProfileForm>, session: Session) -> impl Responder {
    let mut context = template::create_context(&session);

    let user_id = match session.get::<i32>("id") {
        Ok(Some(id)) => id,
        _ => return Either::Left(web::Redirect::to("/").see_other()),
    };

    let form = form.into_inner();
    let (valid, errors) = form.validate(&state, user_id).await;

    if valid {
        let birth_date = form.birth_date.as_deref()
            .filter(|s| !s.is_empty())
            .and_then(|s| NaiveDate::parse_from_str(s, "%Y-%m-%d").ok());

        match repository::update_user_account(&state.pool, user_id, &form.first_name, &form.last_name, birth_date, &form.email).await {
            Ok(_) => {
                let _ = session.insert("full_name", format!("{} {}", form.first_name, form.last_name));
                let _ = session.insert("email", form.email.clone());
                return Either::Left(web::Redirect::to("/profile").see_other());
            },
            Err(e) => {
                log::error!("Error updating user account: {}", e);
                context.insert("error", &e.to_string());
                context.insert("errors", &errors);
                context.insert("form", &form);
            }
        }
    } else {
        context.insert("errors", &errors);
        context.insert("form", &form);
    }

    Either::Right(respond_with_template(state, context, "profile_edit.html"))
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