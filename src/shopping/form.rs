use std::collections::HashMap;
use crate::shopping::model::{Good, GoodLocation};

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
