use std::collections::HashMap;
use actix_session::Session;
use actix_web::{web, HttpResponse, Responder, Either};
use tera::Context;
use crate::account::model::{GroupType, UserAccount};
use crate::account;
use crate::shopping::model::{Good, GoodLocation};
use crate::shopping::form::{GoodForm, GoodLocationForm};
use crate::shopping::repository;
use crate::state::AppState;
use crate::template;
use crate::template::respond_with_template;

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
    let user_group = match account::repository::get_user_group(&state.pool, user_id, GroupType::Family).await {
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

// Renders the good's page with its edit form filled with the good, unless form_context brings a submitted one.
async fn render_good(state: web::Data<AppState>, session: &Session, user_id: i32, good_id: i32, form_context: Context) -> Either<web::Redirect, HttpResponse> {
    let mut context = template::create_context(session);

    match repository::get_good(&state.pool, good_id, user_id).await {
        Ok(good) => {
            let good_form = GoodForm::from_good(&good);
            context.extend(good_form_context(Some(good_id), &good_form, &good_form.get_errors()));
            context.insert("good", &good);
        }
        Err(e) => {
            log::error!("Error retrieving good: {}", e);
            return Either::Left(web::Redirect::to("/shopping").see_other());
        }
    }

    match repository::get_good_locations(&state.pool, user_id).await {
        Ok(locations) => context.insert("locations", &locations),
        Err(e) => {
            log::error!("Error retrieving good locations: {}", e);
            context.insert("error", "Unable to load the locations.");
        }
    }

    context.extend(form_context);

    Either::Right(respond_with_template(state, context, "good.html"))
}

pub async fn good(state: web::Data<AppState>, path: web::Path<i32>, session: Session) -> impl Responder {
    let user_id = match session.get::<i32>("id") {
        Ok(Some(id)) => id,
        _ => return Either::Left(web::Redirect::to("/signin").see_other()),
    };

    render_good(state, &session, user_id, path.into_inner(), Context::new()).await
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
        if form.returns_to_good() {
            return render_good(state, &session, user_id, good_id, good_form_context(Some(good_id), &form, &errors)).await;
        }
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
    if form.returns_to_good() {
        return Either::Left(web::Redirect::to(format!("/shopping/goods/{}", good_id)).see_other());
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
