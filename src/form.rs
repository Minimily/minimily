use std::collections::HashMap;
use bcrypt::{hash, DEFAULT_COST};
use chrono::{NaiveDate};
use crate::model::{AppState, Good, GoodLocation, UserAccount};
use crate::repository;

#[derive(serde::Serialize, serde::Deserialize, Clone)]
pub struct SignUpForm {
    pub first_name: String,
    pub last_name: String,
    pub birth_date: Option<String>,
    pub email: Option<String>,
    pub password: Option<String>,
    pub confirm_password: Option<String>,
}

impl SignUpForm {
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
                first_name: Some(self.first_name.clone()),
                last_name: Some(self.last_name.clone()),
                email: self.email.clone(),
                phone: None,
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

#[derive(serde::Serialize, serde::Deserialize, Clone)]
pub struct EditProfileForm {
    pub first_name: String,
    pub last_name: String,
    pub birth_date: Option<String>,
    pub email: String,
    pub phone: Option<String>,
}

impl EditProfileForm {
    pub fn get_errors(&self) -> HashMap<&str, String> {
        HashMap::from([
            ("first_name", "".to_string()),
            ("last_name", "".to_string()),
            ("email", "".to_string()),
            ("birth_date", "".to_string()),
            ("phone", "".to_string()),
        ])
    }

    pub async fn validate(&self, state: &AppState, current_id: i32) -> (bool, HashMap<&str, String>) {
        let mut errors = self.get_errors();
        let mut valid = true;

        if self.first_name.is_empty() {
            errors.insert("first_name", "First name cannot be empty".to_string());
            valid = false;
        }

        if self.last_name.is_empty() {
            errors.insert("last_name", "Last name cannot be empty".to_string());
            valid = false;
        }

        if self.email.is_empty() {
            errors.insert("email", "Email cannot be empty".to_string());
            valid = false;
        } else if let Ok(ua) = repository::get_user_account_by_email(&state.pool, self.email.clone()).await {
            if ua.id != current_id {
                errors.insert("email", format!("Email {} is already in use", self.email));
                valid = false;
            }
        }

        let birth_date_str = self.birth_date.clone().unwrap_or_default();
        if !birth_date_str.is_empty() {
            if NaiveDate::parse_from_str(&birth_date_str, "%Y-%m-%d").is_err() {
                errors.insert("birth_date", "Invalid date format".to_string());
                valid = false;
            }
        }

        (valid, errors)
    }
}

#[derive(serde::Deserialize)]
pub struct CreateFamilyForm {
    pub name: String,
}

#[derive(serde::Serialize, serde::Deserialize)]
pub struct SignInForm {
    pub email: String,
    pub password: String,
}

impl SignInForm {
    pub async fn validate(&self, state: &AppState) -> (Option<UserAccount>, HashMap<&str, String>) {
        let user_account = repository::get_user_account_by_email(&state.pool, self.email.clone()).await;
        match user_account {
            Ok(ua) => {
                if bcrypt::verify(self.password.as_str(), ua.password.as_ref().unwrap()).unwrap() {
                    return (Some(ua), HashMap::new());
                }
            },
            Err(e) => {
                log::error!("Error getting user by email: {:?}", e);
            }
        }
        (None, HashMap::new())
    }
}
#[derive(serde::Serialize, serde::Deserialize, Clone, Default)]
pub struct GoodForm {
    pub name: String,
    pub description: Option<String>,
    pub location: Option<String>,
    // Set to "good" when the form is submitted from the good's page, so we return there.
    pub return_to: Option<String>,
}

impl GoodForm {
    pub fn from_good(good: &Good) -> Self {
        GoodForm {
            name: good.name.clone(),
            description: good.description.clone(),
            location: good.location.as_ref().map(|l| l.id.to_string()),
            return_to: Some("good".to_string()),
        }
    }

    pub fn returns_to_good(&self) -> bool {
        self.return_to.as_deref() == Some("good")
    }

    pub fn get_errors(&self) -> HashMap<&str, String> {
        HashMap::from([
            ("name", "".to_string()),
            ("description", "".to_string()),
            ("location", "".to_string()),
        ])
    }

    pub fn validate(&self) -> (bool, HashMap<&str, String>) {
        let mut errors = self.get_errors();
        let mut valid = true;

        if self.name.trim().is_empty() {
            errors.insert("name", "Name cannot be empty".to_string());
            valid = false;
        } else if self.name.trim().chars().count() > 100 {
            errors.insert("name", "Name cannot be longer than 100 characters".to_string());
            valid = false;
        }

        if self.location.as_deref().is_some_and(|l| !l.is_empty() && l.parse::<i32>().is_err()) {
            errors.insert("location", "Invalid location".to_string());
            valid = false;
        }

        (valid, errors)
    }

    pub fn location(&self) -> Option<GoodLocation> {
        self.location.as_deref()
            .and_then(|l| l.parse::<i32>().ok())
            .map(GoodLocation::new_good_location)
    }

    pub fn description(&self) -> Option<String> {
        self.description.as_deref()
            .map(str::trim)
            .filter(|s| !s.is_empty())
            .map(str::to_string)
    }
}

#[derive(serde::Serialize, serde::Deserialize, Clone, Default)]
pub struct GoodLocationForm {
    pub name: String,
    pub description: Option<String>,
}

impl GoodLocationForm {
    pub fn get_errors(&self) -> HashMap<&str, String> {
        HashMap::from([
            ("name", "".to_string()),
            ("description", "".to_string()),
        ])
    }

    pub fn validate(&self) -> (bool, HashMap<&str, String>) {
        let mut errors = self.get_errors();
        let mut valid = true;

        if self.name.trim().is_empty() {
            errors.insert("name", "Name cannot be empty".to_string());
            valid = false;
        } else if self.name.trim().chars().count() > 100 {
            errors.insert("name", "Name cannot be longer than 100 characters".to_string());
            valid = false;
        }

        (valid, errors)
    }

    pub fn description(&self) -> Option<String> {
        self.description.as_deref()
            .map(str::trim)
            .filter(|s| !s.is_empty())
            .map(str::to_string)
    }
}
