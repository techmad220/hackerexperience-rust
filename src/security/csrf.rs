use actix_web::Error;
use rand::Rng;
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;
use chrono::{DateTime, Utc, Duration};

/// CSRF Protection implementation
pub struct CsrfProtection {
    token_length: usize,
    tokens: Arc<RwLock<HashMap<String, (String, DateTime<Utc>)>>>,
    token_lifetime: i64, // seconds
}

impl CsrfProtection {
    pub fn new(token_length: usize) -> Self {
        Self {
            token_length,
            tokens: Arc::new(RwLock::new(HashMap::new())),
            token_lifetime: 3600, // 1 hour
        }
    }

    pub fn generate_token(&self) -> String {
        let mut rng = rand::thread_rng();
        let token: String = (0..self.token_length)
            .map(|_| {
                let idx = rng.gen_range(0..62);
                match idx {
                    0..=9 => (b'0' + idx) as char,
                    10..=35 => (b'A' + idx - 10) as char,
                    36..=61 => (b'a' + idx - 36) as char,
                    _ => unreachable!(),
                }
            })
            .collect();
        token
    }

    pub async fn store_token(&self, session_id: &str, token: &str) {
        let mut tokens = self.tokens.write().await;
        tokens.insert(session_id.to_string(), (token.to_string(), Utc::now()));
    }

    pub async fn validate_token(&self, token: &str, session_id: &str) -> Result<(), Error> {
        let tokens = self.tokens.read().await;

        if let Some((stored_token, timestamp)) = tokens.get(session_id) {
            if stored_token != token {
                return Err(actix_web::error::ErrorForbidden("Invalid CSRF token"));
            }

            let now = Utc::now();
            if (now - *timestamp).num_seconds() > self.token_lifetime {
                return Err(actix_web::error::ErrorForbidden("CSRF token expired"));
            }

            Ok(())
        } else {
            Err(actix_web::error::ErrorForbidden("CSRF token not found"))
        }
    }

    pub async fn clean_expired_tokens(&self) {
        let mut tokens = self.tokens.write().await;
        let now = Utc::now();
        let expiry_time = now - Duration::seconds(self.token_lifetime);

        tokens.retain(|_, (_, timestamp)| *timestamp > expiry_time);
    }
}