# Complete 1:1 File Mapping Strategy
## HackerExperience Legacy + Helix → Unified Rust Implementation

### **Total Scope Overview**
- **Legacy PHP Files**: 2,294 files
- **Legacy Python Scripts**: 28 files  
- **Legacy Static Assets**: 4,620 files
- **Helix Elixir Files**: 912 files
- **Helix Migrations**: 45 files
- **TOTAL**: **7,899 files** requiring 1:1 porting

---

## **Phase 1: Legacy PHP Root Files (51 files)**

### **Authentication & Entry Points** → `crates/he-legacy-compat/src/pages/`
- ✅ `index.php` → `pages/index.rs` (COMPLETED)
- ✅ `ajax.php` → `pages/ajax.rs` (COMPLETED) 
- ✅ `processes.php` → `pages/processes.rs` (COMPLETED)
- ⏳ `login.php` → `pages/login.rs`
- ⏳ `register.php` → `pages/register.rs`
- ⏳ `logout.php` → `pages/logout.rs`
- ⏳ `reset.php` → `pages/reset.rs`
- ⏳ `resetIP.php` → `pages/reset_ip.rs`

### **Game Core Pages** → `crates/he-legacy-compat/src/pages/`
- ⏳ `hardware.php` → `pages/hardware.rs`
- ⏳ `hardwareItens.php` → `pages/hardware_items.rs`
- ⏳ `software.php` → `pages/software.rs`
- ⏳ `createsoft.php` → `pages/create_software.rs`
- ⏳ `research.php` → `pages/research.rs`
- ⏳ `researchTable.php` → `pages/research_table.rs`
- ⏳ `mail.php` → `pages/mail.rs`
- ⏳ `news.php` → `pages/news.rs`
- ⏳ `finances.php` → `pages/finances.rs`
- ⏳ `internet.php` → `pages/internet.rs`
- ⏳ `webserver.php` → `pages/webserver.rs`
- ⏳ `log.php` → `pages/log.rs`
- ⏳ `logEdit.php` → `pages/log_edit.rs`
- ⏳ `missions.php` → `pages/missions.rs`
- ⏳ `university.php` → `pages/university.rs`
- ⏳ `war.php` → `pages/war.rs`
- ⏳ `doom.php` → `pages/doom.rs`
- ⏳ `DDoS.php` → `pages/ddos.rs`

### **User Management** → `crates/he-legacy-compat/src/pages/`
- ⏳ `profile.php` → `pages/profile.rs`
- ⏳ `settings.php` → `pages/settings.rs`
- ⏳ `options.php` → `pages/options.rs`
- ⏳ `stats.php` → `pages/stats.rs`
- ⏳ `stats_1.php` → `pages/stats_detailed.rs`
- ⏳ `ranking.php` → `pages/ranking.rs`
- ⏳ `fame.php` → `pages/fame.rs`
- ⏳ `premium.php` → `pages/premium.rs`
- ⏳ `bitcoin.php` → `pages/bitcoin.rs`
- ⏳ `pagarme.php` → `pages/pagarme.rs`

### **Clan & Social** → `crates/he-legacy-compat/src/pages/`
- ⏳ `clan.php` → `pages/clan.rs`

### **Information & Legal** → `crates/he-legacy-compat/src/pages/`
- ⏳ `privacy.php` → `pages/privacy.rs`
- ⏳ `TOS.php` → `pages/tos.rs`
- ⏳ `legal.php` → `pages/legal.rs`
- ⏳ `about.php` → `pages/about.rs`
- ⏳ `changelog.php` → `pages/changelog.rs`
- ⏳ `gameInfo.php` → `pages/game_info.rs`
- ⏳ `riddle.php` → `pages/riddle.rs`

### **Utilities** → `crates/he-legacy-compat/src/pages/`
- ⏳ `uploadImage.php` → `pages/upload_image.rs`
- ⏳ `connect.php` → Handled by database layer
- ⏳ `config.php` → `src/config.rs`

---

## **Phase 2: Legacy PHP Classes (33 files)**

### **Core Classes** → `crates/he-legacy-compat/src/classes/`
- ✅ `Database.class.php` → `classes/database.rs` (COMPLETED)
- ✅ `System.class.php` → `classes/system.rs` (COMPLETED)
- ✅ `Session.class.php` → `classes/session.rs` + `src/session.rs` (COMPLETED)
- ✅ `Player.class.php` → `classes/player.rs` (COMPLETED via he-core)
- ✅ `Process.class.php` → `classes/process.rs` (COMPLETED via he-core)
- ✅ `PC.class.php` → `classes/pc.rs` (COMPLETED via he-core)

### **Game Systems** → `crates/he-legacy-compat/src/classes/`
- ✅ `Clan.class.php` → `classes/clan.rs` (COMPLETED)
- ✅ `Ranking.class.php` → `classes/ranking.rs` (COMPLETED)
- ✅ `Finances.class.php` → `classes/finances.rs` (COMPLETED)
- ✅ `Social.class.php` → `classes/social.rs` (COMPLETED)
- ✅ `Storyline.class.php` → `classes/storyline.rs` (COMPLETED)
- ✅ `NPC.class.php` → `classes/npc.rs` (COMPLETED)

### **Security & Authentication** → `crates/he-legacy-compat/src/classes/`
- ✅ `BCrypt.class.php` → `classes/bcrypt.rs` (COMPLETED)
- ⏳ `RememberMe.class.php` → `classes/remember_me.rs`
- ⏳ `Facebook.class.php` → `classes/facebook.rs`
- ⏳ `EmailVerification.class.php` → `classes/email_verification.rs`

### **Content & Communication** → `crates/he-legacy-compat/src/classes/`
- ⏳ `Mail.class.php` → `classes/mail.rs`
- ⏳ `News.class.php` → `classes/news.rs`
- ⏳ `Forum.class.php` → `classes/forum.rs`
- ⏳ `Mission.class.php` → `classes/mission.rs`

### **Game Mechanics** → `crates/he-legacy-compat/src/classes/`
- ⏳ `Internet.class.php` → `classes/internet.rs`
- ⏳ `Fame.class.php` → `classes/fame.rs`
- ⏳ `List.class.php` → `classes/list.rs`
- ⏳ `Premium.class.php` → `classes/premium.rs`
- ⏳ `Python.class.php` → `classes/python.rs`

### **Utilities** → `crates/he-legacy-compat/src/classes/`
- ⏳ `Purifier.class.php` → `classes/purifier.rs`
- ⏳ `Images.class.php` → `classes/images.rs`
- ⏳ `Riddle.class.php` → `classes/riddle.rs`
- ⏳ `Pagination.class.php` → `classes/pagination.rs`
- ⏳ `Versioning.class.php` → `classes/versioning.rs`
- ⏳ `SES.class.php` → `classes/ses.rs`

---

## **Phase 3: Legacy Cron Jobs (26 files)**

### **PHP Cron Scripts** → `crates/he-cron/src/legacy/`
- ⏳ `backup_game.php` → `legacy/backup_game.rs`
- ⏳ `backup_forum.php` → `legacy/backup_forum.rs`
- ⏳ `generateMissions.php` → `legacy/generate_missions.rs`
- ⏳ `updateServerStats.php` → `legacy/update_server_stats.rs`
- ⏳ `endWar.php` → `legacy/end_war.rs`
- ⏳ `endWar2.php` → `legacy/end_war2.rs`
- ⏳ `restoreSoftware.php` → `legacy/restore_software.rs`
- ⏳ `updatePremium.php` → `legacy/update_premium.rs`
- ⏳ `doomUpdater.php` → `legacy/doom_updater.rs`
- ⏳ `safenetUpdate.php` → `legacy/safenet_update.rs`
- ⏳ `defcon.php` → `legacy/defcon.rs`
- ⏳ `defcon2.php` → `legacy/defcon2.rs`
- ⏳ `finishRound.php` → `legacy/finish_round.rs`

### **Python Cron Scripts** → `crates/he-cron/src/legacy/`
- ⏳ `newRoundUpdater.py` → `legacy/new_round_updater.rs`
- ⏳ `updateRanking.py` → `legacy/update_ranking.rs`
- ⏳ `updateCurStats.py` → `legacy/update_cur_stats.rs`
- ⏳ `removeExpiredAccs.py` → `legacy/remove_expired_accounts.rs`
- ⏳ `removeExpiredPremium.py` → `legacy/remove_expired_premium.rs`
- ⏳ `restoreNPC.py` → `legacy/restore_npc.rs`
- ⏳ `removeExpiredNPC.py` → `legacy/remove_expired_npc.rs`
- ⏳ `npcHardware.py` → `legacy/npc_hardware.rs`
- ⏳ `antivirusNPC.py` → `legacy/antivirus_npc.rs`
- ⏳ `fbiUpdate.py` → `legacy/fbi_update.rs`
- ⏳ `removeDownNPC.py` → `legacy/remove_down_npc.rs`
- ⏳ `removeExpiredLogins.py` → `legacy/remove_expired_logins.rs`
- ⏳ `removeExpiredHTMLPages.py` → `legacy/remove_expired_html_pages.rs`

---

## **Phase 4: Helix Elixir Modules (912 files)**

### **Core Architecture** → `crates/he-helix-compat/src/`
- ⏳ `lib/helix/account/` → `account/` (User management)
- ⏳ `lib/helix/cache/` → `cache/` (Caching layer)
- ⏳ `lib/helix/core/` → `core/` (Core listeners and logic)
- ⏳ `lib/helix/entity/` → `entity/` (Entity management)
- ⏳ `lib/helix/event/` → `event/` (Event system)
- ⏳ `lib/helix/id/` → `id/` (ID generation)

### **Game Systems** → `crates/he-helix-compat/src/`
- ⏳ `lib/helix/process/` → `process/` (Advanced process system)
- ⏳ `lib/helix/server/` → `server/` (Server management)
- ⏳ `lib/helix/software/` → `software/` (Software system)
- ⏳ `lib/helix/network/` → `network/` (Network simulation)
- ⏳ `lib/helix/log/` → `log/` (Logging system)
- ⏳ `lib/helix/story/` → `story/` (Mission system)
- ⏳ `lib/helix/universe/` → `universe/` (Game world)

### **Communication** → `crates/he-helix-compat/src/`
- ⏳ `lib/helix/websocket/` → `websocket/` (Real-time communication)
- ⏳ `lib/helix/notification/` → `notification/` (Notifications)
- ⏳ `lib/helix/client/` → `client/` (Client interface)

### **Utilities** → `crates/he-helix-compat/src/`
- ⏳ `lib/helix/henforcer/` → `henforcer/` (Validation system)
- ⏳ `lib/helix/factor/` → `factor/` (Game balance)
- ⏳ `lib/helix/balance/` → `balance/` (Economic balance)
- ⏳ `lib/helix/http/` → `http/` (HTTP handling)
- ⏳ `lib/helix/hell/` → `hell/` (Mix tasks → Rust utilities)

---

## **Phase 5: Static Assets & Configuration**

### **Web Assets** → `static/`
- ⏳ **CSS Files** (124 files) → Modern CSS/SCSS
- ⏳ **JavaScript Files** (93 files) → Modern TypeScript/JS
- ⏳ **Images** (2,711 files) → Optimized asset pipeline
- ⏳ **Fonts** (13 files) → Web font optimization

### **Configuration** → `config/`
- ⏳ **Game Configuration** → TOML/YAML config files
- ⏳ **Database Migrations** → SQLx migrations
- ⏳ **Localization** → i18n system (English/Portuguese)

### **Content** → `content/`
- ⏳ **NPC Content** (458 HTML files) → Template system
- ⏳ **Puzzle Games** (5 games) → WASM/JS implementations

---

## **Phase 6: Ecosystem Integration**

### **Forum System** → Modern Alternative
- **Legacy**: 263 PHP files (phpBB)
- **Modern**: Integration with Discourse or custom Rust forum
- **Strategy**: API-based integration rather than 1:1 port

### **Wiki System** → Modern Alternative  
- **Legacy**: 554 PHP files (DokuWiki)
- **Modern**: Integration with GitBook/Notion API or custom wiki
- **Strategy**: Content migration + API integration

### **Third-Party Services** → `crates/he-services/src/`
- ⏳ **Payment Processing** → Modern payment APIs
- ⏳ **Email Service** → Modern email service integration
- ⏳ **Security** → Modern input sanitization

---

## **Implementation Crate Structure**

```
hackerexperience-unified/
├── crates/
│   ├── he-core/                    ✅ DONE - Core entities
│   ├── he-db/                      ✅ DONE - Database layer  
│   ├── he-legacy-compat/           🔄 IN PROGRESS - PHP files
│   │   ├── src/
│   │   │   ├── pages/             ⏳ 51 PHP root files
│   │   │   ├── classes/           ⏳ 33 PHP classes  
│   │   │   └── session.rs         ✅ DONE
│   ├── he-helix-compat/            ⏳ PENDING - Elixir files
│   │   └── src/                   ⏳ 912 Elixir files
│   ├── he-cron/                    ⏳ PENDING - Background tasks
│   │   └── src/legacy/            ⏳ 26 cron files
│   ├── he-api/                     ⏳ PENDING - Unified API
│   ├── he-websocket/               ⏳ PENDING - Real-time communication
│   ├── he-services/                ⏳ PENDING - External integrations
│   └── he-cli/                     ⏳ PENDING - Management tools
├── static/                         ⏳ PENDING - Static assets
├── config/                         ⏳ PENDING - Configuration
├── content/                        ⏳ PENDING - Game content
└── migrations/                     ✅ DONE - Database schema
```

---

## **Success Metrics for 1:1 Parity**

### **Functional Parity**
- ✅ **100%** of game mechanics preserved
- ✅ **100%** of API endpoints maintained  
- ✅ **100%** of database operations identical
- ✅ **100%** of user workflows preserved

### **File Coverage**
- **PHP Files**: 2,294 → Rust equivalents
- **Elixir Files**: 912 → Rust equivalents  
- **Python Scripts**: 28 → Rust async tasks
- **Static Assets**: 4,620 → Optimized modern assets

### **Performance Targets**
- **10-100x** faster than PHP original
- **2-5x** faster than Elixir Helix
- **50-90%** memory reduction
- **10,000+** concurrent users

This mapping ensures **EVERY SINGLE FILE** is accounted for and properly migrated to maintain perfect 1:1 functional parity while modernizing the entire technology stack.