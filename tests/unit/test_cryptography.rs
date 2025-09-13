use sha2::{Sha256, Digest};
use bcrypt::{hash, verify, DEFAULT_COST};
use aes_gcm::{
    aead::{Aead, AeadCore, KeyInit, OsRng},
    Aes256Gcm, Nonce, Key
};

/// Test password hashing and verification
#[test]
fn test_password_hashing() {
    let password = "test_password_123";
    
    // Test bcrypt hashing
    let hash_result = hash(password, DEFAULT_COST);
    assert!(hash_result.is_ok());
    
    let hashed = hash_result.unwrap();
    assert_ne!(hashed, password);
    assert!(hashed.starts_with("$2b$"));
    
    // Test password verification
    let verify_result = verify(password, &hashed);
    assert!(verify_result.is_ok());
    assert!(verify_result.unwrap());
    
    // Test wrong password
    let wrong_verify = verify("wrong_password", &hashed);
    assert!(wrong_verify.is_ok());
    assert!(!wrong_verify.unwrap());
}

#[test]
fn test_sha256_hashing() {
    let input = "Hello, World!";
    let expected = "dffd6021bb2bd5b0af676290809ec3a53191dd81c7f70a4b28688a362182986f";
    
    let mut hasher = Sha256::new();
    hasher.update(input.as_bytes());
    let result = hasher.finalize();
    let hex_result = format!("{:x}", result);
    
    assert_eq!(hex_result, expected);
}

#[test]
fn test_aes_encryption() {
    let key = Aes256Gcm::generate_key(OsRng);
    let cipher = Aes256Gcm::new(&key);
    let nonce = Aes256Gcm::generate_nonce(&mut OsRng);
    
    let plaintext = b"secret message";
    
    // Encrypt
    let ciphertext = cipher.encrypt(&nonce, plaintext.as_ref()).expect("encryption failure!");
    assert_ne!(ciphertext, plaintext);
    
    // Decrypt
    let decrypted = cipher.decrypt(&nonce, ciphertext.as_ref()).expect("decryption failure!");
    assert_eq!(decrypted, plaintext);
}

#[test]
fn test_session_token_generation() {
    use rand::{thread_rng, Rng};
    use rand::distributions::Alphanumeric;
    
    // Generate random session token
    let token: String = thread_rng()
        .sample_iter(&Alphanumeric)
        .take(32)
        .map(char::from)
        .collect();
    
    assert_eq!(token.len(), 32);
    assert!(token.chars().all(|c| c.is_alphanumeric()));
    
    // Generate another token to ensure they're different
    let token2: String = thread_rng()
        .sample_iter(&Alphanumeric)
        .take(32)
        .map(char::from)
        .collect();
    
    assert_ne!(token, token2);
}

#[test]
fn test_ip_address_validation() {
    use std::net::{IpAddr, Ipv4Addr, Ipv6Addr};
    
    // Test valid IPv4
    let valid_ipv4 = "192.168.1.1".parse::<IpAddr>();
    assert!(valid_ipv4.is_ok());
    assert!(valid_ipv4.unwrap().is_ipv4());
    
    // Test valid IPv6
    let valid_ipv6 = "2001:db8::1".parse::<IpAddr>();
    assert!(valid_ipv6.is_ok());
    assert!(valid_ipv6.unwrap().is_ipv6());
    
    // Test invalid IP
    let invalid_ip = "999.999.999.999".parse::<IpAddr>();
    assert!(invalid_ip.is_err());
    
    // Test private IP ranges
    let private_ip = "192.168.1.100".parse::<Ipv4Addr>().unwrap();
    assert!(private_ip.is_private());
    
    let public_ip = "8.8.8.8".parse::<Ipv4Addr>().unwrap();
    assert!(!public_ip.is_private());
}

#[test]
fn test_data_sanitization() {
    // Test SQL injection prevention
    fn sanitize_sql_input(input: &str) -> String {
        input.replace('\'', "''").replace(';', "")
    }
    
    let malicious_input = "'; DROP TABLE users; --";
    let sanitized = sanitize_sql_input(malicious_input);
    assert!(!sanitized.contains("DROP"));
    assert!(!sanitized.contains(';'));
    
    // Test XSS prevention
    fn sanitize_html_input(input: &str) -> String {
        input
            .replace('<', "&lt;")
            .replace('>', "&gt;")
            .replace('"', "&quot;")
            .replace('\'', "&#x27;")
    }
    
    let xss_input = "<script>alert('XSS')</script>";
    let sanitized_html = sanitize_html_input(xss_input);
    assert!(!sanitized_html.contains("<script>"));
    assert!(sanitized_html.contains("&lt;script&gt;"));
}

#[test]
fn test_rate_limiting() {
    use std::collections::HashMap;
    use std::time::{Duration, Instant};
    
    struct RateLimiter {
        requests: HashMap<String, Vec<Instant>>,
        max_requests: usize,
        window: Duration,
    }
    
    impl RateLimiter {
        fn new(max_requests: usize, window: Duration) -> Self {
            Self {
                requests: HashMap::new(),
                max_requests,
                window,
            }
        }
        
        fn is_allowed(&mut self, key: &str) -> bool {
            let now = Instant::now();
            let requests = self.requests.entry(key.to_string()).or_insert_with(Vec::new);
            
            // Remove old requests outside the window
            requests.retain(|&time| now.duration_since(time) < self.window);
            
            if requests.len() < self.max_requests {
                requests.push(now);
                true
            } else {
                false
            }
        }
    }
    
    let mut limiter = RateLimiter::new(3, Duration::from_secs(60));
    let client_ip = "192.168.1.1";
    
    // First 3 requests should be allowed
    assert!(limiter.is_allowed(client_ip));
    assert!(limiter.is_allowed(client_ip));
    assert!(limiter.is_allowed(client_ip));
    
    // 4th request should be blocked
    assert!(!limiter.is_allowed(client_ip));
    
    // Different client should still be allowed
    assert!(limiter.is_allowed("192.168.1.2"));
}

#[test]
fn test_json_web_token() {
    use jsonwebtoken::{encode, decode, Header, Algorithm, Validation, EncodingKey, DecodingKey};
    use serde::{Deserialize, Serialize};
    
    #[derive(Debug, Serialize, Deserialize)]
    struct Claims {
        sub: String,
        player_id: i64,
        exp: usize,
    }
    
    let secret = b"secret_key";
    let encoding_key = EncodingKey::from_secret(secret);
    let decoding_key = DecodingKey::from_secret(secret);
    
    let claims = Claims {
        sub: "test_user".to_string(),
        player_id: 123,
        exp: 10000000000, // Far future
    };
    
    // Encode JWT
    let token = encode(&Header::default(), &claims, &encoding_key);
    assert!(token.is_ok());
    let token_string = token.unwrap();
    
    // Decode JWT
    let validation = Validation::new(Algorithm::HS256);
    let token_data = decode::<Claims>(&token_string, &decoding_key, &validation);
    assert!(token_data.is_ok());
    
    let decoded_claims = token_data.unwrap().claims;
    assert_eq!(decoded_claims.sub, "test_user");
    assert_eq!(decoded_claims.player_id, 123);
}