use std::collections::HashMap;
use bcrypt::{hash, DEFAULT_COST};
use chrono::{NaiveDate};
use crate::model::{AppState, UserAccount};
use crate::repository;

#[derive(serde::Serialize, serde::Deserialize, Clone)]
pub struct UserAccountForm {
    pub first_name: String,
    pub last_name: String,
    pub birth_date: Option<String>,
    pub email: Option<String>,
    pub password: Option<String>,
    pub confirm_password: Option<String>,
}

impl UserAccountForm {
    pub fn get_errors(&self) -> HashMap<&str, String> {
        HashMap::from([
            ("first_name", "".to_string()),
            ("last_name", "".to_string()),
            ("email", "".to_string()),
            ("birth_date", "".to_string()),
            ("password", "".to_string()),
            ("confirm_password", "".to_string()),
        ])
    }

    pub async fn validate(&self, state: &AppState) -> (Option<UserAccount>, HashMap<&str, String>) {
        let mut errors = self.get_errors();
        let mut valid = true;

        if self.first_name.is_empty() {
            errors.insert("first_name", "First name cannot be empty".to_string());
            valid = false;
        }

        // Last name validation
        if self.last_name.is_empty() {
            errors.insert("last_name", "Last name cannot be empty".to_string());
            valid = false;
        }

        // Email validation
        let email = self.email.clone().unwrap();
        let existing_user_account = repository::get_user_account_by_email(&state.pool, email).await;
        match existing_user_account {
            Ok(ua) => {
                errors.insert("email", format!("Email {:?} already exists", ua.email.clone().unwrap()));
                valid = false;
            },
            Err(_) => {}
        }

        // Date of birth validation
        let birth_date_str = self.birth_date.clone().unwrap_or("".to_string());
        let mut birth_date = None;
        if !birth_date_str.is_empty() {
            match NaiveDate::parse_from_str(&birth_date_str, "%Y-%m-%d") {
                Ok(date) => {
                    log::info!("Valid birth date: {:?}", date);
                    birth_date = Some(date);
                },
                Err(_) => {
                    errors.insert("birth_date", "Invalid date format".to_string());
                    valid = false;
                }
            };
        }

        // Password validation
        if self.password != self.confirm_password {
            errors.insert("confirm_password", "Passwords do not match".to_string());
            valid = false;
        }
        if self.password.is_none() || self.password.as_ref().unwrap().is_empty() {
            errors.insert("password", "Password cannot be empty".to_string());
            valid = false;
        }
        if self.password.as_ref().unwrap().len() < 8 {
            errors.insert("password", "Password must be at least 8 characters long".to_string());
            valid = false;
        }

        let user_account = if valid {
            let hashed_password = hash(self.password.clone().unwrap().as_str(), DEFAULT_COST).unwrap();

            Some(UserAccount {
                id: 0,
                first_name: self.first_name.clone(),
                last_name: self.last_name.clone(),
                email: self.email.clone(),
                birth_date,
                password: Some(hashed_password),
                created: None,
                modified: None,
            })
        } else {
            None
        };

        (user_account, errors)
    }
}