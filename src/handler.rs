use tera::Context;
use crate::model::AppState;
use crate::form::UserAccountForm;
use crate::{repository, template};

pub async fn handle_signup(state: &AppState, form: Option<UserAccountForm>) -> Context {
    let mut context = template::create_context(state);

    match form {
        Some(form) => {
            // Fist name validation
            let (user_account, errors) = &form.validate(state).await;

            match user_account {
                // A user_account was successfully created from a form
                Some(user_account) => {
                    let created_account = repository::create_user_account(&state.pool, user_account.clone()).await;
                    match created_account {
                        Ok(_) => {
                           context.insert("next", "signup_ok.html");                            
                        }
                        Err(e) => {
                            context.insert("error", &e.to_string());
                            context.insert("errors", &errors);
                            context.insert("user_account", &form)
                        }
                    }
                },
                // Validation failed
                None => {
                    context.insert("errors", &errors);
                    context.insert("user_account", &form)
                }
            }
        },
        None => {
            let user_account_form = UserAccountForm {
                first_name: "".to_string(),
                last_name: "".to_string(),
                birth_date: None,
                email: None,
                password: None,
                confirm_password: None,
            };
            context.insert("errors", &user_account_form.get_errors());
            context.insert("user_account", &user_account_form);
        },
    }

    context.insert("confirm_password", "");

    context
}