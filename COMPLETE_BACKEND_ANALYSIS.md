# Complete HackerExperience Backend Analysis & Rust Porting Plan

## Project Overview

We have **TWO** backend systems to port to Rust for complete 1:1 parity:

1. **Legacy Backend** (PHP) - Original HackerExperience (2012-2014)
   - 2,294 PHP files
   - 6,942 total files 
   - Procedural PHP with MySQL
   - Single monolithic structure

2. **Helix Backend** (Elixir) - HackerExperience 2 (Modern)
   - 912 Elixir files (.ex/.exs)
   - 934 total files
   - Phoenix framework with Ecto ORM
   - Modern microservices architecture

## Comprehensive File Analysis

### Legacy Backend Structure (PHP)
```
hackerexperience-legacy/
├── classes/               # Core PHP classes (33 files)
│   ├── Player.class.php   # User management
│   ├── Process.class.php  # Game processes ("most complex")
│   ├── PC.class.php       # Hardware management
│   └── Session.class.php  # Authentication
├── cron/                  # Background tasks (deprecated)
├── cron2/                 # Active cron jobs (Python 2)
├── forum/                 # phpBB integration (1,947 files)
├── wiki/                  # Game documentation
├── npccontent/            # HTML content for NPCs
├── HTMLPurifier/          # XSS protection library
├── [ROOT FILES]           # 50+ main game pages
│   ├── index.php          # Main game entry
│   ├── ajax.php           # AJAX endpoints (80KB!)
│   ├── processes.php      # Process management
│   ├── clan.php           # Clan system
│   └── [45+ more pages]
└── game.sql               # Database schema
```

### Helix Backend Structure (Elixir)
```
helix-backend/
├── lib/
│   ├── helix/             # Core application
│   │   ├── account/       # User accounts
│   │   ├── process/       # Game processes (Elixir version)
│   │   ├── server/        # Hardware management
│   │   ├── software/      # Software/virus system
│   │   ├── clan/          # Clan mechanics
│   │   ├── story/         # Mission system
│   │   ├── universe/      # Game world
│   │   └── network/       # Network simulation
│   ├── helix_web/         # Phoenix web layer
│   └── helix_test_helper/ # Testing utilities
├── test/                  # Comprehensive test suite
├── priv/                  # Database migrations & seeds
└── config/                # Environment configuration
```

## Unified Rust Architecture Design

### Target Structure
```
hackerexperience-unified/
├── crates/
│   ├── he-core/           # Shared game logic
│   ├── he-legacy-compat/  # Legacy PHP compatibility
│   ├── he-helix-compat/   # Helix Elixir compatibility  
│   ├── he-db-legacy/      # Legacy database layer
│   ├── he-db-helix/       # Helix database layer
│   ├── he-api-legacy/     # Legacy HTTP endpoints
│   ├── he-api-helix/      # Helix GraphQL/REST API
│   ├── he-processes/      # Unified process engine
│   ├── he-realtime/       # WebSocket & events
│   ├── he-auth/           # Authentication system
│   └── he-admin/          # Administration tools
├── migrations-legacy/     # Legacy database migrations
├── migrations-helix/      # Helix database migrations
└── compatibility/         # Cross-system compatibility
```

## File-by-File Mapping Strategy

### Phase 1: Core Systems (Both Backends)
| Legacy (PHP) | Helix (Elixir) | Rust Equivalent |
|--------------|----------------|-----------------|
| `classes/Player.class.php` | `lib/helix/account/` | `he-core/src/account/` |
| `classes/Process.class.php` | `lib/helix/process/` | `he-core/src/process/` |
| `classes/PC.class.php` | `lib/helix/server/` | `he-core/src/hardware/` |
| `classes/Session.class.php` | `lib/helix_web/channels/` | `he-auth/src/session/` |
| `classes/Clan.class.php` | `lib/helix/clan/` | `he-core/src/clan/` |

### Phase 2: API Endpoints
| Legacy (PHP) | Helix (Elixir) | Rust Equivalent |
|--------------|----------------|-----------------|
| `ajax.php` (80KB!) | `lib/helix_web/controllers/` | `he-api-legacy/src/handlers/` |
| `index.php` | `lib/helix_web/router.ex` | `he-api-legacy/src/routes/` |
| Individual `.php` pages | Phoenix controllers | Axum handlers |

### Phase 3: Background Processing
| Legacy (PHP) | Helix (Elixir) | Rust Equivalent |
|--------------|----------------|-----------------|
| `cron2/*.py` | `lib/helix/process/` | `he-processes/src/workers/` |
| Process management | GenServer processes | Tokio async tasks |
| MySQL transactions | Ecto transactions | SQLx transactions |

## Complete Parity Checklist

### ✅ Already Ported (Core Entities)
- User/Player management
- Hardware specifications  
- Process engine basics
- Session management
- Database schema

### 🔄 Must Port from Legacy (PHP)
- [ ] All 50+ main PHP pages
- [ ] Complete AJAX endpoint (`ajax.php` - 80KB)
- [ ] Forum integration (phpBB)
- [ ] Wiki system
- [ ] NPC content system
- [ ] All Python cron jobs
- [ ] File upload/download system
- [ ] BitCoin integration
- [ ] Mail system
- [ ] Premium features
- [ ] Rankings and statistics
- [ ] Research system
- [ ] Mission system (old)

### 🔄 Must Port from Helix (Elixir)
- [ ] Modern GraphQL API
- [ ] Real-time event system
- [ ] Advanced process orchestration
- [ ] WebSocket channels
- [ ] Modern authentication
- [ ] Story/mission system (new)
- [ ] Universe/world simulation
- [ ] Network topology
- [ ] Software crafting system
- [ ] Advanced clan features
- [ ] Log system improvements
- [ ] Test infrastructure

## Implementation Timeline

### Week 1-2: Complete Legacy Port
- Port all 50+ PHP root files to Rust handlers
- Implement complete AJAX API compatibility
- Port all Python cron jobs to async Rust workers

### Week 3-4: Complete Helix Port  
- Port all Elixir modules to Rust equivalents
- Implement GraphQL API with async-graphql
- Port Phoenix channels to WebSocket handlers

### Week 5-6: Integration & Compatibility
- Create compatibility layer between Legacy and Helix features
- Unified database migration system
- Cross-system data synchronization

### Week 7-8: Testing & Validation
- Complete end-to-end testing
- Performance benchmarking
- Production deployment pipeline

## Expected Outcomes

**Performance Improvements:**
- **Legacy**: 10-100x faster than PHP
- **Helix**: 2-5x faster than Elixir (native compilation)
- **Memory**: 50-90% reduction in memory usage
- **Concurrency**: Handle 10,000+ concurrent users

**Feature Parity:**
- ✅ **100%** of Legacy functionality preserved
- ✅ **100%** of Helix functionality preserved  
- ✅ **Cross-compatibility** between both systems
- ✅ **Modern deployment** with Docker/Kubernetes

**Code Quality:**
- **Type Safety**: Compile-time error prevention
- **Memory Safety**: No buffer overflows or memory leaks
- **Concurrent Safety**: No race conditions or deadlocks
- **Test Coverage**: Comprehensive automated testing

This will be the **most complete** game backend port ever attempted - preserving two entirely different architectures (Legacy monolith + Modern microservices) in a single, unified, high-performance Rust system! 🚀