# Remaining Legacy PHP Files Analysis

## Already Ported (22 files):
✅ ajax.php → ajax.rs  
✅ clan.php → clan.rs  
✅ finances.php → finances.rs  
✅ hardware.php → hardware.rs  
✅ index.php → index.rs  
✅ internet.php → internet.rs  
✅ login.php → login.rs  
✅ logout.php → logout.rs  
✅ mail.php → mail.rs  
✅ missions.php → missions.rs  
✅ news.php → news.rs  
✅ processes.php → processes.rs  
✅ profile.php → profile.rs  
✅ ranking.php → ranking.rs  
✅ register.php → register.rs  
✅ research.php → research.rs  
✅ reset.php → reset.rs  
✅ settings.php → settings.rs  
✅ software.php → software.rs  
✅ stats.php → stats.rs  
✅ university.php → university.rs  

## Remaining to Port (27 files):

### High Priority Game Pages (10 files):
1. **DDoS.php** → ddos.rs (DDoS attack system)
2. **createsoft.php** → create_software.rs (Software creation)
3. **doom.php** → doom.rs (Special game mode)
4. **hardwareItens.php** → hardware_items.rs (Hardware inventory)
5. **list.php** → list.rs (Database list viewer) 
6. **log.php** → log.rs (Log viewer)
7. **logEdit.php** → log_edit.rs (Log editing)
8. **researchTable.php** → research_table.rs (Research display)
9. **war.php** → war.rs (Clan war system)
10. **webserver.php** → webserver.rs (Web server management)

### User Experience Pages (7 files):
11. **about.php** → about.rs (About page)
12. **changelog.php** → changelog.rs (Game changelog)
13. **fame.php** → fame.rs (Hall of fame)
14. **gameInfo.php** → game_info.rs (Game information)
15. **options.php** → options.rs (User options)
16. **riddle.php** → riddle.rs (Puzzle system)
17. **stats_1.php** → stats_detailed.rs (Detailed stats)

### Legal and Information (4 files):
18. **TOS.php** → tos.rs (Terms of Service)
19. **legal.php** → legal.rs (Legal information)
20. **privacy.php** → privacy.rs (Privacy policy)
21. **welcome.php** → welcome.rs (Welcome page)

### Payment and Premium (3 files):
22. **bitcoin.php** → bitcoin.rs (Bitcoin payment)
23. **pagarme.php** → pagarme.rs (Payment gateway)
24. **premium.php** → premium.rs (Premium features)

### Utility and Special (3 files):
25. **uploadImage.php** → upload_image.rs (Image upload)
26. **connect.php** → connect.rs (Connection handling)
27. **resetIP.php** → reset_ip.rs (IP reset functionality)

### Configuration Files (Not for porting):
- **config.php** (Database config - already handled in Rust)
- **badge_config.php** (Badge configuration - data only)
- **certs.php** (Certificates config - data only)

## Porting Priority Order:

### Phase 1: Core Game Features (10 files)
Focus on essential gameplay mechanics that users interact with most:
- createsoft.php (Software creation workflow)
- hardwareItens.php (Hardware inventory management) 
- log.php & logEdit.php (Log system for covering tracks)
- war.php (Clan warfare - major social feature)
- webserver.php (Web server hosting and management)
- DDoS.php (Attack system)
- list.php (Database exploration)
- researchTable.php (Research progress display)
- doom.php (Special game mode)

### Phase 2: User Experience (7 files)
Improve user engagement and information:
- about.php, changelog.php, gameInfo.php
- fame.php (Recognition system)
- options.php (User preferences)
- riddle.php (Entertainment/tutorial)
- stats_1.php (Advanced statistics)

### Phase 3: Legal and Business (7 files)
Complete the platform:
- TOS.php, legal.php, privacy.php
- bitcoin.php, pagarme.php, premium.php
- welcome.php

### Phase 4: Utilities (3 files)
Support features:
- uploadImage.php (File handling)
- connect.php (Connection management)
- resetIP.php (IP management)

Total: **27 remaining PHP files** to achieve complete 1:1 parity.