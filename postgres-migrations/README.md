# HackerExperience PostgreSQL Database Schema

This directory contains the complete PostgreSQL database schema for the HackerExperience game, ported from the original MySQL system with significant enhancements.

## Overview

The database schema implements a comprehensive multi-system architecture supporting:

- **User Management** - Authentication, profiles, statistics, and progression
- **Hardware Systems** - Servers, specifications, and NPC infrastructure  
- **Process Management** - Complex process execution with resource tracking
- **Software Systems** - Tools, viruses, installations, and dependencies
- **Session Management** - Security-focused session tracking
- **Banking System** - Multi-bank financial operations and transactions
- **Clan System** - Organizations, membership, activities, and warfare
- **Mission System** - Quests, objectives, rewards, and progression
- **Logging System** - Comprehensive audit trails and monitoring
- **Network System** - Topology, connections, routing, and security
- **Performance Views** - Real-time dashboards and analytics

## Database Architecture

### Core Design Principles

1. **Performance First** - Comprehensive indexing and optimized queries
2. **Data Integrity** - Extensive constraints and validation
3. **Security Focus** - Audit trails and security monitoring
4. **Scalability** - Designed for high-concurrency gameplay
5. **Maintainability** - Clear documentation and modular design

### Table Count: 30+ Tables
- **Users & Authentication**: 3 tables
- **Hardware & Servers**: 3 tables  
- **Processes**: 3 tables
- **Software**: 3 tables
- **Sessions**: 2 tables
- **Banking**: 3 tables
- **Clans**: 4 tables
- **Missions**: 5 tables
- **Logging**: 6 tables
- **Network**: 5 tables
- **Utilities**: Various support tables

### Function Count: 25+ Functions
- User management and authentication
- Process lifecycle management  
- Financial transaction processing
- Clan operations and statistics
- Mission acceptance and completion
- Security event logging
- Network operations and scanning
- Database maintenance automation

### View Count: 15+ Views
- Real-time dashboards
- Performance monitoring
- Security alerts
- Player progress tracking
- Financial summaries
- Network topology

## Quick Start

### Prerequisites

- PostgreSQL 13+ with extensions:
  - `uuid-ossp` - UUID generation
  - `postgis` - Geographic data support  
  - `pg_trgm` - Text search capabilities
  - `btree_gin` - Advanced indexing
  - `pg_stat_statements` - Query monitoring

### Installation

1. **Run the setup script** (as PostgreSQL superuser):
   ```bash
   # Set environment variables
   export DB_HOST=localhost
   export DB_PORT=5432
   export DB_PASSWORD=your_secure_password
   
   # Run all migrations
   ./run_all_migrations.sh
   
   # Or with sample data for testing
   ./run_all_migrations.sh --with-sample-data
   ```

2. **Manual setup** (if needed):
   ```bash
   # Create database and user
   psql -U postgres -f 000_setup_database.sql
   
   # Run migrations in order
   psql -U he_app -d hackerexperience_rust -f 001_create_users_table.sql
   # ... continue with 002-014
   ```

### Configuration

Default connection settings:
- **Host**: localhost
- **Port**: 5432
- **Database**: hackerexperience_rust
- **User**: he_app
- **Password**: he_secure_password_change_in_production ⚠️ **Change this!**

Environment variables:
- `DB_HOST` - Database host
- `DB_PORT` - Database port
- `DB_NAME` - Database name
- `DB_USER` - Database user
- `DB_PASSWORD` - Database password

## Migration Files

| File | Description | Key Features |
|------|-------------|--------------|
| `000_setup_database.sql` | Database and user setup | Extensions, permissions, configuration |
| `001_create_users_table.sql` | User authentication | BCrypt passwords, IP tracking, premium status |
| `002_create_user_stats_table.sql` | User statistics | Experience, levels, financial tracking, automated calculations |
| `003_create_hardware_table.sql` | Server hardware | Performance metrics, security levels, connection tracking |
| `004_create_external_hardware_table.sql` | NPC servers | Hackable targets, difficulty calculation, rewards |
| `005_create_processes_table.sql` | Process management | Complex state machine, resource usage, progress tracking |
| `006_create_processes_paused_table.sql` | Pause management | Detailed pause tracking, auto-resume functionality |
| `007_create_software_table.sql` | Software system | Installation tracking, usage statistics, dependencies |
| `008_create_sessions_table.sql` | Session management | Security tracking, device fingerprinting, concurrent limits |
| `009_create_bank_system_tables.sql` | Banking system | Multi-bank support, transaction processing, interest calculation |
| `010_create_clan_system_tables.sql` | Clan management | Membership tracking, activities, warfare system |
| `011_create_mission_system_tables.sql` | Mission system | Objectives tracking, rewards, progress management |
| `012_create_comprehensive_logging_system.sql` | Logging infrastructure | Security events, audit trails, performance monitoring |
| `013_create_network_connection_system.sql` | Network topology | Connection tracking, routing, scanning, traffic analysis |
| `014_create_views_and_final_optimizations.sql` | Views and maintenance | Dashboard views, maintenance functions, performance optimization |

## Key Features

### 🔐 Security & Authentication
- **BCrypt password hashing** with proper salt handling
- **Session management** with device fingerprinting
- **IP tracking** for security monitoring  
- **Comprehensive audit logs** for all actions
- **Rate limiting** and suspicious activity detection

### ⚡ Performance & Scalability
- **100+ optimized indexes** for fast queries
- **Materialized views** for complex aggregations
- **Partitioning strategies** for large tables
- **Query optimization** with statistical analysis
- **Connection pooling** support

### 🎮 Game Mechanics
- **Complex process system** with resource management
- **Multi-objective missions** with branching logic
- **Clan warfare** with scoring and rewards
- **Economic system** with multiple currencies
- **Network simulation** with realistic topology

### 📊 Analytics & Monitoring  
- **Real-time dashboards** for players and admins
- **Performance metrics** collection and alerting
- **Player behavior tracking** for balance adjustments
- **System health monitoring** with automated maintenance

## Database Schema Highlights

### Advanced Constraints & Validation
```sql
-- Example: Process validation ensuring logical consistency
CONSTRAINT chk_process_state CHECK (
    NOT (is_completed = TRUE AND is_failed = TRUE)
),
CONSTRAINT chk_process_timing CHECK (
    processed_at IS NULL OR processed_at >= initiated_at
)
```

### Sophisticated Indexing Strategy
```sql
-- Partial indexes for active data
CREATE INDEX idx_processes_active ON processes(p_creator_id, p_time_end) 
    WHERE is_paused = FALSE AND is_completed = FALSE;

-- GIN indexes for JSONB searching
CREATE INDEX idx_software_dependencies ON software USING GIN(dependencies);

-- Geographic indexes for location queries  
CREATE INDEX idx_network_nodes_geo ON network_nodes USING GIST(geo_location);
```

### Automated Business Logic
```sql
-- Automatic level calculation from experience
CREATE TRIGGER user_stats_level_update_trigger
    BEFORE INSERT OR UPDATE OF experience ON user_stats
    FOR EACH ROW
    EXECUTE FUNCTION update_level_from_experience();
```

### Complex Query Views
```sql
-- Player dashboard with aggregated data
CREATE VIEW player_dashboard AS
SELECT 
    u.id, u.login, us.level, us.money,
    COUNT(p.pid) as active_processes,
    SUM(ba.balance) as total_bank_balance,
    c.name as clan_name
FROM users u
JOIN user_stats us ON u.id = us.user_id
LEFT JOIN processes p ON p.p_creator_id = u.id AND p.is_completed = FALSE
LEFT JOIN bank_accounts ba ON ba.user_id = u.id AND ba.account_status = 'active'
LEFT JOIN clan_members cm ON cm.user_id = u.id AND cm.status = 'active'  
LEFT JOIN clans c ON cm.clan_id = c.id
GROUP BY u.id, u.login, us.level, us.money, c.name;
```

## Maintenance & Operations

### Automated Maintenance
The database includes automated maintenance functions:

```sql
-- Run comprehensive maintenance
SELECT * FROM perform_database_maintenance();

-- Check system health  
SELECT * FROM get_database_health();

-- Clean up old data
SELECT * FROM cleanup_old_logs();
```

### Performance Monitoring
```sql
-- View performance statistics
SELECT * FROM log_statistics;

-- Monitor active processes
SELECT * FROM active_processes;

-- Check security alerts
SELECT * FROM security_alerts WHERE risk_level = 'IMMEDIATE';
```

### Backup Strategy
Recommended backup approach:
```bash
# Daily full backup
pg_dump -h localhost -U he_app hackerexperience_rust > daily_backup.sql

# Continuous WAL archiving for point-in-time recovery
# Configure postgresql.conf:
# archive_mode = on
# archive_command = 'cp %p /backup/wal_archive/%f'
```

## Development Guidelines

### Adding New Features
1. **Create migration file** with sequential numbering
2. **Add comprehensive indexes** for new queries
3. **Include proper constraints** for data integrity
4. **Document new functions** and views
5. **Update this README** with changes

### Performance Considerations
- **Always add indexes** for foreign keys and query columns
- **Use JSONB** instead of JSON for searchable data
- **Consider partitioning** for tables that grow rapidly
- **Profile queries** with `EXPLAIN ANALYZE`
- **Monitor index usage** with `pg_stat_user_indexes`

### Security Best Practices
- **Never store plain text passwords** 
- **Validate all inputs** with CHECK constraints
- **Log all security events** with proper context
- **Use row-level security** for multi-tenant data
- **Regularly audit permissions** and access patterns

## Troubleshooting

### Common Issues

**Connection refused**: Check PostgreSQL is running and accepting connections
```bash
sudo service postgresql start
pg_isready -h localhost -p 5432
```

**Permission denied**: Ensure user has proper privileges
```sql
GRANT ALL PRIVILEGES ON DATABASE hackerexperience_rust TO he_app;
GRANT ALL ON SCHEMA public TO he_app;
```

**Migration fails**: Check for conflicts with existing data
```sql
-- Check migration status
SELECT * FROM schema_migrations ORDER BY applied_at;

-- Rollback if needed (manual process)
-- Review migration file and manually reverse changes
```

**Poor performance**: Analyze query patterns and add indexes
```sql
-- Enable query logging
ALTER SYSTEM SET log_statement = 'all';
ALTER SYSTEM SET log_min_duration_statement = 1000; -- Log slow queries

-- Analyze table statistics
ANALYZE;

-- Check index usage
SELECT schemaname, tablename, indexname, idx_scan 
FROM pg_stat_user_indexes 
WHERE idx_scan = 0;
```

## Contributing

When contributing to the database schema:

1. **Follow naming conventions** (snake_case for all identifiers)
2. **Add comprehensive comments** explaining complex logic
3. **Include example usage** in function comments
4. **Test with sample data** before committing
5. **Update documentation** for any new features

## License

This database schema is part of the HackerExperience project. See the main project license for details.

## Support

For questions about the database schema:
1. Check this README and migration file comments
2. Review the function documentation in the SQL files
3. Create an issue in the main project repository

---

**⚠️ Security Notice**: Change all default passwords before production use!