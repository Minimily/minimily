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
    let context = Context::new();
    context
}