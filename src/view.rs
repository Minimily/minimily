use std::collections::HashMap;
use actix_session::Session;
use actix_web::{web, HttpResponse, Responder, Either};
use tera::Context;
use chrono::NaiveDate;
use crate::handler::{handle_sign_in, handle_sign_up};
use crate::model::{AppState, Good, GoodLocation, GroupType, UserAccount};
use crate::form::{CreateFamilyForm, EditProfileForm, GoodForm, GoodLocationForm, SignInForm, SignUpForm};
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
        let user_account = repository::get_user_account_by_email(&state.pool, email).await;
        match user_account {
            Ok(mut ua) => {
                ua.password = None;
                context.insert("user_account", &ua);

                match repository::get_user_group(&state.pool, ua.id, GroupType::Family).await {
                    Ok(f) => {
                        context.insert("family", &f);
                        match repository::get_group_members(&state.pool, f.id).await {
                            Ok(m) => context.insert("family_members", &m),
                            Err(e) => log::error!("Error retrieving family members: {}", e)
                        }
                    },
                    Err(sqlx::Error::RowNotFound) => context.insert("no_family", &true),
                    Err(e) => log::error!("Error retrieving family: {}", e)
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
                first_name: ua.first_name.unwrap(),
                last_name: ua.last_name.unwrap(),
                birth_date: ua.birth_date.map(|d| d.format("%Y-%m-%d").to_string()),
                email: ua.email.unwrap_or_default(),
                phone: ua.phone,
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

        let user_account = UserAccount {
            id: user_id,
            first_name: Some(form.first_name.clone()),
            last_name: Some(form.last_name.clone()),
            birth_date,
            email: Some(form.email.clone()),
            phone: form.phone.clone().filter(|s| !s.is_empty()),
            password: None,
            created: None,
            modified: None,
        };
        match repository::update_user_account(&state.pool, &user_account).await {
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

pub async fn family_create_post(state: web::Data<AppState>, form: web::Form<CreateFamilyForm>, session: Session) -> web::Redirect {
    let user_id = match session.get::<i32>("id") {
        Ok(Some(id)) => id,
        _ => return web::Redirect::to("/").see_other(),
    };

    let form = form.into_inner();
    if let Err(e) = repository::create_group(&state.pool, &form.name, GroupType::Family, user_id).await {
        log::error!("Error creating family: {}", e);
    }
    web::Redirect::to("/profile").see_other()
}

fn good_form_context(good_id: Option<i32>, form: &GoodForm, errors: &HashMap<&str, String>) -> Context {
    let mut context = Context::new();
    context.insert("good_id", &good_id);
    context.insert("good_form", form);
    context.insert("good_errors", errors);
    // Reopen the good modal when the submitted form has validation errors.
    context.insert("show_good_modal", &errors.values().any(|e| !e.is_empty()));
    context
}

fn location_form_context(location_id: Option<i32>, form: &GoodLocationForm, errors: &HashMap<&str, String>) -> Context {
    let mut context = Context::new();
    context.insert("location_id", &location_id);
    context.insert("location_form", form);
    context.insert("location_errors", errors);
    // Reopen the location modal when the submitted form has validation errors.
    context.insert("show_location_modal", &errors.values().any(|e| !e.is_empty()));
    context
}

#[derive(serde::Deserialize)]
pub struct ShoppingFilter {
    location: Option<String>,
}

impl ShoppingFilter {
    fn location_id(&self) -> Option<i32> {
        self.location.as_deref().and_then(|l| l.parse().ok())
    }
}

fn shopping_url(location_id: Option<i32>) -> String {
    match location_id {
        Some(id) => format!("/shopping?location={}", id),
        None => "/shopping".to_string(),
    }
}

// Renders the shopping page with empty good and location forms, unless form_context brings a submitted one.
async fn render_shopping(state: web::Data<AppState>, session: &Session, user_id: i32, location_filter: Option<i32>, form_context: Context) -> HttpResponse {
    let mut context = template::create_context(session);

    context.insert("location_filter", &location_filter);
    match repository::get_goods(&state.pool, user_id, location_filter).await {
        Ok(goods) => context.insert("goods", &goods),
        Err(e) => {
            log::error!("Error retrieving goods: {}", e);
            context.insert("error", "Unable to load the goods.");
        }
    }
    match repository::get_good_locations(&state.pool, user_id).await {
        Ok(locations) => context.insert("locations", &locations),
        Err(e) => {
            log::error!("Error retrieving good locations: {}", e);
            context.insert("error", "Unable to load the locations.");
        }
    }

    let good_form = GoodForm::default();
    context.extend(good_form_context(None, &good_form, &good_form.get_errors()));
    let location_form = GoodLocationForm::default();
    context.extend(location_form_context(None, &location_form, &location_form.get_errors()));
    context.extend(form_context);

    respond_with_template(state, context, "shopping.html")
}

pub async fn shopping(state: web::Data<AppState>, filter: web::Query<ShoppingFilter>, session: Session) -> impl Responder {
    let user_id = match session.get::<i32>("id") {
        Ok(Some(id)) => id,
        _ => return Either::Left(web::Redirect::to("/signin").see_other()),
    };

    Either::Right(render_shopping(state, &session, user_id, filter.location_id(), Context::new()).await)
}

pub async fn good_create_post(state: web::Data<AppState>, form: web::Form<GoodForm>, session: Session) -> impl Responder {
    let user_id = match session.get::<i32>("id") {
        Ok(Some(id)) => id,
        _ => return Either::Left(web::Redirect::to("/signin").see_other()),
    };

    let form = form.into_inner();
    let (valid, errors) = form.validate();
    if !valid {
        return Either::Right(render_shopping(state, &session, user_id, form.location().map(|l| l.id), good_form_context(None, &form, &errors)).await);
    }

    // Goods are shared with the user's family, when there is one.
    let user_group = match repository::get_user_group(&state.pool, user_id, GroupType::Family).await {
        Ok(f) => Some(f),
        Err(sqlx::Error::RowNotFound) => None,
        Err(e) => {
            log::error!("Error retrieving family: {}", e);
            None
        }
    };

    let good = Good {
        id: 0,
        user_group,
        name: form.name.trim().to_string(),
        description: form.description(),
        location: form.location(),
        created_by: Some(UserAccount::new_user_account(user_id)),
        created_date: None,
    };
    if let Err(e) = repository::create_good(&state.pool, &good).await {
        log::error!("Error creating good: {}", e);
    }
    Either::Left(web::Redirect::to(shopping_url(good.location.map(|l| l.id))).see_other())
}

pub async fn good(state: web::Data<AppState>, path: web::Path<i32>, session: Session) -> impl Responder {
    let mut context = template::create_context(&session);

    let user_id = match session.get::<i32>("id") {
        Ok(Some(id)) => id,
        _ => return Either::Left(web::Redirect::to("/signin").see_other()),
    };

    match repository::get_good(&state.pool, path.into_inner(), user_id).await {
        Ok(good) => context.insert("good", &good),
        Err(e) => {
            log::error!("Error retrieving good: {}", e);
            return Either::Left(web::Redirect::to("/shopping").see_other());
        }
    }

    Either::Right(respond_with_template(state, context, "good.html"))
}

pub async fn good_edit_post(state: web::Data<AppState>, path: web::Path<i32>, form: web::Form<GoodForm>, session: Session) -> impl Responder {
    let user_id = match session.get::<i32>("id") {
        Ok(Some(id)) => id,
        _ => return Either::Left(web::Redirect::to("/signin").see_other()),
    };

    let good_id = path.into_inner();
    let form = form.into_inner();
    let (valid, errors) = form.validate();
    if !valid {
        return Either::Right(render_shopping(state, &session, user_id, form.location().map(|l| l.id), good_form_context(Some(good_id), &form, &errors)).await);
    }

    let good = Good {
        id: good_id,
        user_group: None,
        name: form.name.trim().to_string(),
        description: form.description(),
        location: form.location(),
        created_by: None,
        created_date: None,
    };
    if let Err(e) = repository::update_good(&state.pool, &good, user_id).await {
        log::error!("Error updating good: {}", e);
    }
    Either::Left(web::Redirect::to(shopping_url(good.location.map(|l| l.id))).see_other())
}

pub async fn good_delete_post(state: web::Data<AppState>, path: web::Path<i32>, session: Session) -> web::Redirect {
    let user_id = match session.get::<i32>("id") {
        Ok(Some(id)) => id,
        _ => return web::Redirect::to("/signin").see_other(),
    };

    // Return to the list the good was in.
    let good_id = path.into_inner();
    let location_id = match repository::get_good(&state.pool, good_id, user_id).await {
        Ok(good) => good.location.map(|l| l.id),
        Err(e) => {
            log::error!("Error retrieving good to delete: {}", e);
            return web::Redirect::to("/shopping").see_other();
        }
    };

    if let Err(e) = repository::delete_good(&state.pool, good_id, user_id).await {
        log::error!("Error deleting good: {}", e);
    }
    web::Redirect::to(shopping_url(location_id)).see_other()
}

pub async fn location_create_post(state: web::Data<AppState>, form: web::Form<GoodLocationForm>, session: Session) -> impl Responder {
    let user_id = match session.get::<i32>("id") {
        Ok(Some(id)) => id,
        _ => return Either::Left(web::Redirect::to("/signin").see_other()),
    };

    let form = form.into_inner();
    let (valid, errors) = form.validate();
    if !valid {
        return Either::Right(render_shopping(state, &session, user_id, None, location_form_context(None, &form, &errors)).await);
    }

    let location = GoodLocation {
        id: 0,
        name: Some(form.name.trim().to_string()),
        description: form.description(),
        created_by: Some(UserAccount::new_user_account(user_id)),
        created_date: None,
    };
    if let Err(e) = repository::create_good_location(&state.pool, &location).await {
        log::error!("Error creating good location: {}", e);
    }
    Either::Left(web::Redirect::to("/shopping").see_other())
}

pub async fn location_edit_post(state: web::Data<AppState>, path: web::Path<i32>, form: web::Form<GoodLocationForm>, session: Session) -> impl Responder {
    let user_id = match session.get::<i32>("id") {
        Ok(Some(id)) => id,
        _ => return Either::Left(web::Redirect::to("/signin").see_other()),
    };

    let location_id = path.into_inner();
    let form = form.into_inner();
    let (valid, errors) = form.validate();
    if !valid {
        return Either::Right(render_shopping(state, &session, user_id, None, location_form_context(Some(location_id), &form, &errors)).await);
    }

    let location = GoodLocation {
        id: location_id,
        name: Some(form.name.trim().to_string()),
        description: form.description(),
        created_by: None,
        created_date: None,
    };
    if let Err(e) = repository::update_good_location(&state.pool, &location, user_id).await {
        log::error!("Error updating good location: {}", e);
    }
    Either::Left(web::Redirect::to("/shopping").see_other())
}

pub async fn location_delete_post(state: web::Data<AppState>, path: web::Path<i32>, session: Session) -> web::Redirect {
    let user_id = match session.get::<i32>("id") {
        Ok(Some(id)) => id,
        _ => return web::Redirect::to("/signin").see_other(),
    };

    if let Err(e) = repository::delete_good_location(&state.pool, path.into_inner(), user_id).await {
        log::error!("Error deleting good location: {}", e);
    }
    web::Redirect::to("/shopping").see_other()
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