// DATABASE.CLASS.PHP PORT - Core system database operations
// Original: LRSys class with user registration and login functionality
// Includes spam protection, Facebook/Twitter integration, and forum sync

use std::collections::HashMap;
use chrono::{DateTime, Utc};
use sqlx::MySqlPool;
use he_core::*;
use he_db::*;

pub struct LRSys {
    pub name: Option<String>,
    pub user: Option<String>,
    pass: Option<String>,
    pub email: Option<String>,
    pub keepalive: bool,
    
    db_pool: MySqlPool,
    // TODO: Add these services when they're implemented
    // session: Session,
    // log: LogVPC,
    // ranking: Ranking,
    // storyline: Storyline,
    // clan: Clan,
}

impl LRSys {
    pub fn new(db_pool: MySqlPool) -> Self {
        Self {
            name: None,
            user: None,
            pass: None,
            email: None,
            keepalive: false,
            db_pool,
        }
    }
    
    pub fn set_keepalive(&mut self, keep: bool) {
        self.keepalive = keep;
    }
    
    // Original PHP: register function with spam protection and validation
    pub async fn register(&mut self, reg_user: String, reg_pass: String, reg_mail: String, client_ip: String) -> Result<bool, DatabaseError> {
        self.user = Some(reg_user.clone());
        self.pass = Some(reg_pass.clone());
        self.email = Some(reg_mail.clone());
        
        // Spam protection - limit 1 registration per IP per 10 minutes
        let spam_check = sqlx::query_scalar::<_, i64>(
            "SELECT COUNT(*) FROM stats_register WHERE ip = ? AND TIMESTAMPDIFF(MINUTE, registrationDate, NOW()) < 10"
        )
        .bind(&client_ip)
        .fetch_one(&self.db_pool)
        .await
        .map_err(DatabaseError::SqlxError)?;
        
        if spam_check >= 1 {
            return Err(DatabaseError::ValidationError("IP blocked for multiple registrations. Try again in 10 minutes.".to_string()));
        }
        
        if self.verify_register(&client_ip).await? {
            // Hash password using BCrypt equivalent
            let hash = bcrypt::hash(&reg_pass, bcrypt::DEFAULT_COST)
                .map_err(|e| DatabaseError::ValidationError(format!("Password hashing failed: {}", e)))?;
                
            // Generate random game IP
            let game_ip1 = rand::random::<u8>();
            let game_ip2 = rand::random::<u8>();
            let game_ip3 = rand::random::<u8>();
            let game_ip4 = rand::random::<u8>();
            let game_ip = format!("{}.{}.{}.{}", game_ip1, game_ip2, game_ip3, game_ip4);
            
            // TODO: Create user via Python script equivalent
            // python.createUser(self.user, hash, self.email, gameIP);
            
            // Create user in database - this should be done by the Python equivalent
            let user_id = sqlx::query_scalar::<_, i64>(
                "INSERT INTO users (login, password, email, game_ip) VALUES (?, ?, ?, INET_ATON(?)) RETURNING id"
            )
            .bind(&reg_user)
            .bind(&hash)
            .bind(&reg_mail)
            .bind(&game_ip)
            .fetch_optional(&self.db_pool)
            .await
            .map_err(DatabaseError::SqlxError)?;
            
            let user_id = match user_id {
                Some(id) => id,
                None => {
                    return Err(DatabaseError::ValidationError("Error while completing registration. Please, try again later.".to_string()));
                }
            };
            
            // TODO: Send verification email
            // let email_verification = EmailVerification::new();
            // email_verification.send_mail(user_id, &reg_mail, &reg_user).await?;
            
            // TODO: Create financial account
            // let finances = Finances::new();
            // finances.create_account(user_id).await?;
            
            // TODO: Register with forum
            // let forum = Forum::new();
            // forum.external_register(&reg_user, &reg_pass, &reg_mail, user_id).await?;
            
            // Track registration statistics
            sqlx::query("INSERT INTO stats_register (userID, ip) VALUES (?, ?)")
                .bind(user_id)
                .bind(&client_ip)
                .execute(&self.db_pool)
                .await
                .map_err(DatabaseError::SqlxError)?;
                
            // TODO: Add success message to session
            // self.session.add_msg("Registration complete. You can login now.", "notice");
            
            Ok(true)
        } else {
            Ok(false)
        }
    }
    
    // Original PHP: verifyRegister with spam detection and validation
    async fn verify_register(&self, client_ip: &str) -> Result<bool, DatabaseError> {
        let username = self.user.as_ref().ok_or_else(|| DatabaseError::ValidationError("Username not set".to_string()))?;
        let email = self.email.as_ref().ok_or_else(|| DatabaseError::ValidationError("Email not set".to_string()))?;
        
        // Username validation - allow azAZ09._-
        if !Self::validate_username(username) {
            return Err(DatabaseError::ValidationError(format!("Invalid username. Allowed characters are azAZ09._-")));
        }
        
        // Email validation
        if !Self::validate_email(email) {
            return Err(DatabaseError::ValidationError(format!("The email {} is not valid.", email)));
        }
        
        // Spam detection - original PHP anti-spam logic preserved
        let uppercase_count = email.chars().filter(|c| c.is_ascii_uppercase()).count();
        let number_count = email.chars().filter(|c| c.is_ascii_digit()).count();
        
        // Block emails with too many uppercase letters and numbers (spam pattern)
        if (uppercase_count >= 5 && number_count >= 2) || number_count >= 5 {
            return Err(DatabaseError::ValidationError("Registration complete. You can login now.".to_string())); // Fake success message for spam
        }
        
        // Block short emails with uppercase letters (another spam pattern)
        if uppercase_count >= 2 && email.len() <= 12 {
            return Err(DatabaseError::ValidationError("Registration complete. You can login now.".to_string())); // Fake success message for spam
        }
        
        // Check if username or email already exists
        let existing_count = sqlx::query_scalar::<_, i64>(
            "SELECT COUNT(*) FROM users WHERE login = ? OR email = ? LIMIT 1"
        )
        .bind(username)
        .bind(email)
        .fetch_one(&self.db_pool)
        .await
        .map_err(DatabaseError::SqlxError)?;
        
        if existing_count > 0 {
            // Check which field is duplicate
            let existing_email = sqlx::query_scalar::<_, Option<String>>(
                "SELECT email FROM users WHERE login = ? OR email = ? LIMIT 1"
            )
            .bind(username)
            .bind(email)
            .fetch_optional(&self.db_pool)
            .await
            .map_err(DatabaseError::SqlxError)?;
            
            if let Some(existing) = existing_email {
                if existing == *email {
                    return Err(DatabaseError::ValidationError("This email is already used.".to_string()));
                } else {
                    return Err(DatabaseError::ValidationError("This username is already taken.".to_string()));
                }
            }
        }
        
        // Check for empty fields
        if username.is_empty() || self.pass.as_ref().map_or(true, |p| p.is_empty()) || email.is_empty() {
            return Err(DatabaseError::ValidationError("Some fields are empty.".to_string()));
        }
        
        // Username length check
        if username.len() > 15 {
            return Err(DatabaseError::ValidationError("Your username is too big :( Please, limit it to 15 characters.".to_string()));
        }
        
        Ok(true)
    }
    
    // Original PHP: login function with Facebook/Twitter/Remember Me support
    pub async fn login(&mut self, log_user: String, log_pass: String, special: Option<&str>) -> Result<bool, DatabaseError> {
        let mut facebook = false;
        let mut twitter = false;
        let mut remember = false;
        
        if let Some(special_type) = special {
            match special_type {
                "remember" => remember = true,
                "facebook" => facebook = true,
                "twitter" => {
                    twitter = true;
                    // TODO: unset Twitter session data
                },
                _ => return Err(DatabaseError::ValidationError("Invalid special login type".to_string())),
            }
        }
        
        self.user = Some(log_user.clone());
        self.pass = Some(log_pass.clone());
        
        if self.verify_login(facebook, remember, twitter)? {
            // Get user password and ID from database
            let user_data = sqlx::query_as::<_, (String, i64)>(
                "SELECT password, id FROM users WHERE BINARY login = ? LIMIT 1"
            )
            .bind(&log_user)
            .fetch_optional(&self.db_pool)
            .await
            .map_err(DatabaseError::SqlxError)?;
            
            if let Some((stored_password, user_id)) = user_data {
                // Verify password or allow social/remember login
                let password_valid = if facebook || remember || twitter {
                    true
                } else {
                    bcrypt::verify(&log_pass, &stored_password).unwrap_or(false)
                };
                
                if password_valid {
                    // Check for conflicting social logins (original security check)
                    if !facebook && !twitter {
                        let fb_count = sqlx::query_scalar::<_, i64>(
                            "SELECT COUNT(*) FROM users_facebook WHERE gameID = ? LIMIT 1"
                        )
                        .bind(user_id)
                        .fetch_one(&self.db_pool)
                        .await
                        .map_err(DatabaseError::SqlxError)?;
                        
                        if fb_count > 0 {
                            return Err(DatabaseError::ValidationError("Facebook fail".to_string()));
                        }
                        
                        let twitter_count = sqlx::query_scalar::<_, i64>(
                            "SELECT COUNT(*) FROM users_twitter WHERE gameID = ? LIMIT 1"
                        )
                        .bind(user_id)
                        .fetch_one(&self.db_pool)
                        .await
                        .map_err(DatabaseError::SqlxError)?;
                        
                        if twitter_count > 0 {
                            return Err(DatabaseError::ValidationError("Twitter fail".to_string()));
                        }
                    }
                    
                    // Check premium status
                    let premium_count = sqlx::query_scalar::<_, i64>(
                        "SELECT COUNT(*) FROM users_premium WHERE id = ? LIMIT 1"
                    )
                    .bind(user_id)
                    .fetch_one(&self.db_pool)
                    .await
                    .map_err(DatabaseError::SqlxError)?;
                    
                    let premium = premium_count > 0;
                    
                    // TODO: Login to forum
                    // let forum = Forum::new();
                    // forum.login(&log_user, &log_pass, true).await?;
                    
                    // TODO: Setup session
                    // self.session.login_session(user_id, &log_user, premium, special);
                    
                    // Update login database
                    self.login_database(user_id).await?;
                    
                    // TODO: Setup other game systems
                    // let certs_array = ranking.cert_get_all();
                    // mission.restore_mission_session(user_id);
                    // session.cert_session(certs_array);
                    
                    // TODO: Setup clan session
                    // let clan_id = clan.get_player_clan(user_id);
                    // SESSION['CLAN_ID'] = clan_id;
                    
                    // TODO: Setup game state
                    // SESSION['LAST_CHECK'] = DateTime::now();
                    // SESSION['ROUND_STATUS'] = storyline.round_status();
                    
                    // TODO: Add login log if round is active
                    // if SESSION['ROUND_STATUS'] == 1 {
                    //     log.add_log(user_id, log.log_text('LOGIN', vec![0]), '0');
                    //     session.exp_add('LOGIN');
                    // }
                    
                    Ok(true)
                } else {
                    Err(DatabaseError::ValidationError("Username and password doesn't match. Some accounts were lost, sorry!".to_string()))
                }
            } else {
                Err(DatabaseError::ValidationError("Username and password doesn't match. Some accounts were lost, sorry!".to_string()))
            }
        } else {
            Ok(false)
        }
    }
    
    // Original PHP: loginDatabase - handle online status and remember me
    async fn login_database(&self, user_id: i64) -> Result<(), DatabaseError> {
        // Remove from online users if already present
        let online_count = sqlx::query_scalar::<_, i64>(
            "SELECT COUNT(*) FROM users_online WHERE id = ? LIMIT 1"
        )
        .bind(user_id)
        .fetch_one(&self.db_pool)
        .await
        .map_err(DatabaseError::SqlxError)?;
        
        if online_count > 0 {
            sqlx::query("DELETE FROM users_online WHERE id = ? LIMIT 1")
                .bind(user_id)
                .execute(&self.db_pool)
                .await
                .map_err(DatabaseError::SqlxError)?;
        }
        
        // TODO: Handle Remember Me functionality
        // let remember_me = RememberMe::new(key, &self.db_pool);
        // remember_me.remember(user_id, false, self.keepalive).await?;
        
        // Update last login timestamp
        sqlx::query("UPDATE users SET lastLogin = NOW() WHERE id = ?")
            .bind(user_id)
            .execute(&self.db_pool)
            .await
            .map_err(DatabaseError::SqlxError)?;
        
        // TODO: Set cookie equivalent
        // setcookie('logged', '1', time() + 172800);
        
        Ok(())
    }
    
    // Original PHP: verifyLogin - basic login field validation
    fn verify_login(&self, fb: bool, tt: bool, rm: bool) -> Result<bool, DatabaseError> {
        if fb || rm || tt {
            return Ok(true);
        }
        
        let username = self.user.as_ref().ok_or_else(|| DatabaseError::ValidationError("Username not set".to_string()))?;
        let password = self.pass.as_ref().ok_or_else(|| DatabaseError::ValidationError("Password not set".to_string()))?;
        
        if username.is_empty() || password.is_empty() {
            return Err(DatabaseError::ValidationError("Some fields are empty.".to_string()));
        }
        
        Ok(true)
    }
    
    // Username validation - allow azAZ09._-
    fn validate_username(username: &str) -> bool {
        username.chars().all(|c| c.is_ascii_alphanumeric() || c == '.' || c == '_' || c == '-')
    }
    
    // Basic email validation
    fn validate_email(email: &str) -> bool {
        email.contains('@') && email.contains('.') && email.len() >= 5
    }
}

#[derive(Debug)]
pub enum DatabaseError {
    SqlxError(sqlx::Error),
    ValidationError(String),
}

impl std::fmt::Display for DatabaseError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            DatabaseError::SqlxError(e) => write!(f, "Database error: {}", e),
            DatabaseError::ValidationError(msg) => write!(f, "Validation error: {}", msg),
        }
    }
}

impl std::error::Error for DatabaseError {}