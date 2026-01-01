use actix_web::{web, HttpResponse};
use tera::{Context, Tera};
use crate::model::AppState;

pub fn preload_templates() -> Tera {
    let mut tera = match Tera::new("content/templates/**/*.html") {
        Ok(t) => t,
        Err(e) => {
            log::error!("Template parsing error(s): {}", e);
            std::process::exit(1);
        }
    };
    tera.full_reload().expect("Failed to reload templates");
    tera
}

pub fn create_context(_state: &AppState) -> Context {
    let mut context = Context::new();
    context.insert("error", "");
    context
}

pub fn respond_with_template(state: web::Data<AppState>, context: Context, template: &str) -> HttpResponse {
    HttpResponse::Ok()
        .content_type("text/html; charset=utf-8")
        .body(
            state.get_ref().template.render(template, &context).unwrap()
        )
}
