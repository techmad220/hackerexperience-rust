//! OAuth 2.0 and OpenID Connect implementation for multiple providers

use anyhow::{anyhow, Result};
use async_trait::async_trait;
use chrono::{DateTime, Duration, Utc};
use rand::{thread_rng, Rng};
use reqwest::Client;
use serde::{Deserialize, Serialize};
use serde_json::Value;
use sha2::{Digest, Sha256};
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;
use tracing::{debug, error, info, warn};
use uuid::Uuid;
use base64::{Engine as _, engine::general_purpose::URL_SAFE_NO_PAD};

/// OAuth configuration
#[derive(Debug, Clone)]
pub struct OAuthConfig {
    /// Enable OAuth
    pub enabled: bool,
    /// Redirect URI base
    pub redirect_base_url: String,
    /// State token expiry in seconds
    pub state_expiry_seconds: u64,
    /// Providers configuration
    pub providers: HashMap<String, OAuthProviderConfig>,
    /// Allow registration via OAuth
    pub allow_registration: bool,
    /// Auto-link accounts with same email
    pub auto_link_accounts: bool,
    /// Required scopes for each provider
    pub required_scopes: HashMap<String, Vec<String>>,
}

impl Default for OAuthConfig {
    fn default() -> Self {
        let mut providers = HashMap::new();

        // Default provider configurations
        providers.insert("google".to_string(), OAuthProviderConfig {
            name: "Google".to_string(),
            client_id: String::new(),
            client_secret: String::new(),
            auth_url: "https://accounts.google.com/o/oauth2/v2/auth".to_string(),
            token_url: "https://oauth2.googleapis.com/token".to_string(),
            user_info_url: "https://www.googleapis.com/oauth2/v3/userinfo".to_string(),
            scopes: vec!["openid".to_string(), "email".to_string(), "profile".to_string()],
            enabled: false,
        });

        providers.insert("github".to_string(), OAuthProviderConfig {
            name: "GitHub".to_string(),
            client_id: String::new(),
            client_secret: String::new(),
            auth_url: "https://github.com/login/oauth/authorize".to_string(),
            token_url: "https://github.com/login/oauth/access_token".to_string(),
            user_info_url: "https://api.github.com/user".to_string(),
            scopes: vec!["read:user".to_string(), "user:email".to_string()],
            enabled: false,
        });

        providers.insert("discord".to_string(), OAuthProviderConfig {
            name: "Discord".to_string(),
            client_id: String::new(),
            client_secret: String::new(),
            auth_url: "https://discord.com/api/oauth2/authorize".to_string(),
            token_url: "https://discord.com/api/oauth2/token".to_string(),
            user_info_url: "https://discord.com/api/users/@me".to_string(),
            scopes: vec!["identify".to_string(), "email".to_string()],
            enabled: false,
        });

        Self {
            enabled: true,
            redirect_base_url: "http://localhost:3000/auth/callback".to_string(),
            state_expiry_seconds: 600,
            providers,
            allow_registration: true,
            auto_link_accounts: true,
            required_scopes: HashMap::new(),
        }
    }
}

/// OAuth provider configuration
#[derive(Debug, Clone)]
pub struct OAuthProviderConfig {
    pub name: String,
    pub client_id: String,
    pub client_secret: String,
    pub auth_url: String,
    pub token_url: String,
    pub user_info_url: String,
    pub scopes: Vec<String>,
    pub enabled: bool,
}

/// OAuth provider trait
#[async_trait]
pub trait OAuthProvider: Send + Sync {
    /// Get provider name
    fn name(&self) -> &str;

    /// Generate authorization URL
    async fn get_authorization_url(&self, state: &str, nonce: Option<&str>) -> Result<String>;

    /// Exchange authorization code for tokens
    async fn exchange_code(&self, code: &str, state: &str) -> Result<OAuthTokens>;

    /// Get user information
    async fn get_user_info(&self, access_token: &str) -> Result<OAuthUserInfo>;

    /// Refresh access token
    async fn refresh_token(&self, refresh_token: &str) -> Result<OAuthTokens>;

    /// Revoke token
    async fn revoke_token(&self, token: &str) -> Result<()>;
}

/// OAuth tokens
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OAuthTokens {
    pub access_token: String,
    pub token_type: String,
    pub expires_in: Option<u64>,
    pub refresh_token: Option<String>,
    pub id_token: Option<String>,
    pub scope: Option<String>,
}

/// OAuth user information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OAuthUserInfo {
    pub provider: String,
    pub provider_user_id: String,
    pub email: Option<String>,
    pub email_verified: Option<bool>,
    pub name: Option<String>,
    pub given_name: Option<String>,
    pub family_name: Option<String>,
    pub picture: Option<String>,
    pub locale: Option<String>,
    pub raw_data: Value,
}

/// OAuth state tracking
#[derive(Debug, Clone)]
struct OAuthState {
    state: String,
    provider: String,
    code_verifier: Option<String>,
    nonce: Option<String>,
    redirect_uri: String,
    created_at: DateTime<Utc>,
    expires_at: DateTime<Utc>,
    user_id: Option<Uuid>,
}

/// OAuth account link
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OAuthAccount {
    pub user_id: Uuid,
    pub provider: String,
    pub provider_user_id: String,
    pub email: Option<String>,
    pub display_name: Option<String>,
    pub avatar_url: Option<String>,
    pub access_token: String,
    pub refresh_token: Option<String>,
    pub token_expires_at: Option<DateTime<Utc>>,
    pub linked_at: DateTime<Utc>,
    pub last_used: DateTime<Utc>,
}

/// OAuth manager
pub struct OAuthManager {
    config: OAuthConfig,
    http_client: Client,
    providers: Arc<RwLock<HashMap<String, Box<dyn OAuthProvider>>>>,
    states: Arc<RwLock<HashMap<String, OAuthState>>>,
    linked_accounts: Arc<RwLock<HashMap<Uuid, Vec<OAuthAccount>>>>,
}

impl OAuthManager {
    /// Create new OAuth manager
    pub fn new(config: OAuthConfig) -> Result<Self> {
        let http_client = Client::builder()
            .timeout(std::time::Duration::from_secs(30))
            .build()?;

        let manager = Self {
            config: config.clone(),
            http_client,
            providers: Arc::new(RwLock::new(HashMap::new())),
            states: Arc::new(RwLock::new(HashMap::new())),
            linked_accounts: Arc::new(RwLock::new(HashMap::new())),
        };

        // Initialize providers
        manager.initialize_providers(config);

        // Start cleanup task
        manager.start_cleanup_task();

        Ok(manager)
    }

    /// Initialize OAuth providers
    fn initialize_providers(&self, config: OAuthConfig) {
        let providers = self.providers.clone();
        let http_client = self.http_client.clone();

        tokio::spawn(async move {
            let mut providers_map = providers.write().await;

            for (key, provider_config) in config.providers {
                if !provider_config.enabled {
                    continue;
                }

                let provider: Box<dyn OAuthProvider> = match key.as_str() {
                    "google" => Box::new(GoogleOAuthProvider::new(
                        provider_config,
                        http_client.clone(),
                        &config.redirect_base_url,
                    )),
                    "github" => Box::new(GitHubOAuthProvider::new(
                        provider_config,
                        http_client.clone(),
                        &config.redirect_base_url,
                    )),
                    "discord" => Box::new(DiscordOAuthProvider::new(
                        provider_config,
                        http_client.clone(),
                        &config.redirect_base_url,
                    )),
                    _ => {
                        warn!("Unknown OAuth provider: {}", key);
                        continue;
                    }
                };

                providers_map.insert(key, provider);
            }

            info!("Initialized {} OAuth providers", providers_map.len());
        });
    }

    /// Start OAuth login flow
    pub async fn start_login_flow(
        &self,
        provider_name: &str,
        user_id: Option<Uuid>,
    ) -> Result<String> {
        let providers = self.providers.read().await;
        let provider = providers.get(provider_name)
            .ok_or_else(|| anyhow!("Provider not found or not enabled"))?;

        // Generate state and PKCE parameters
        let state = self.generate_state();
        let code_verifier = self.generate_code_verifier();
        let nonce = self.generate_nonce();

        // Get authorization URL
        let auth_url = provider.get_authorization_url(&state, Some(&nonce)).await?;

        // Store state
        let oauth_state = OAuthState {
            state: state.clone(),
            provider: provider_name.to_string(),
            code_verifier: Some(code_verifier),
            nonce: Some(nonce),
            redirect_uri: format!("{}/{}", self.config.redirect_base_url, provider_name),
            created_at: Utc::now(),
            expires_at: Utc::now() + Duration::seconds(self.config.state_expiry_seconds as i64),
            user_id,
        };

        let mut states = self.states.write().await;
        states.insert(state.clone(), oauth_state);

        info!("Started OAuth flow for provider {}", provider_name);
        Ok(auth_url)
    }

    /// Handle OAuth callback
    pub async fn handle_callback(
        &self,
        provider_name: &str,
        code: &str,
        state: &str,
    ) -> Result<OAuthCallbackResult> {
        // Validate state
        let oauth_state = {
            let mut states = self.states.write().await;
            states.remove(state)
                .ok_or_else(|| anyhow!("Invalid or expired state"))?
        };

        if oauth_state.provider != provider_name {
            return Err(anyhow!("Provider mismatch"));
        }

        if oauth_state.expires_at < Utc::now() {
            return Err(anyhow!("State expired"));
        }

        // Exchange code for tokens
        let providers = self.providers.read().await;
        let provider = providers.get(provider_name)
            .ok_or_else(|| anyhow!("Provider not found"))?;

        let tokens = provider.exchange_code(code, state).await?;

        // Get user information
        let user_info = provider.get_user_info(&tokens.access_token).await?;

        // Check if account exists
        let existing_account = self.find_linked_account(
            &user_info.provider,
            &user_info.provider_user_id,
        ).await;

        if let Some(account) = existing_account {
            // Update tokens
            self.update_account_tokens(
                &account.user_id,
                &user_info.provider,
                tokens,
            ).await?;

            return Ok(OAuthCallbackResult::ExistingUser {
                user_id: account.user_id,
                provider: user_info.provider,
            });
        }

        // Check if email exists (for auto-linking)
        if self.config.auto_link_accounts {
            if let Some(email) = &user_info.email {
                if let Some(user_id) = self.find_user_by_email(email).await {
                    // Link account
                    self.link_account(user_id, user_info.clone(), tokens).await?;

                    return Ok(OAuthCallbackResult::AccountLinked {
                        user_id,
                        provider: user_info.provider,
                    });
                }
            }
        }

        // New registration
        if !self.config.allow_registration {
            return Err(anyhow!("Registration via OAuth is not allowed"));
        }

        Ok(OAuthCallbackResult::NewUser {
            user_info,
            tokens,
        })
    }

    /// Link OAuth account to user
    pub async fn link_account(
        &self,
        user_id: Uuid,
        user_info: OAuthUserInfo,
        tokens: OAuthTokens,
    ) -> Result<()> {
        let account = OAuthAccount {
            user_id,
            provider: user_info.provider.clone(),
            provider_user_id: user_info.provider_user_id,
            email: user_info.email,
            display_name: user_info.name,
            avatar_url: user_info.picture,
            access_token: tokens.access_token,
            refresh_token: tokens.refresh_token,
            token_expires_at: tokens.expires_in.map(|exp| {
                Utc::now() + Duration::seconds(exp as i64)
            }),
            linked_at: Utc::now(),
            last_used: Utc::now(),
        };

        let mut linked_accounts = self.linked_accounts.write().await;
        linked_accounts
            .entry(user_id)
            .or_insert_with(Vec::new)
            .push(account);

        info!("Linked {} account for user {}", user_info.provider, user_id);
        Ok(())
    }

    /// Unlink OAuth account
    pub async fn unlink_account(
        &self,
        user_id: &Uuid,
        provider: &str,
    ) -> Result<()> {
        let mut linked_accounts = self.linked_accounts.write().await;
        if let Some(accounts) = linked_accounts.get_mut(user_id) {
            accounts.retain(|a| a.provider != provider);

            // Revoke tokens if possible
            if let Some(account) = accounts.iter().find(|a| a.provider == provider) {
                let providers = self.providers.read().await;
                if let Some(oauth_provider) = providers.get(provider) {
                    let _ = oauth_provider.revoke_token(&account.access_token).await;
                }
            }

            info!("Unlinked {} account for user {}", provider, user_id);
        }

        Ok(())
    }

    /// Get linked accounts for user
    pub async fn get_linked_accounts(&self, user_id: &Uuid) -> Vec<OAuthAccount> {
        let linked_accounts = self.linked_accounts.read().await;
        linked_accounts.get(user_id).cloned().unwrap_or_default()
    }

    /// Find linked account
    async fn find_linked_account(
        &self,
        provider: &str,
        provider_user_id: &str,
    ) -> Option<OAuthAccount> {
        let linked_accounts = self.linked_accounts.read().await;
        for accounts in linked_accounts.values() {
            for account in accounts {
                if account.provider == provider && account.provider_user_id == provider_user_id {
                    return Some(account.clone());
                }
            }
        }
        None
    }

    /// Find user by email (placeholder - should query database)
    async fn find_user_by_email(&self, _email: &str) -> Option<Uuid> {
        // In production, query database for user with this email
        None
    }

    /// Update account tokens
    async fn update_account_tokens(
        &self,
        user_id: &Uuid,
        provider: &str,
        tokens: OAuthTokens,
    ) -> Result<()> {
        let mut linked_accounts = self.linked_accounts.write().await;
        if let Some(accounts) = linked_accounts.get_mut(user_id) {
            for account in accounts {
                if account.provider == provider {
                    account.access_token = tokens.access_token;
                    account.refresh_token = tokens.refresh_token.or(account.refresh_token.clone());
                    account.token_expires_at = tokens.expires_in.map(|exp| {
                        Utc::now() + Duration::seconds(exp as i64)
                    });
                    account.last_used = Utc::now();
                    break;
                }
            }
        }
        Ok(())
    }

    /// Generate random state
    fn generate_state(&self) -> String {
        let mut rng = thread_rng();
        let state: String = (0..32)
            .map(|_| {
                let idx = rng.gen_range(0..62);
                "ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789"
                    .chars()
                    .nth(idx)
                    .unwrap()
            })
            .collect();
        state
    }

    /// Generate PKCE code verifier
    fn generate_code_verifier(&self) -> String {
        let mut rng = thread_rng();
        let verifier: String = (0..128)
            .map(|_| {
                let idx = rng.gen_range(0..66);
                "ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789-._~"
                    .chars()
                    .nth(idx)
                    .unwrap()
            })
            .collect();
        verifier
    }

    /// Generate PKCE code challenge
    fn generate_code_challenge(&self, verifier: &str) -> String {
        let mut hasher = Sha256::new();
        hasher.update(verifier.as_bytes());
        let result = hasher.finalize();
        URL_SAFE_NO_PAD.encode(result)
    }

    /// Generate nonce for OpenID Connect
    fn generate_nonce(&self) -> String {
        self.generate_state() // Same as state generation
    }

    /// Start cleanup task
    fn start_cleanup_task(&self) {
        let states = self.states.clone();

        tokio::spawn(async move {
            let mut interval = tokio::time::interval(std::time::Duration::from_secs(300));

            loop {
                interval.tick().await;

                let mut states_guard = states.write().await;
                let now = Utc::now();
                states_guard.retain(|_, state| state.expires_at > now);

                debug!("OAuth state cleanup completed");
            }
        });
    }
}

/// OAuth callback result
#[derive(Debug, Clone)]
pub enum OAuthCallbackResult {
    ExistingUser {
        user_id: Uuid,
        provider: String,
    },
    AccountLinked {
        user_id: Uuid,
        provider: String,
    },
    NewUser {
        user_info: OAuthUserInfo,
        tokens: OAuthTokens,
    },
}

/// Google OAuth provider implementation
struct GoogleOAuthProvider {
    config: OAuthProviderConfig,
    http_client: Client,
    redirect_uri: String,
}

impl GoogleOAuthProvider {
    fn new(config: OAuthProviderConfig, http_client: Client, redirect_base: &str) -> Self {
        Self {
            config,
            http_client,
            redirect_uri: format!("{}/google", redirect_base),
        }
    }
}

#[async_trait]
impl OAuthProvider for GoogleOAuthProvider {
    fn name(&self) -> &str {
        "google"
    }

    async fn get_authorization_url(&self, state: &str, nonce: Option<&str>) -> Result<String> {
        let mut params = vec![
            ("client_id", self.config.client_id.as_str()),
            ("redirect_uri", self.redirect_uri.as_str()),
            ("response_type", "code"),
            ("scope", &self.config.scopes.join(" ")),
            ("state", state),
            ("access_type", "offline"),
            ("prompt", "consent"),
        ];

        if let Some(n) = nonce {
            params.push(("nonce", n));
        }

        let url = format!("{}?{}", self.config.auth_url, serde_urlencoded::to_string(params)?);
        Ok(url)
    }

    async fn exchange_code(&self, code: &str, _state: &str) -> Result<OAuthTokens> {
        let params = [
            ("code", code),
            ("client_id", &self.config.client_id),
            ("client_secret", &self.config.client_secret),
            ("redirect_uri", &self.redirect_uri),
            ("grant_type", "authorization_code"),
        ];

        let response = self.http_client
            .post(&self.config.token_url)
            .form(&params)
            .send()
            .await?;

        if !response.status().is_success() {
            let error = response.text().await?;
            return Err(anyhow!("Token exchange failed: {}", error));
        }

        let tokens: OAuthTokens = response.json().await?;
        Ok(tokens)
    }

    async fn get_user_info(&self, access_token: &str) -> Result<OAuthUserInfo> {
        let response = self.http_client
            .get(&self.config.user_info_url)
            .bearer_auth(access_token)
            .send()
            .await?;

        if !response.status().is_success() {
            return Err(anyhow!("Failed to get user info"));
        }

        let raw_data: Value = response.json().await?;

        Ok(OAuthUserInfo {
            provider: "google".to_string(),
            provider_user_id: raw_data["sub"].as_str().unwrap_or_default().to_string(),
            email: raw_data["email"].as_str().map(String::from),
            email_verified: raw_data["email_verified"].as_bool(),
            name: raw_data["name"].as_str().map(String::from),
            given_name: raw_data["given_name"].as_str().map(String::from),
            family_name: raw_data["family_name"].as_str().map(String::from),
            picture: raw_data["picture"].as_str().map(String::from),
            locale: raw_data["locale"].as_str().map(String::from),
            raw_data,
        })
    }

    async fn refresh_token(&self, refresh_token: &str) -> Result<OAuthTokens> {
        let params = [
            ("refresh_token", refresh_token),
            ("client_id", &self.config.client_id),
            ("client_secret", &self.config.client_secret),
            ("grant_type", "refresh_token"),
        ];

        let response = self.http_client
            .post(&self.config.token_url)
            .form(&params)
            .send()
            .await?;

        let tokens: OAuthTokens = response.json().await?;
        Ok(tokens)
    }

    async fn revoke_token(&self, token: &str) -> Result<()> {
        let url = "https://oauth2.googleapis.com/revoke";
        let params = [("token", token)];

        self.http_client
            .post(url)
            .form(&params)
            .send()
            .await?;

        Ok(())
    }
}

/// GitHub OAuth provider implementation
struct GitHubOAuthProvider {
    config: OAuthProviderConfig,
    http_client: Client,
    redirect_uri: String,
}

impl GitHubOAuthProvider {
    fn new(config: OAuthProviderConfig, http_client: Client, redirect_base: &str) -> Self {
        Self {
            config,
            http_client,
            redirect_uri: format!("{}/github", redirect_base),
        }
    }
}

#[async_trait]
impl OAuthProvider for GitHubOAuthProvider {
    fn name(&self) -> &str {
        "github"
    }

    async fn get_authorization_url(&self, state: &str, _nonce: Option<&str>) -> Result<String> {
        let params = [
            ("client_id", self.config.client_id.as_str()),
            ("redirect_uri", self.redirect_uri.as_str()),
            ("scope", &self.config.scopes.join(" ")),
            ("state", state),
        ];

        let url = format!("{}?{}", self.config.auth_url, serde_urlencoded::to_string(params)?);
        Ok(url)
    }

    async fn exchange_code(&self, code: &str, _state: &str) -> Result<OAuthTokens> {
        let params = [
            ("code", code),
            ("client_id", &self.config.client_id),
            ("client_secret", &self.config.client_secret),
            ("redirect_uri", &self.redirect_uri),
        ];

        let response = self.http_client
            .post(&self.config.token_url)
            .header("Accept", "application/json")
            .form(&params)
            .send()
            .await?;

        let tokens: OAuthTokens = response.json().await?;
        Ok(tokens)
    }

    async fn get_user_info(&self, access_token: &str) -> Result<OAuthUserInfo> {
        let response = self.http_client
            .get(&self.config.user_info_url)
            .header("User-Agent", "HackerExperience")
            .bearer_auth(access_token)
            .send()
            .await?;

        let raw_data: Value = response.json().await?;

        // Get email separately if needed
        let email = if raw_data["email"].is_null() {
            self.get_github_email(access_token).await.ok()
        } else {
            raw_data["email"].as_str().map(String::from)
        };

        Ok(OAuthUserInfo {
            provider: "github".to_string(),
            provider_user_id: raw_data["id"].to_string(),
            email,
            email_verified: Some(true),
            name: raw_data["name"].as_str().map(String::from),
            given_name: None,
            family_name: None,
            picture: raw_data["avatar_url"].as_str().map(String::from),
            locale: None,
            raw_data,
        })
    }

    async fn refresh_token(&self, _refresh_token: &str) -> Result<OAuthTokens> {
        Err(anyhow!("GitHub does not support refresh tokens"))
    }

    async fn revoke_token(&self, _token: &str) -> Result<()> {
        // GitHub tokens can be revoked through settings
        Ok(())
    }
}

impl GitHubOAuthProvider {
    async fn get_github_email(&self, access_token: &str) -> Result<String> {
        let response = self.http_client
            .get("https://api.github.com/user/emails")
            .header("User-Agent", "HackerExperience")
            .bearer_auth(access_token)
            .send()
            .await?;

        let emails: Vec<Value> = response.json().await?;

        // Find primary email
        for email in emails {
            if email["primary"].as_bool().unwrap_or(false) {
                if let Some(email_str) = email["email"].as_str() {
                    return Ok(email_str.to_string());
                }
            }
        }

        Err(anyhow!("No primary email found"))
    }
}

/// Discord OAuth provider implementation
struct DiscordOAuthProvider {
    config: OAuthProviderConfig,
    http_client: Client,
    redirect_uri: String,
}

impl DiscordOAuthProvider {
    fn new(config: OAuthProviderConfig, http_client: Client, redirect_base: &str) -> Self {
        Self {
            config,
            http_client,
            redirect_uri: format!("{}/discord", redirect_base),
        }
    }
}

#[async_trait]
impl OAuthProvider for DiscordOAuthProvider {
    fn name(&self) -> &str {
        "discord"
    }

    async fn get_authorization_url(&self, state: &str, _nonce: Option<&str>) -> Result<String> {
        let params = [
            ("client_id", self.config.client_id.as_str()),
            ("redirect_uri", self.redirect_uri.as_str()),
            ("response_type", "code"),
            ("scope", &self.config.scopes.join(" ")),
            ("state", state),
        ];

        let url = format!("{}?{}", self.config.auth_url, serde_urlencoded::to_string(params)?);
        Ok(url)
    }

    async fn exchange_code(&self, code: &str, _state: &str) -> Result<OAuthTokens> {
        let params = [
            ("code", code.to_string()),
            ("client_id", self.config.client_id.clone()),
            ("client_secret", self.config.client_secret.clone()),
            ("redirect_uri", self.redirect_uri.clone()),
            ("grant_type", "authorization_code".to_string()),
        ];

        let response = self.http_client
            .post(&self.config.token_url)
            .form(&params)
            .send()
            .await?;

        let tokens: OAuthTokens = response.json().await?;
        Ok(tokens)
    }

    async fn get_user_info(&self, access_token: &str) -> Result<OAuthUserInfo> {
        let response = self.http_client
            .get(&self.config.user_info_url)
            .bearer_auth(access_token)
            .send()
            .await?;

        let raw_data: Value = response.json().await?;

        Ok(OAuthUserInfo {
            provider: "discord".to_string(),
            provider_user_id: raw_data["id"].as_str().unwrap_or_default().to_string(),
            email: raw_data["email"].as_str().map(String::from),
            email_verified: raw_data["verified"].as_bool(),
            name: raw_data["username"].as_str().map(String::from),
            given_name: None,
            family_name: None,
            picture: raw_data["avatar"].as_str().map(|avatar| {
                format!("https://cdn.discordapp.com/avatars/{}/{}.png",
                    raw_data["id"].as_str().unwrap_or_default(),
                    avatar
                )
            }),
            locale: raw_data["locale"].as_str().map(String::from),
            raw_data,
        })
    }

    async fn refresh_token(&self, refresh_token: &str) -> Result<OAuthTokens> {
        let params = [
            ("refresh_token", refresh_token),
            ("client_id", &self.config.client_id),
            ("client_secret", &self.config.client_secret),
            ("grant_type", "refresh_token"),
        ];

        let response = self.http_client
            .post(&self.config.token_url)
            .form(&params)
            .send()
            .await?;

        let tokens: OAuthTokens = response.json().await?;
        Ok(tokens)
    }

    async fn revoke_token(&self, token: &str) -> Result<()> {
        let url = "https://discord.com/api/oauth2/token/revoke";
        let params = [
            ("token", token),
            ("client_id", &self.config.client_id),
            ("client_secret", &self.config.client_secret),
        ];

        self.http_client
            .post(url)
            .form(&params)
            .send()
            .await?;

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_oauth_manager_creation() {
        let config = OAuthConfig::default();
        let manager = OAuthManager::new(config);
        assert!(manager.is_ok());
    }

    #[test]
    fn test_state_generation() {
        let config = OAuthConfig::default();
        let manager = OAuthManager::new(config).unwrap();

        let state1 = manager.generate_state();
        let state2 = manager.generate_state();

        assert_eq!(state1.len(), 32);
        assert_eq!(state2.len(), 32);
        assert_ne!(state1, state2);
    }

    #[test]
    fn test_code_verifier_generation() {
        let config = OAuthConfig::default();
        let manager = OAuthManager::new(config).unwrap();

        let verifier = manager.generate_code_verifier();
        assert_eq!(verifier.len(), 128);

        let challenge = manager.generate_code_challenge(&verifier);
        assert!(!challenge.is_empty());
    }
}