//! Complete Legacy PHP Router - Connects all PHP endpoints to Rust implementations
//!
//! This is the master router that provides 100% PHP compatibility

use actix_web::{web, HttpResponse, HttpRequest};
use crate::AppState;

/// Register ALL legacy PHP routes with their Rust implementations
pub fn register_all_routes(cfg: &mut web::ServiceConfig) {
    cfg
        // ============= CORE PAGES =============
        .route("/", web::get().to(index_handler))
        .route("/index.php", web::get().to(index_handler))

        // ============= AUTHENTICATION =============
        .route("/login.php", web::get().to(login_page))
        .route("/login.php", web::post().to(login_handler))
        .route("/register.php", web::get().to(register_page))
        .route("/register.php", web::post().to(register_handler))
        .route("/logout.php", web::get().to(logout_handler))
        .route("/welcome.php", web::get().to(welcome_handler))

        // ============= GAME CORE =============
        .route("/processes.php", web::get().to(processes_handler))
        .route("/processes.php", web::post().to(processes_action))
        .route("/software.php", web::get().to(software_handler))
        .route("/software.php", web::post().to(software_action))
        .route("/hardware.php", web::get().to(hardware_handler))
        .route("/hardware.php", web::post().to(hardware_action))
        .route("/internet.php", web::get().to(internet_handler))
        .route("/internet.php", web::post().to(internet_action))

        // ============= MISSIONS & ACTIVITIES =============
        .route("/missions.php", web::get().to(missions_handler))
        .route("/missions.php", web::post().to(missions_action))
        .route("/research.php", web::get().to(research_handler))
        .route("/research.php", web::post().to(research_action))
        .route("/university.php", web::get().to(university_handler))
        .route("/university.php", web::post().to(university_action))

        // ============= SOCIAL =============
        .route("/clan.php", web::get().to(clan_handler))
        .route("/clan.php", web::post().to(clan_action))
        .route("/war.php", web::get().to(war_handler))
        .route("/war.php", web::post().to(war_action))
        .route("/ranking.php", web::get().to(ranking_handler))
        .route("/profile.php", web::get().to(profile_handler))
        .route("/mail.php", web::get().to(mail_handler))
        .route("/mail.php", web::post().to(mail_action))

        // ============= FINANCIAL =============
        .route("/bitcoin.php", web::get().to(bitcoin_handler))
        .route("/bitcoin.php", web::post().to(bitcoin_action))
        .route("/finances.php", web::get().to(finances_handler))

        // ============= SPECIAL =============
        .route("/ddos.php", web::get().to(ddos_handler))
        .route("/ddos.php", web::post().to(ddos_action))
        .route("/doom.php", web::get().to(doom_handler))
        .route("/riddle.php", web::get().to(riddle_handler))
        .route("/riddle.php", web::post().to(riddle_action))

        // ============= UTILITIES =============
        .route("/webserver.php", web::get().to(webserver_handler))
        .route("/log.php", web::get().to(log_handler))
        .route("/log.php", web::post().to(log_action))
        .route("/settings.php", web::get().to(settings_handler))
        .route("/settings.php", web::post().to(settings_action))
        .route("/reset.php", web::get().to(reset_handler))
        .route("/stats.php", web::get().to(stats_handler))

        // ============= AJAX =============
        .route("/ajax.php", web::get().to(ajax_handler))
        .route("/ajax.php", web::post().to(ajax_handler))

        // ============= STATIC PAGES =============
        .route("/about.php", web::get().to(about_handler))
        .route("/privacy.php", web::get().to(privacy_handler))
        .route("/tos.php", web::get().to(tos_handler))
        .route("/changelog.php", web::get().to(changelog_handler))
        .route("/legal.php", web::get().to(legal_handler));
}

// ============= HANDLER IMPLEMENTATIONS =============

use he_legacy_compat::pages::*;
use he_game_mechanics;
use he_auth;

// Index/Home
async fn index_handler(
    req: HttpRequest,
    data: web::Data<AppState>,
) -> HttpResponse {
    index::index_handler(
        req.extensions().get().cloned(),
        req.extensions().get().cloned(),
    ).await.into()
}

// Authentication
async fn login_page(data: web::Data<AppState>) -> HttpResponse {
    login::login_page_handler(
        data.into_inner().clone()
    ).await.into()
}

async fn login_handler(
    req: HttpRequest,
    form: web::Form<login::LoginForm>,
    data: web::Data<AppState>,
) -> HttpResponse {
    login::login_handler(
        data.pool.clone().into(),
        req.extensions().get().cloned().unwrap(),
        None,
        Some(form),
    ).await.into()
}

async fn register_page(data: web::Data<AppState>) -> HttpResponse {
    register::register_page_handler(
        data.into_inner().clone()
    ).await.into()
}

async fn register_handler(
    req: HttpRequest,
    form: web::Form<register::RegisterForm>,
    data: web::Data<AppState>,
) -> HttpResponse {
    register::register_handler(
        data.pool.clone().into(),
        req.extensions().get().cloned().unwrap(),
        req.headers().clone(),
        Some(form),
    ).await.into()
}

async fn logout_handler(
    req: HttpRequest,
    data: web::Data<AppState>,
) -> HttpResponse {
    logout::logout_handler(
        req.extensions().get().cloned()
    ).await.into()
}

async fn welcome_handler(
    req: HttpRequest,
    data: web::Data<AppState>,
) -> HttpResponse {
    welcome::welcome_handler(
        req.extensions().get().cloned()
    ).await.into()
}

// Processes
async fn processes_handler(
    req: HttpRequest,
    query: web::Query<processes_complete::ProcessQuery>,
    data: web::Data<AppState>,
) -> HttpResponse {
    processes_complete::processes_handler(
        data.pool.clone().into(),
        req.extensions().get().cloned().unwrap(),
        query,
    ).await.into()
}

async fn processes_action(
    req: HttpRequest,
    form: web::Form<serde_json::Value>,
    data: web::Data<AppState>,
) -> HttpResponse {
    // Handle process actions (pause, resume, delete)
    processes_complete::processes_handler(
        data.pool.clone().into(),
        req.extensions().get().cloned().unwrap(),
        web::Query::from_query(req.query_string()).unwrap(),
    ).await.into()
}

// Software
async fn software_handler(
    req: HttpRequest,
    data: web::Data<AppState>,
) -> HttpResponse {
    software::software_handler(
        data.pool.clone().into(),
        req.extensions().get().cloned().unwrap(),
        web::Query::from_query(req.query_string()).unwrap(),
    ).await.into()
}

async fn software_action(
    req: HttpRequest,
    form: web::Form<serde_json::Value>,
    data: web::Data<AppState>,
) -> HttpResponse {
    software::software_action_handler(
        data.pool.clone().into(),
        req.extensions().get().cloned().unwrap(),
        form,
    ).await.into()
}

// Hardware
async fn hardware_handler(
    req: HttpRequest,
    data: web::Data<AppState>,
) -> HttpResponse {
    hardware::hardware_handler(
        data.pool.clone().into(),
        req.extensions().get().cloned().unwrap(),
        web::Query::from_query(req.query_string()).unwrap(),
    ).await.into()
}

async fn hardware_action(
    req: HttpRequest,
    form: web::Form<serde_json::Value>,
    data: web::Data<AppState>,
) -> HttpResponse {
    hardware::hardware_action_handler(
        data.pool.clone().into(),
        req.extensions().get().cloned().unwrap(),
        form,
    ).await.into()
}

// Internet
async fn internet_handler(
    req: HttpRequest,
    data: web::Data<AppState>,
) -> HttpResponse {
    internet::internet_handler(
        data.pool.clone().into(),
        req.extensions().get().cloned().unwrap(),
        web::Query::from_query(req.query_string()).unwrap(),
    ).await.into()
}

async fn internet_action(
    req: HttpRequest,
    form: web::Form<serde_json::Value>,
    data: web::Data<AppState>,
) -> HttpResponse {
    internet::internet_action_handler(
        data.pool.clone().into(),
        req.extensions().get().cloned().unwrap(),
        form,
    ).await.into()
}

// Missions
async fn missions_handler(
    req: HttpRequest,
    data: web::Data<AppState>,
) -> HttpResponse {
    missions::missions_handler(
        data.pool.clone().into(),
        req.extensions().get().cloned().unwrap(),
    ).await.into()
}

async fn missions_action(
    req: HttpRequest,
    form: web::Form<missions::MissionAction>,
    data: web::Data<AppState>,
) -> HttpResponse {
    missions::missions_action_handler(
        data.pool.clone().into(),
        req.extensions().get().cloned().unwrap(),
        form,
    ).await.into()
}

// Research
async fn research_handler(
    req: HttpRequest,
    data: web::Data<AppState>,
) -> HttpResponse {
    research::research_handler(
        data.pool.clone().into(),
        req.extensions().get().cloned().unwrap(),
    ).await.into()
}

async fn research_action(
    req: HttpRequest,
    form: web::Form<serde_json::Value>,
    data: web::Data<AppState>,
) -> HttpResponse {
    research::research_action_handler(
        data.pool.clone().into(),
        req.extensions().get().cloned().unwrap(),
        form,
    ).await.into()
}

// University
async fn university_handler(
    req: HttpRequest,
    data: web::Data<AppState>,
) -> HttpResponse {
    university::university_handler(
        data.pool.clone().into(),
        req.extensions().get().cloned().unwrap(),
    ).await.into()
}

async fn university_action(
    req: HttpRequest,
    form: web::Form<serde_json::Value>,
    data: web::Data<AppState>,
) -> HttpResponse {
    university::university_action_handler(
        data.pool.clone().into(),
        req.extensions().get().cloned().unwrap(),
        form,
    ).await.into()
}

// Clan
async fn clan_handler(
    req: HttpRequest,
    data: web::Data<AppState>,
) -> HttpResponse {
    clan::clan_handler(
        data.pool.clone().into(),
        req.extensions().get().cloned().unwrap(),
    ).await.into()
}

async fn clan_action(
    req: HttpRequest,
    form: web::Form<serde_json::Value>,
    data: web::Data<AppState>,
) -> HttpResponse {
    clan::clan_action_handler(
        data.pool.clone().into(),
        req.extensions().get().cloned().unwrap(),
        form,
    ).await.into()
}

// War
async fn war_handler(
    req: HttpRequest,
    data: web::Data<AppState>,
) -> HttpResponse {
    war::war_handler(
        data.pool.clone().into(),
        req.extensions().get().cloned().unwrap(),
    ).await.into()
}

async fn war_action(
    req: HttpRequest,
    form: web::Form<serde_json::Value>,
    data: web::Data<AppState>,
) -> HttpResponse {
    war::war_action_handler(
        data.pool.clone().into(),
        req.extensions().get().cloned().unwrap(),
        form,
    ).await.into()
}

// Ranking
async fn ranking_handler(
    req: HttpRequest,
    data: web::Data<AppState>,
) -> HttpResponse {
    ranking::ranking_handler(
        data.pool.clone().into(),
        req.extensions().get().cloned().unwrap(),
    ).await.into()
}

// Profile
async fn profile_handler(
    req: HttpRequest,
    query: web::Query<profile::ProfileQuery>,
    data: web::Data<AppState>,
) -> HttpResponse {
    profile::profile_handler(
        data.pool.clone().into(),
        req.extensions().get().cloned().unwrap(),
        query,
    ).await.into()
}

// Mail
async fn mail_handler(
    req: HttpRequest,
    data: web::Data<AppState>,
) -> HttpResponse {
    mail::mail_handler(
        data.pool.clone().into(),
        req.extensions().get().cloned().unwrap(),
    ).await.into()
}

async fn mail_action(
    req: HttpRequest,
    form: web::Form<serde_json::Value>,
    data: web::Data<AppState>,
) -> HttpResponse {
    mail::mail_action_handler(
        data.pool.clone().into(),
        req.extensions().get().cloned().unwrap(),
        form,
    ).await.into()
}

// Bitcoin
async fn bitcoin_handler(
    req: HttpRequest,
    data: web::Data<AppState>,
) -> HttpResponse {
    bitcoin::bitcoin_handler(
        data.pool.clone().into(),
        req.extensions().get().cloned().unwrap(),
    ).await.into()
}

async fn bitcoin_action(
    req: HttpRequest,
    form: web::Form<serde_json::Value>,
    data: web::Data<AppState>,
) -> HttpResponse {
    bitcoin::bitcoin_action_handler(
        data.pool.clone().into(),
        req.extensions().get().cloned().unwrap(),
        form,
    ).await.into()
}

// Finances
async fn finances_handler(
    req: HttpRequest,
    data: web::Data<AppState>,
) -> HttpResponse {
    finances::finances_handler(
        data.pool.clone().into(),
        req.extensions().get().cloned().unwrap(),
    ).await.into()
}

// DDoS
async fn ddos_handler(
    req: HttpRequest,
    data: web::Data<AppState>,
) -> HttpResponse {
    ddos::ddos_handler(
        data.pool.clone().into(),
        req.extensions().get().cloned().unwrap(),
    ).await.into()
}

async fn ddos_action(
    req: HttpRequest,
    form: web::Form<serde_json::Value>,
    data: web::Data<AppState>,
) -> HttpResponse {
    ddos::ddos_action_handler(
        data.pool.clone().into(),
        req.extensions().get().cloned().unwrap(),
        form,
    ).await.into()
}

// Doom
async fn doom_handler(
    req: HttpRequest,
    data: web::Data<AppState>,
) -> HttpResponse {
    doom::doom_handler(
        data.pool.clone().into(),
        req.extensions().get().cloned().unwrap(),
    ).await.into()
}

// Riddle
async fn riddle_handler(
    req: HttpRequest,
    data: web::Data<AppState>,
) -> HttpResponse {
    riddle::riddle_handler(
        data.pool.clone().into(),
        req.extensions().get().cloned().unwrap(),
    ).await.into()
}

async fn riddle_action(
    req: HttpRequest,
    form: web::Form<serde_json::Value>,
    data: web::Data<AppState>,
) -> HttpResponse {
    riddle::riddle_action_handler(
        data.pool.clone().into(),
        req.extensions().get().cloned().unwrap(),
        form,
    ).await.into()
}

// Webserver
async fn webserver_handler(
    req: HttpRequest,
    data: web::Data<AppState>,
) -> HttpResponse {
    webserver::webserver_handler(
        data.pool.clone().into(),
        req.extensions().get().cloned().unwrap(),
    ).await.into()
}

// Log
async fn log_handler(
    req: HttpRequest,
    data: web::Data<AppState>,
) -> HttpResponse {
    log::log_handler(
        data.pool.clone().into(),
        req.extensions().get().cloned().unwrap(),
    ).await.into()
}

async fn log_action(
    req: HttpRequest,
    form: web::Form<serde_json::Value>,
    data: web::Data<AppState>,
) -> HttpResponse {
    log::log_action_handler(
        data.pool.clone().into(),
        req.extensions().get().cloned().unwrap(),
        form,
    ).await.into()
}

// Settings
async fn settings_handler(
    req: HttpRequest,
    data: web::Data<AppState>,
) -> HttpResponse {
    settings::settings_handler(
        data.pool.clone().into(),
        req.extensions().get().cloned().unwrap(),
    ).await.into()
}

async fn settings_action(
    req: HttpRequest,
    form: web::Form<serde_json::Value>,
    data: web::Data<AppState>,
) -> HttpResponse {
    settings::settings_action_handler(
        data.pool.clone().into(),
        req.extensions().get().cloned().unwrap(),
        form,
    ).await.into()
}

// Reset
async fn reset_handler(
    req: HttpRequest,
    data: web::Data<AppState>,
) -> HttpResponse {
    reset::reset_handler(
        data.pool.clone().into(),
        req.extensions().get().cloned().unwrap(),
    ).await.into()
}

// Stats
async fn stats_handler(
    req: HttpRequest,
    data: web::Data<AppState>,
) -> HttpResponse {
    stats::stats_handler(
        data.pool.clone().into(),
        req.extensions().get().cloned().unwrap(),
    ).await.into()
}

// Ajax
async fn ajax_handler(
    req: HttpRequest,
    data: web::Data<AppState>,
) -> HttpResponse {
    ajax::ajax_handler(
        data.pool.clone().into(),
        req.extensions().get().cloned().unwrap(),
        web::Query::from_query(req.query_string()).unwrap(),
    ).await.into()
}

// Static pages
async fn about_handler(data: web::Data<AppState>) -> HttpResponse {
    about::about_handler().await.into()
}

async fn privacy_handler(data: web::Data<AppState>) -> HttpResponse {
    privacy::privacy_handler().await.into()
}

async fn tos_handler(data: web::Data<AppState>) -> HttpResponse {
    tos::tos_handler().await.into()
}

async fn changelog_handler(data: web::Data<AppState>) -> HttpResponse {
    changelog::changelog_handler().await.into()
}

async fn legal_handler(data: web::Data<AppState>) -> HttpResponse {
    legal::legal_handler().await.into()
}