use crate::constants::GITHUB_RATE_LIMIT_MODE_ENV;

#[derive(Debug, Clone)]
pub enum RateLimitMode {
    Defer,
    Sleep,
}

impl RateLimitMode {
    pub fn from_env() -> Self {
        match std::env::var(GITHUB_RATE_LIMIT_MODE_ENV)
            .unwrap_or_else(|_| "defer".to_string())
            .to_lowercase()
            .as_str()
        {
            "sleep" => Self::Sleep,
            _ => Self::Defer,
        }
    }
}
