// PHP Classes ported to Rust - 1:1 functionality preservation
// Original: 30+ PHP classes with complex game mechanics

pub mod database;     // Database.class.php - Core system operations (LRSys)
pub mod clan;         // Clan.class.php - Complex clan system with wars  
pub mod ranking;      // Ranking.class.php - Player stats and certifications
pub mod finances;     // Finances.class.php - Banking and Bitcoin system
pub mod system;       // System.class.php - Core utilities
pub mod bcrypt;       // BCrypt.class.php - Password hashing
pub mod social;       // Social.class.php - Social features
pub mod storyline;    // Storyline.class.php - Mission system
pub mod npc;          // NPC.class.php - NPC management
pub mod forum;        // Forum.class.php - phpBB integration
pub mod mission;      // Mission.class.php - Mission mechanics
pub mod internet;     // Internet.class.php - Network simulation
pub mod mail;         // Mail.class.php - Messaging system
pub mod news;         // News.class.php - News system
pub mod python;       // Python.class.php - Python script interface
pub mod facebook;     // Facebook.class.php - Social login
pub mod remember_me;  // RememberMe.class.php - Session persistence
pub mod premium;      // Premium.class.php - Premium features
pub mod purifier;     // Purifier.class.php - HTML sanitization
pub mod images;       // Images.class.php - Image handling
pub mod fame;         // Fame.class.php - Hall of fame
pub mod list;         // List.class.php - Hacked database
pub mod riddle;       // Riddle.class.php - Puzzle system
pub mod pagination;   // Pagination.class.php - Pagination utilities
pub mod versioning;   // Versioning.class.php - Version control
pub mod ses;          // SES.class.php - Amazon SES integration
pub mod email_verification; // EmailVerification.class.php - Email verification
pub mod software;       // Software.class.php - Software management
pub mod player;         // Player.class.php - Player operations
pub mod player_display; // Player display methods - HTML rendering and UI

// Re-export main classes for easy access
pub use database::*;
pub use clan::*;
pub use ranking::*;
pub use finances::*;
pub use system::*;
pub use bcrypt::*;
pub use social::*;
pub use storyline::*;
pub use npc::*;
pub use mail::*;
pub use news::*;
pub use forum::*;
pub use mission::*;
pub use software::*;
pub use player::*;

// TODO: Implement remaining classes as needed
// Priority order based on dependency analysis:
// 1. system - Core utilities (used by many classes)
// 2. bcrypt - Password hashing (security critical)
// 3. social - Social features
// 4. storyline - Mission system  
// 5. npc - NPC management
// 6. forum - phpBB integration
// 7. mission - Mission mechanics
// 8. internet - Network simulation
// 9. mail - Messaging system
// 10. news - News system
// 11. python - Python script interface
// 12. facebook - Social login
// 13. remember_me - Session persistence
// 14. premium - Premium features
// 15. purifier - HTML sanitization
// 16. images - Image handling
// 17. fame - Hall of fame
// 18. list - Hacked database
// 19. riddle - Puzzle system
// 20. pagination - Pagination utilities
// 21. versioning - Version control
// 22. ses - Amazon SES integration
// 23. email_verification - Email verification