use tera::Context;
use crate::model::AppState;
use crate::form::{SignInForm, SignUpForm};
use crate::{repository, template};

pub async fn handle_signup(state: &AppState, form: Option<SignUpForm>) -> Context {
    let mut context = template::create_context(state);

    match form {
        Some(form) => {
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
                            context.insert("form", &form)
                        }
                    }
                },
                // Validation failed
                None => {
                    context.insert("errors", &errors);
                    context.insert("form", &form)
                }
            }
        },
        None => {
            let user_account_form = SignUpForm {
                first_name: "".to_string(),
                last_name: "".to_string(),
                birth_date: None,
                email: None,
                password: None,
                confirm_password: None,
            };
            context.insert("errors", &user_account_form.get_errors());
            context.insert("form", &user_account_form);
        },
    }

    context.insert("confirm_password", "");

    context
}

pub async fn handle_signin(state: &AppState, form: Option<SignInForm>) -> Context {
    let mut context = template::create_context(state);

    match form {
        Some(form) => {
            let (user_account, _errors) = &form.validate(state).await;

            match user_account {
                Some(_) => {
                    context.insert("redirect", "/");
                },
                None => {
                    context.insert("error", "These credentials don't match your account. Please, try again.");
                    let signin_form = SignInForm { email: form.email.to_string(), password: "".to_string() };
                    context.insert("form", &signin_form);
                }
            }
        },
        None => {
            let signin_form = SignInForm { email: "".to_string(), password: "".to_string() };
            context.insert("form", &signin_form);
        },
    }
    context
}