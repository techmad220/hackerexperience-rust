# JWT Key Rotation Strategy

## Overview
This document outlines the JWT secret key rotation process for HackerExperience Rust to maintain security while ensuring zero downtime.

## Current Implementation
- JWT secrets are stored in environment variables (`JWT_SECRET`)
- Tokens have configurable expiration (default: 15 minutes access, 7 days refresh)
- JWT validation uses the `jsonwebtoken` crate with HS256 algorithm

## Key Rotation Process

### 1. Dual-Key Support Implementation
The system supports two JWT secrets simultaneously during rotation:
- **Primary Key**: Used for signing new tokens
- **Secondary Key**: Used for validating existing tokens only

### 2. Environment Variables
```bash
# Primary key (for signing and validation)
JWT_SECRET=your-new-secure-secret-key-here
# Secondary key (for validation only during rotation)
JWT_SECRET_SECONDARY=your-previous-secure-secret-key-here
```

### 3. Rotation Steps

#### Step 1: Generate New Secret
```bash
# Generate a secure 256-bit key
openssl rand -base64 32
```

#### Step 2: Update Configuration
1. Move current `JWT_SECRET` to `JWT_SECRET_SECONDARY`
2. Set new key as `JWT_SECRET`
3. Deploy with both keys active

#### Step 3: Grace Period
- Keep both keys active for at least 2x the refresh token lifetime (14 days)
- During this period:
  - New tokens are signed with the primary key
  - Old tokens are validated against both keys

#### Step 4: Complete Rotation
- After grace period, remove `JWT_SECRET_SECONDARY`
- All old tokens will have expired naturally

### 4. Implementation Code

```rust
// In crates/he-auth/src/jwt.rs
pub struct JwtManager {
    primary_key: DecodingKey,
    secondary_key: Option<DecodingKey>,
    encoding_key: EncodingKey,
}

impl JwtManager {
    pub fn new() -> Result<Self> {
        let primary_secret = env::var("JWT_SECRET")
            .expect("JWT_SECRET must be set");

        let secondary_secret = env::var("JWT_SECRET_SECONDARY").ok();

        Ok(Self {
            encoding_key: EncodingKey::from_secret(primary_secret.as_bytes()),
            primary_key: DecodingKey::from_secret(primary_secret.as_bytes()),
            secondary_key: secondary_secret.map(|s|
                DecodingKey::from_secret(s.as_bytes())
            ),
        })
    }

    pub fn validate_token(&self, token: &str) -> Result<Claims> {
        // Try primary key first
        if let Ok(data) = decode::<Claims>(token, &self.primary_key, &Validation::default()) {
            return Ok(data.claims);
        }

        // Fall back to secondary key if available
        if let Some(ref secondary) = self.secondary_key {
            if let Ok(data) = decode::<Claims>(token, secondary, &Validation::default()) {
                return Ok(data.claims);
            }
        }

        Err(anyhow!("Invalid token"))
    }
}
```

### 5. Automation Script

```bash
#!/bin/bash
# rotate_jwt_key.sh

# Generate new key
NEW_KEY=$(openssl rand -base64 32)
OLD_KEY=$(grep JWT_SECRET .env.production | cut -d'=' -f2)

# Backup current configuration
cp .env.production .env.production.backup

# Update environment file
sed -i "s/JWT_SECRET=.*/JWT_SECRET=$NEW_KEY/" .env.production
echo "JWT_SECRET_SECONDARY=$OLD_KEY" >> .env.production

# Deploy with dual keys
docker-compose -f docker-compose.production.yml up -d

# Schedule removal of secondary key after 14 days
echo "docker-compose exec api sed -i '/JWT_SECRET_SECONDARY/d' /app/.env" | at now + 14 days

echo "JWT key rotation initiated. Old tokens valid for 14 days."
```

### 6. Monitoring

Monitor the following metrics during rotation:
- Authentication failure rate
- Token validation errors
- User session disruptions

### 7. Emergency Rollback

If issues arise during rotation:
1. Restore the previous key as primary
2. Investigate failed authentications
3. Extend grace period if needed

## Security Best Practices

1. **Key Generation**: Use cryptographically secure random generators
2. **Key Length**: Minimum 256 bits (32 bytes)
3. **Rotation Frequency**: Every 90 days or after any security incident
4. **Key Storage**: Use environment variables or secret management systems
5. **Audit Logging**: Log all key rotation events

## Integration with Secret Management

For production environments, integrate with:
- **HashiCorp Vault**: Dynamic secret generation
- **AWS Secrets Manager**: Automatic rotation
- **Kubernetes Secrets**: Native K8s integration
- **Azure Key Vault**: Managed secret rotation

Example Vault integration:
```bash
# Fetch JWT secret from Vault
vault kv get -field=jwt_secret secret/hackerexperience/jwt
```

## Compliance

This rotation strategy complies with:
- PCI DSS requirements for key management
- SOC 2 Type II controls for cryptographic key rotation
- NIST 800-57 recommendations for key lifecycle

## Testing

Before production deployment:
1. Test rotation in staging environment
2. Verify zero downtime during rotation
3. Confirm all services handle dual keys
4. Validate monitoring and alerting

## References

- [NIST SP 800-57](https://nvlpubs.nist.gov/nistpubs/SpecialPublications/NIST.SP.800-57pt1r5.pdf)
- [JWT Best Practices RFC 8725](https://datatracker.ietf.org/doc/html/rfc8725)
- [OWASP Key Management Cheat Sheet](https://cheatsheetseries.owasp.org/cheatsheets/Key_Management_Cheat_Sheet.html)