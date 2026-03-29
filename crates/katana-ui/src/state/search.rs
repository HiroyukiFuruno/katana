use std::collections::HashSet;
use std::path::PathBuf;

pub struct SearchState {
    pub filter_enabled: bool,
    pub filter_query: String,
    pub filter_cache: Option<(String, HashSet<PathBuf>)>,
    pub query: String,
    pub include_pattern: String,
    pub exclude_pattern: String,
    pub last_params: Option<(String, String, String)>,
    pub results: Vec<PathBuf>,
}

impl Default for SearchState {
    fn default() -> Self {
        Self::new()
    }
}

impl SearchState {
    pub fn new() -> Self {
        Self {
            filter_enabled: false,
            filter_query: String::new(),
            filter_cache: None,
            query: String::new(),
            include_pattern: String::new(),
            exclude_pattern: String::new(),
            last_params: None,
            results: Vec::new(),
        }
    }
}
