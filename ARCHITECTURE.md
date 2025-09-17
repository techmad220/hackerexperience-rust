# HackerExperience Architecture Guide

## 🏗️ System Architecture

### Overview
HackerExperience is built using a microservices-inspired modular monolith architecture with clear separation of concerns.

```
┌─────────────────────────────────────────────────────────────┐
│                     Client Layer                             │
│  ┌──────────────┐  ┌──────────────┐  ┌──────────────┐      │
│  │   Web App    │  │  Mobile App  │  │   CLI Tool   │      │
│  │   (Leptos)   │  │   (Future)   │  │   (he-cli)   │      │
│  └──────┬───────┘  └──────┬───────┘  └──────┬───────┘      │
└─────────┼──────────────────┼──────────────────┼─────────────┘
          │                  │                  │
          ▼                  ▼                  ▼
┌─────────────────────────────────────────────────────────────┐
│                    API Gateway (Nginx)                       │
│         Load Balancing | SSL | Rate Limiting | Cache         │
└─────────────────────────┬───────────────────────────────────┘
                          │
┌─────────────────────────▼───────────────────────────────────┐
│                    Application Layer                         │
│  ┌────────────────────────────────────────────────────┐     │
│  │               API Service (he-api)                  │     │
│  │  ┌──────────┐  ┌──────────┐  ┌──────────┐        │     │
│  │  │  Auth    │  │   Game   │  │  Admin   │        │     │
│  │  │ Handlers │  │ Handlers │  │ Handlers │        │     │
│  │  └──────────┘  └──────────┘  └──────────┘        │     │
│  └────────────────────────────────────────────────────┘     │
│                                                              │
│  ┌────────────────────────────────────────────────────┐     │
│  │           WebSocket Service (he-websocket)          │     │
│  │     Real-time Events | Game Updates | Chat         │     │
│  └────────────────────────────────────────────────────┘     │
└──────────────────────────────────────────────────────────────┘
                          │
┌─────────────────────────▼───────────────────────────────────┐
│                   Business Logic Layer                       │
│  ┌──────────────────────────────────────────────────────┐   │
│  │          Game Mechanics Engine (he-game-mechanics)    │   │
│  │  ┌──────────┐  ┌──────────┐  ┌──────────┐          │   │
│  │  │ Process  │  │ Hacking  │  │ Hardware │          │   │
│  │  │ Manager  │  │  System  │  │ Manager  │          │   │
│  │  └──────────┘  └──────────┘  └──────────┘          │   │
│  └──────────────────────────────────────────────────────┘   │
│                                                              │
│  ┌──────────────────────────────────────────────────────┐   │
│  │            Security Module (he-helix-security)        │   │
│  │   Auth | Encryption | Audit | Intrusion Detection    │   │
│  └──────────────────────────────────────────────────────┘   │
└──────────────────────────────────────────────────────────────┘
                          │
┌─────────────────────────▼───────────────────────────────────┐
│                    Data Access Layer                         │
│  ┌──────────────────────────────────────────────────────┐   │
│  │         Database Service (he-database)               │   │
│  │   Models | Queries | Migrations | Connection Pool    │   │
│  └──────────────────────────────────────────────────────┘   │
└──────────────────────────────────────────────────────────────┘
                          │
┌─────────────────────────▼───────────────────────────────────┐
│                    Infrastructure Layer                      │
│  ┌────────────┐  ┌────────────┐  ┌────────────┐           │
│  │ PostgreSQL │  │   Redis    │  │    S3      │           │
│  │  Database  │  │   Cache    │  │  Storage   │           │
│  └────────────┘  └────────────┘  └────────────┘           │
└──────────────────────────────────────────────────────────────┘
```

## 📦 Module Organization

### Core Modules
```
crates/
├── he-api/              # REST API endpoints
├── he-auth/             # Authentication & authorization
├── he-database/         # Database access layer
├── he-game-mechanics/   # Game logic and calculations
├── he-websocket/        # Real-time communication
├── he-leptos-frontend/  # Web frontend
└── he-core/             # Shared types and utilities
```

### Module Responsibilities

#### `he-api`
- HTTP request handling
- Request validation
- Response formatting
- Rate limiting
- CORS management

#### `he-auth`
- JWT token generation/validation
- Password hashing (Argon2id)
- Session management
- Permission checking
- OAuth integration (future)

#### `he-database`
- Database models
- Query builders
- Migration management
- Connection pooling
- Transaction handling

#### `he-game-mechanics`
- Process calculations
- Combat simulation
- Resource management
- Mission logic
- Experience/leveling

#### `he-websocket`
- Real-time events
- Connection management
- Event broadcasting
- Presence tracking
- Chat system

## 🎯 Design Principles

### 1. Separation of Concerns
Each module has a single, well-defined responsibility.

### 2. Dependency Injection
```rust
// Bad
async fn get_user() -> User {
    let db = Database::new().await;
    db.get_user(1).await
}

// Good
async fn get_user(db: &Database) -> User {
    db.get_user(1).await
}
```

### 3. Error Handling
```rust
// Unified error type
#[derive(Debug, thiserror::Error)]
pub enum HeError {
    #[error("Database error: {0}")]
    Database(#[from] sqlx::Error),

    #[error("Authentication failed")]
    AuthFailed,

    #[error("Invalid input: {0}")]
    InvalidInput(String),

    #[error("Not found")]
    NotFound,
}

// All functions return Result
pub async fn process_action() -> Result<Response, HeError> {
    // ...
}
```

### 4. Async by Default
```rust
// All I/O operations are async
pub async fn fetch_data(id: i64) -> Result<Data, HeError> {
    // ...
}
```

### 5. Type Safety
```rust
// Use strong types instead of primitives
pub struct UserId(i64);
pub struct ProcessId(Uuid);
pub struct Money(Decimal);

// Type-safe queries
let user = sqlx::query_as!(
    User,
    "SELECT * FROM users WHERE id = $1",
    user_id.0
).fetch_one(&pool).await?;
```

## 🔧 Standard Patterns

### Request Handler Pattern
```rust
pub async fn handler(
    State(state): State<AppState>,
    ValidatedJson(payload): ValidatedJson<Request>,
) -> Result<Json<Response>, ApiError> {
    // 1. Validate input
    payload.validate()?;

    // 2. Check authorization
    let user = state.auth.validate_token(&payload.token)?;

    // 3. Business logic
    let result = state.service.process(&user, &payload).await?;

    // 4. Format response
    Ok(Json(Response::from(result)))
}
```

### Service Pattern
```rust
pub struct GameService {
    db: Arc<Database>,
    cache: Arc<Redis>,
    engine: Arc<GameEngine>,
}

impl GameService {
    pub async fn create_process(
        &self,
        user: &User,
        process_type: ProcessType,
    ) -> Result<Process, ServiceError> {
        // 1. Check preconditions
        self.validate_resources(user, &process_type).await?;

        // 2. Calculate parameters
        let duration = self.engine.calculate_duration(user, &process_type);

        // 3. Create in database
        let process = self.db.create_process(user.id, process_type, duration).await?;

        // 4. Update cache
        self.cache.set(&format!("process:{}", process.id), &process).await?;

        // 5. Send event
        self.broadcast_event(GameEvent::ProcessStarted(process.clone())).await?;

        Ok(process)
    }
}
```

### Repository Pattern
```rust
#[async_trait]
pub trait UserRepository {
    async fn find_by_id(&self, id: UserId) -> Result<Option<User>, RepoError>;
    async fn find_by_email(&self, email: &str) -> Result<Option<User>, RepoError>;
    async fn create(&self, user: NewUser) -> Result<User, RepoError>;
    async fn update(&self, id: UserId, updates: UserUpdate) -> Result<User, RepoError>;
    async fn delete(&self, id: UserId) -> Result<(), RepoError>;
}

pub struct PostgresUserRepository {
    pool: PgPool,
}

#[async_trait]
impl UserRepository for PostgresUserRepository {
    async fn find_by_id(&self, id: UserId) -> Result<Option<User>, RepoError> {
        sqlx::query_as!(User, "SELECT * FROM users WHERE id = $1", id.0)
            .fetch_optional(&self.pool)
            .await
            .map_err(Into::into)
    }
    // ... other methods
}
```

## 🧪 Testing Strategy

### Unit Tests
```rust
#[cfg(test)]
mod tests {
    use super::*;
    use mockall::mock;

    mock! {
        Database {}

        #[async_trait]
        impl UserRepository for Database {
            async fn find_by_id(&self, id: UserId) -> Result<Option<User>, RepoError>;
        }
    }

    #[tokio::test]
    async fn test_get_user() {
        let mut mock_db = MockDatabase::new();
        mock_db.expect_find_by_id()
            .with(eq(UserId(1)))
            .returning(|_| Ok(Some(User::default())));

        let service = UserService::new(Arc::new(mock_db));
        let user = service.get_user(UserId(1)).await.unwrap();
        assert!(user.is_some());
    }
}
```

### Integration Tests
```rust
// tests/integration_test.rs
#[tokio::test]
async fn test_user_registration_flow() {
    let app = create_test_app().await;

    let response = app
        .post("/api/auth/register")
        .json(&json!({
            "username": "testuser",
            "email": "test@example.com",
            "password": "password123"
        }))
        .send()
        .await;

    assert_eq!(response.status(), StatusCode::CREATED);

    let body: RegisterResponse = response.json().await;
    assert!(!body.token.is_empty());
}
```

## 🔐 Security Patterns

### Input Validation
```rust
#[derive(Debug, Deserialize, Validate)]
pub struct CreateProcessRequest {
    #[validate(length(min = 1, max = 50))]
    pub process_type: String,

    #[validate(range(min = 1, max = 100))]
    pub priority: Option<u8>,

    #[validate(custom = "validate_target")]
    pub target_id: Option<String>,
}
```

### Rate Limiting
```rust
pub async fn rate_limit_middleware(
    State(state): State<Arc<RateLimiter>>,
    req: Request,
    next: Next,
) -> Response {
    let ip = extract_ip(&req);

    if !state.check_rate_limit(&ip).await {
        return StatusCode::TOO_MANY_REQUESTS.into_response();
    }

    next.run(req).await
}
```

## 🚀 Performance Patterns

### Caching
```rust
pub async fn get_user_cached(
    &self,
    id: UserId,
) -> Result<User, Error> {
    // Check cache first
    if let Some(user) = self.cache.get(&format!("user:{}", id.0)).await? {
        return Ok(user);
    }

    // Fetch from database
    let user = self.db.find_by_id(id).await?
        .ok_or(Error::NotFound)?;

    // Update cache
    self.cache.set(&format!("user:{}", id.0), &user, Duration::from_secs(300)).await?;

    Ok(user)
}
```

### Connection Pooling
```rust
pub async fn create_pool() -> PgPool {
    PgPoolOptions::new()
        .max_connections(100)
        .min_connections(10)
        .connect_timeout(Duration::from_secs(5))
        .idle_timeout(Duration::from_secs(600))
        .connect(&database_url)
        .await
        .expect("Failed to create pool")
}
```

## 📝 Code Style Guidelines

### Naming Conventions
- **Modules**: snake_case (`user_service`)
- **Types**: PascalCase (`UserAccount`)
- **Functions**: snake_case (`get_user_by_id`)
- **Constants**: SCREAMING_SNAKE_CASE (`MAX_RETRY_COUNT`)
- **Lifetimes**: short lowercase (`'a`, `'b`)

### Documentation
```rust
/// Calculates the duration of a process based on hardware specs.
///
/// # Arguments
/// * `process_type` - The type of process to calculate
/// * `hardware` - Current hardware specifications
///
/// # Returns
/// Duration in seconds
///
/// # Example
/// ```
/// let duration = calculate_process_duration(
///     ProcessType::Hack,
///     &hardware_specs
/// );
/// assert!(duration > 0);
/// ```
pub fn calculate_process_duration(
    process_type: ProcessType,
    hardware: &HardwareSpecs,
) -> u64 {
    // Implementation
}
```

## 🔄 Migration Path

### Phase 1: Consolidate Duplicate Systems
1. Merge `he-*` and `he-helix-*` crates
2. Remove Actor pattern where not needed
3. Standardize error types

### Phase 2: Improve Test Coverage
1. Add unit tests to all modules
2. Create integration test suite
3. Add property-based tests

### Phase 3: Documentation
1. Document all public APIs
2. Add architecture diagrams
3. Create developer guides

### Phase 4: Performance
1. Add caching layer
2. Optimize database queries
3. Implement connection pooling

## 🎓 Best Practices

1. **Keep functions small** - Single responsibility
2. **Use descriptive names** - Code should be self-documenting
3. **Handle errors explicitly** - No unwrap() in production
4. **Write tests first** - TDD approach
5. **Document public APIs** - With examples
6. **Use type safety** - Leverage Rust's type system
7. **Async all I/O** - Never block the runtime
8. **Validate input** - At system boundaries
9. **Log appropriately** - Info for operations, Debug for details
10. **Profile regularly** - Measure before optimizing