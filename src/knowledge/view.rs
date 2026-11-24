use crate::state::AppState;
use crate::template;
use crate::template::respond_with_template;
use actix_session::Session;
use actix_web::{Either, HttpResponse, Responder, web};

async fn render_knowledge(
    state: web::Data<AppState>,
    session: &Session,
    _user_id: i32,
) -> HttpResponse {
    let context = template::create_context(session);

    respond_with_template(state, context, "knowledge/knowledge.html")
}

pub async fn knowledge(state: web::Data<AppState>, session: Session) -> impl Responder {
    let user_id = match session.get::<i32>("id") {
        Ok(Some(id)) => id,
        _ => return Either::Left(web::Redirect::to("/signin").see_other()),
    };

    Either::Right(render_knowledge(state, &session, user_id).await)
}
