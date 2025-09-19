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
pub mod list;
pub mod certs;
pub mod connect;

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
pub mod config;
pub mod welcome;

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

// Certificates and connections
pub mod certs;
pub mod connect;

// Configuration modules
pub mod badge_config;
pub mod config;
pub mod welcome;

// Utilities
pub mod upload_image;
pub mod reset_ip;

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
pub use create_software::create_software_handler;
pub use log::log_handler;
pub use log_edit::log_edit_handler;
pub use research_table::research_table_handler;
pub use webserver::webserver_handler;
pub use list::list_handler;
pub use tos::tos_handler;
pub use about::about_handler;
pub use bitcoin::bitcoin_handler;
pub use certs::certs_handler;
pub use changelog::changelog_handler;
pub use connect::connect_handler;
pub use doom::doom_handler;
pub use fame::fame_handler;
pub use game_info::game_info_handler;
pub use legal::legal_handler;
pub use options::options_handler;
pub use pagarme::pagarme_handler;
pub use premium::premium_handler;
pub use privacy::privacy_handler;
pub use reset_ip::reset_ip_handler;
pub use riddle::riddle_handler;
pub use stats_detailed::stats_detailed_handler;
pub use upload_image::upload_image_handler;
pub use welcome::welcome_handler;
pub use config::config_handler;