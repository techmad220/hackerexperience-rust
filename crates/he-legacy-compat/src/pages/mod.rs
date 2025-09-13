//! Legacy PHP pages ported to Rust
//! 
//! This module contains 1:1 ports of all PHP root files from the legacy HackerExperience codebase.
//! Each page maintains exact functional parity with the original PHP implementation.

// Core entry points and authentication
pub mod index;
pub mod ajax;
pub mod login;
pub mod register;
pub mod logout;
pub mod reset;
pub mod reset_ip;
pub mod processes;

// Game core pages  
pub mod hardware;
pub mod hardware_items;
pub mod software;
pub mod create_software;
pub mod research;
pub mod research_table;
pub mod mail;
pub mod news;
pub mod finances;
pub mod internet;
pub mod webserver;
pub mod log;
pub mod log_edit;
pub mod missions;
pub mod university;
pub mod war;
pub mod doom;
pub mod ddos;

// User management
pub mod profile;
pub mod settings;
pub mod options;
pub mod stats;
pub mod stats_detailed;
pub mod ranking;
pub mod fame;
pub mod premium;
pub mod bitcoin;
pub mod pagarme;

// Clan & social
pub mod clan;

// Information & legal
pub mod privacy;
pub mod tos;
pub mod legal;
pub mod about;
pub mod changelog;
pub mod game_info;
pub mod riddle;

// Utilities
pub mod upload_image;

// Re-exports for easy access
pub use index::index_handler;
pub use ajax::ajax_handler;
pub use login::login_handler;
pub use register::register_handler;
pub use logout::logout_handler;
pub use reset::reset_handler;
pub use hardware::hardware_handler;
pub use software::software_handler;
pub use finances::finances_handler;
pub use mail::mail_handler;
pub use news::news_handler;
pub use profile::profile_handler;
pub use settings::settings_handler;
pub use stats::stats_handler;
pub use research::research_handler;
pub use missions::missions_handler;
pub use internet::internet_handler;
pub use processes::processes_handler;
pub use university::university_handler;
pub use ranking::ranking_handler;
pub use clan::clan_handler;