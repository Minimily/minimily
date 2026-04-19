use actix_session::Session;
use actix_web::{web, HttpResponse};
use tera::{Context, Tera};
use crate::model::AppState;

fn filter_format_phone(value: &tera::Value, _args: &std::collections::HashMap<String, tera::Value>) -> tera::Result<tera::Value> {
    let phone = tera::try_get_value!("format_phone", "value", String, value);
    let digits: String = phone.chars().filter(|c| c.is_ascii_digit()).collect();
    let formatted = if digits.len() == 10 {
        format!("({}) {}-{}", &digits[0..3], &digits[3..6], &digits[6..10])
    } else {
        phone
    };
    Ok(tera::to_value(formatted).unwrap())
}

pub fn preload_templates() -> Tera {
    let mut tera = match Tera::new("content/templates/**/*.html") {
        Ok(t) => t,
        Err(e) => {
            log::error!("Template parsing error(s): {}", e);
            std::process::exit(1);
        }
    };
    tera.register_filter("format_phone", filter_format_phone);
    tera.full_reload().expect("Failed to reload templates");
    tera
}

pub fn create_context(session: &Session) -> Context {
    let mut context = Context::new();
    context.insert("error", "");
    context.insert("session_fullname", &session.get::<String>("full_name").unwrap_or_default());
    context
}

pub fn respond_with_template(state: web::Data<AppState>, context: Context, template: &str) -> HttpResponse {
    HttpResponse::Ok()
        .content_type("text/html; charset=utf-8")
        .body(
            state.get_ref().template.render(template, &context).unwrap()
        )
}
