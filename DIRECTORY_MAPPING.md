# Helix Elixir to Rust Directory Structure Mapping

## Current Elixir Structure → Proposed Rust Structure

### Root Level Mapping
```
helix/lib/                    → helix/crates/
helix/mix.exs                 → helix/Cargo.toml (workspace)
helix/config/                 → helix/config/
helix/priv/                   → helix/assets/ (migrations → migrations/)
helix/test/                   → helix/crates/*/tests/
```

### Domain Module Mapping

#### Account Domain (25 files)
```
lib/account/                  → crates/helix-account/
├── model/                    → src/models/
├── public/                   → src/api/
├── event/                    → src/events/
├── action/                   → src/actions/
├── query/                    → src/queries/
├── henforcer/                → src/validation/
└── supervisor.ex             → src/supervisor.rs
```

#### Cache Domain (16 files)
```
lib/cache/                    → crates/helix-cache/
├── model/                    → src/models/
├── public/                   → src/api/
├── query/                    → src/queries/
└── supervisor.ex             → src/supervisor.rs
```

#### Client Domain (17 files)
```
lib/client/                   → crates/helix-client/
├── model/                    → src/models/
├── public/                   → src/api/
├── web1/                     → src/web1/
├── event/                    → src/events/
├── action/                   → src/actions/
└── supervisor.ex             → src/supervisor.rs
```

#### Core Domain (11 files)
```
lib/core/                     → crates/helix-core/
├── listener/                 → src/listeners/
├── validator/                → src/validation/
└── supervisor.ex             → src/supervisor.rs
```

#### Entity Domain (20 files)
```
lib/entity/                   → crates/helix-entity/
├── model/                    → src/models/
├── public/                   → src/api/
├── event/                    → src/events/
├── action/                   → src/actions/
├── query/                    → src/queries/
├── henforcer/                → src/validation/
└── supervisor.ex             → src/supervisor.rs
```

#### Event Domain (17 files)
```
lib/event/                    → crates/helix-events/
├── dispatcher.ex             → src/dispatcher.rs
├── event.ex                  → src/event.rs
├── meta.ex                   → src/metadata.rs
├── publication_handler.ex    → src/publication.rs
├── listenable/               → src/listenable/
├── loggable/                 → src/loggable/
├── notificable/              → src/notificable/
├── publishable/              → src/publishable/
├── state/                    → src/state/
└── supervisor.ex             → src/supervisor.rs
```

#### Log Domain (11 files)
```
lib/log/                      → crates/helix-log/
├── model/                    → src/models/
├── public/                   → src/api/
├── event/                    → src/events/
├── action/                   → src/actions/
├── query/                    → src/queries/
└── supervisor.ex             → src/supervisor.rs
```

#### Network Domain (46 files)
```
lib/network/                  → crates/helix-network/
├── model/                    → src/models/
├── public/                   → src/api/
├── event/                    → src/events/
├── action/                   → src/actions/
├── query/                    → src/queries/
├── henforcer/                → src/validation/
├── internal/                 → src/internal/
└── supervisor.ex             → src/supervisor.rs
```

#### Notification Domain (21 files)
```
lib/notification/             → crates/helix-notification/
├── model/                    → src/models/
├── public/                   → src/api/
├── event/                    → src/events/
├── action/                   → src/actions/
└── supervisor.ex             → src/supervisor.rs
```

#### Process Domain (35 files) - COMPLEX
```
lib/process/                  → crates/helix-process/
├── model/                    → src/models/
├── public/                   → src/api/
├── event/                    → src/events/
├── action/                   → src/actions/
├── query/                    → src/queries/
├── internal/                 → src/internal/
├── resources/                → src/resources/
├── executable.ex             → src/executable.rs
├── processable.ex            → src/processable.rs
├── resourceable.ex           → src/resourceable.rs
├── resources.ex              → src/resources.rs
├── viewable.ex               → src/viewable.rs
└── supervisor.ex             → src/supervisor.rs
```

#### Server Domain (51 files)
```
lib/server/                   → crates/helix-server/
├── model/                    → src/models/
├── public/                   → src/api/
├── event/                    → src/events/
├── action/                   → src/actions/
├── query/                    → src/queries/
├── henforcer/                → src/validation/
├── component/                → src/components/
├── web/                      → src/web/
└── supervisor.ex             → src/supervisor.rs
```

#### Software Domain (87 files) - LARGEST
```
lib/software/                 → crates/helix-software/
├── model/                    → src/models/
├── public/                   → src/api/
├── event/                    → src/events/
├── action/                   → src/actions/
├── query/                    → src/queries/
├── henforcer/                → src/validation/
├── internal/                 → src/internal/
├── file/                     → src/file/
├── process/                  → src/process/
└── supervisor.ex             → src/supervisor.rs
```

#### Story Domain (34 files)
```
lib/story/                    → crates/helix-story/
├── model/                    → src/models/
├── public/                   → src/api/
├── event/                    → src/events/
├── action/                   → src/actions/
├── query/                    → src/queries/
├── mission/                  → src/missions/
└── supervisor.ex             → src/supervisor.rs
```

#### Universe Domain (35 files)
```
lib/universe/                 → crates/helix-universe/
├── model/                    → src/models/
├── public/                   → src/api/
├── event/                    → src/events/
├── action/                   → src/actions/
├── query/                    → src/queries/
├── bank/                     → src/bank/
├── npc/                      → src/npc/
└── supervisor.ex             → src/supervisor.rs
```

#### WebSocket Domain (10 files)
```
lib/websocket/                → crates/helix-websocket/
├── channel.ex                → src/channel.rs
├── flow.ex                   → src/flow.rs
├── websocket.ex              → src/websocket.rs
├── utils.ex                  → src/utils.rs
├── join/                     → src/join/
└── request/                  → src/request/
```

### Supporting Infrastructure

#### HTTP & API
```
lib/http/                     → crates/helix-api/
lib/endpoint.ex               → crates/helix-api/src/endpoint.rs
```

#### Shared Utilities
```
lib/hell/                     → crates/helix-hell/
lib/factor/                   → crates/helix-core/src/factor/
lib/henforcer/                → crates/helix-core/src/henforcer/
lib/id/                       → crates/helix-core/src/id/
lib/balance/                  → crates/helix-core/src/balance/
```

#### Application & Configuration
```
lib/application.ex            → src/main.rs
lib/logger.ex                 → crates/helix-core/src/logger.rs
lib/release.ex                → build configuration
lib/appsignal.ex              → crates/helix-core/src/monitoring.rs
```

## File Pattern Mapping

### Elixir → Rust Patterns
```
*.ex                         → *.rs
*_test.exs                   → tests/*.rs or src/*_test.rs (with #[cfg(test)])
supervisor.ex                → supervisor.rs
repo.ex                      → repository.rs
model/*.ex                   → models/*.rs
public/*.ex                  → api/*.rs
event/*.ex                   → events/*.rs
action/*.ex                  → actions/*.rs
query/*.ex                   → queries/*.rs
henforcer/*.ex               → validation/*.rs
internal/*.ex                → internal/*.rs
```

### Configuration Mapping
```
config/config.exs             → config/default.toml
config/dev.exs               → config/development.toml
config/prod.exs              → config/production.toml
config/test.exs              → config/test.toml
config/*/config.exs          → config/domains/*.toml
```

### Database Mapping
```
priv/repo/*/migrations/      → migrations/*/
priv/repo/*/seeds.exs        → fixtures/*.sql or fixtures/*.rs
```

## Rust Crate Dependencies

### Inter-Crate Dependencies
```
helix-core                   ← All other crates depend on this
helix-database              ← Domain crates depend on this
helix-events                ← Domain crates depend on this

helix-account               → helix-core, helix-database, helix-events
helix-cache                 → helix-core, helix-database
helix-client                → helix-core, helix-database, helix-events
helix-entity                → helix-core, helix-database, helix-events
helix-log                   → helix-core, helix-database, helix-events
helix-network               → helix-core, helix-database, helix-events, helix-server
helix-notification          → helix-core, helix-database, helix-events
helix-process               → helix-core, helix-database, helix-events, helix-server, helix-network
helix-server                → helix-core, helix-database, helix-events
helix-software              → helix-core, helix-database, helix-events, helix-server, helix-process
helix-story                 → helix-core, helix-database, helix-events
helix-universe              → helix-core, helix-database, helix-events

helix-websocket             → All domain crates for message routing
helix-api                   → All domain crates for HTTP endpoints
```

### External Dependencies by Crate

#### helix-core
```toml
[dependencies]
tokio = "1.0"
serde = "1.0"
anyhow = "1.0"
thiserror = "1.0"
tracing = "0.1"
uuid = "1.0"
chrono = "0.4"
```

#### helix-database
```toml
[dependencies]
helix-core = { path = "../helix-core" }
sqlx = "0.7"
uuid = "1.0"
chrono = "0.4"
serde = "1.0"
```

#### helix-events
```toml
[dependencies]
helix-core = { path = "../helix-core" }
tokio = "1.0"
futures = "0.3"
serde = "1.0"
```

#### Domain crates (typical)
```toml
[dependencies]
helix-core = { path = "../helix-core" }
helix-database = { path = "../helix-database" }
helix-events = { path = "../helix-events" }
tokio = "1.0"
serde = "1.0"
uuid = "1.0"
```

#### helix-websocket
```toml
[dependencies]
helix-core = { path = "../helix-core" }
tokio = "1.0"
tokio-tungstenite = "0.21"
serde = "1.0"
serde_json = "1.0"
futures = "0.3"
```

#### helix-api
```toml
[dependencies]
helix-core = { path = "../helix-core" }
axum = "0.7"
tower = "0.4"
tower-http = "0.5"
serde = "1.0"
serde_json = "1.0"
tokio = "1.0"
```

## Migration Strategy

### Phase-by-Phase Directory Creation
1. **Phase 1**: Create helix-core, helix-database, helix-events, helix-config
2. **Phase 2**: Create helix-account, helix-entity, helix-server
3. **Phase 3**: Create helix-process, helix-software, helix-network
4. **Phase 4**: Create helix-websocket, helix-api, helix-cache
5. **Phase 5**: Create helix-story, helix-universe, helix-notification
6. **Phase 6**: Create helix-log and remaining utilities

### Common File Structure per Domain Crate
```
crates/helix-domain/
├── Cargo.toml
├── src/
│   ├── lib.rs              # Public API exports
│   ├── models/             # Database models
│   │   └── mod.rs
│   ├── api/                # Public API functions
│   │   └── mod.rs
│   ├── events/             # Domain events
│   │   └── mod.rs
│   ├── actions/            # Business logic
│   │   └── mod.rs
│   ├── queries/            # Database queries
│   │   └── mod.rs
│   ├── validation/         # Input validation
│   │   └── mod.rs
│   ├── internal/           # Internal utilities
│   │   └── mod.rs
│   └── supervisor.rs       # Actor supervision
├── tests/
│   └── integration.rs
└── migrations/             # Database migrations (if domain-specific)
    └── README.md
```

This mapping provides a clear translation path from the existing Elixir structure to a well-organized Rust workspace.