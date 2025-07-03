use serde::Deserialize;
use std::collections::HashSet;

#[derive(Debug, Clone, Deserialize)]
pub struct FilterConfig {
    pub blocked_methods: HashSet<String>,
}
