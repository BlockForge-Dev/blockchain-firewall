use serde::Deserialize;
use std::fs;
use once_cell::sync::Lazy;

#[derive(Debug, Deserialize)]
pub struct FilterRules {
    pub deny_methods: Vec<String>,
}

pub static RULES: Lazy<FilterRules> = Lazy::new(|| {
    let yaml = fs::read_to_string("config/rules.yaml")
        .expect("Failed to read rules.yaml");
    serde_yaml::from_str(&yaml)
        .expect("Failed to parse rules.yaml")
});

pub fn is_blocked(method: &str) -> bool {
    RULES.deny_methods.iter().any(|m| m == method)
}
