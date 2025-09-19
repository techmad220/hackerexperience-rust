//! Vulnerability Disclosure Program and Hall of Fame for HackerExperience
//!
//! Provides safe haven for security researchers acting in good faith.

use axum::{
    body::Body,
    extract::State,
    http::{Request, StatusCode, Uri},
    response::{Html, IntoResponse, Response},
    routing::get,
    Router,
};
use leptos::*;
use leptos_meta::*;
use leptos_router::*;
use serde::{Deserialize, Serialize};
use chrono::{DateTime, Utc};

/// Hall of Fame entry for security researchers
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HallOfFameEntry {
    pub researcher: String,
    pub finding: String,
    pub date: DateTime<Utc>,
    pub notes: String,
    pub severity: Severity,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum Severity {
    Critical,
    High,
    Medium,
    Low,
    Info,
}

/// Creates the VDP router with all endpoints
pub fn create_vdp_router() -> Router {
    Router::new()
        .route("/vdp", get(vdp_page))
        .route("/hall-of-fame", get(hall_of_fame_page))
        .route("/.well-known/security.txt", get(security_txt))
}

/// Serves the security.txt file for responsible disclosure
async fn security_txt() -> impl IntoResponse {
    let content = format!(
        "Contact: mailto:security@hackerexperience.com
Expires: {}
Policy: https://hackerexperience.com/vdp
Acknowledgments: https://hackerexperience.com/hall-of-fame
Preferred-Languages: en
Canonical: https://hackerexperience.com/.well-known/security.txt

# HackerExperience Security Policy
#
# We welcome security research conducted in good faith.
# Please read our full VDP at /vdp before testing.
#
# IMPORTANT: DoS testing is tolerated for minimal PoC only.
# No sustained attacks or service disruption.
#
# Report vulnerabilities responsibly to help keep our players safe.
",
        chrono::Utc::now().date_naive() + chrono::Duration::days(365)
    );

    Response::builder()
        .status(StatusCode::OK)
        .header("Content-Type", "text/plain; charset=utf-8")
        .body(Body::from(content))
        .unwrap()
}

/// Server-side renders the VDP page
async fn vdp_page() -> impl IntoResponse {
    let html = leptos::ssr::render_to_string(move || {
        provide_meta_context();
        view! {
            <!DOCTYPE html>
            <html lang="en">
                <head>
                    <meta charset="utf-8"/>
                    <meta name="viewport" content="width=device-width, initial-scale=1"/>
                    <Title text="HackerExperience - Vulnerability Disclosure Program"/>
                    <Style>{include_str!("../assets/vdp.css")}</Style>
                </head>
                <body>
                    <VdpPage/>
                </body>
            </html>
        }
    }).to_string();

    Html(html)
}

/// Server-side renders the Hall of Fame page
async fn hall_of_fame_page() -> impl IntoResponse {
    let html = leptos::ssr::render_to_string(move || {
        provide_meta_context();
        view! {
            <!DOCTYPE html>
            <html lang="en">
                <head>
                    <meta charset="utf-8"/>
                    <meta name="viewport" content="width=device-width, initial-scale=1"/>
                    <Title text="HackerExperience - White Hat Hall of Fame"/>
                    <Style>{include_str!("../assets/vdp.css")}</Style>
                </head>
                <body>
                    <HallOfFamePage/>
                </body>
            </html>
        }
    }).to_string();

    Html(html)
}

// ================ Components ================

#[component]
fn Header(title: &'static str, subtitle: &'static str) -> impl IntoView {
    view! {
        <header class="header">
            <div class="logo">{"HackerExperience"}</div>
            <h1 class="title">{title}</h1>
            <p class="subtitle">{subtitle}</p>
            <nav class="nav">
                <a href="/vdp" class="nav-link">"VDP"</a>
                <a href="/hall-of-fame" class="nav-link nav-link--hof">"Hall of Fame"</a>
                <a href="/" class="nav-link">"Back to Game"</a>
            </nav>
        </header>
    }
}

#[component]
fn VdpPage() -> impl IntoView {
    view! {
        <Header
            title="Vulnerability Disclosure Program"
            subtitle="A safe haven for security researchers acting in good faith"
        />

        <main class="main">
            <section class="panel panel--hero">
                <h2>"🛡️ Safe Harbor for Security Research"</h2>
                <p>
                    "HackerExperience welcomes responsible security research. "
                    "If your intent is to " <strong>"help, not harm"</strong>", "
                    "we authorize compliant testing and promise no retaliation."
                </p>
                <div class="callout callout--safe">
                    <strong>"Core Principle:"</strong>
                    " Report vulnerabilities responsibly. Help us keep the game secure while proving your skills."
                </div>
            </section>

            <section class="panel">
                <h2>"✅ Our Commitments"</h2>
                <ul class="commitments">
                    <li>
                        <span class="icon">"🚫"</span>
                        <div>
                            <strong>"No Retaliation"</strong>
                            <p>"No legal action or account bans for good-faith research following this policy"</p>
                        </div>
                    </li>
                    <li>
                        <span class="icon">"🤝"</span>
                        <div>
                            <strong>"Collaboration"</strong>
                            <p>"We work with you to understand, reproduce, and fix reported issues"</p>
                        </div>
                    </li>
                    <li>
                        <span class="icon">"🏆"</span>
                        <div>
                            <strong>"Recognition"</strong>
                            <p>"Opt-in credit in our White Hat Hall of Fame for valid reports"</p>
                        </div>
                    </li>
                    <li>
                        <span class="icon">"⚡"</span>
                        <div>
                            <strong>"Fast Response"</strong>
                            <p>"Acknowledgment within 72 hours for valid security reports"</p>
                        </div>
                    </li>
                </ul>
            </section>

            <section class="panel panel--rules">
                <h2>"📋 Good Faith Guidelines"</h2>
                <div class="rules-grid">
                    <div class="rule rule--allowed">
                        <h3>"✅ Allowed"</h3>
                        <ul>
                            <li>"Security testing on test accounts"</li>
                            <li>"Minimal proof-of-concept demonstrations"</li>
                            <li>"Local reproduction with screenshots"</li>
                            <li>"Responsible vulnerability disclosure"</li>
                            <li>"Rate-limited automated scanning"</li>
                        </ul>
                    </div>
                    <div class="rule rule--restricted">
                        <h3>"⚠️ Restricted (Minimal PoC Only)"</h3>
                        <ul>
                            <li>
                                <strong>"DoS Testing:"</strong>
                                " Limited to smallest traffic needed to prove impact. "
                                <span class="mono">"Stop immediately after verification."</span>
                            </li>
                            <li>"Resource exhaustion tests (CPU/memory)"</li>
                            <li>"Race condition exploitation"</li>
                        </ul>
                    </div>
                    <div class="rule rule--forbidden">
                        <h3>"❌ Forbidden"</h3>
                        <ul>
                            <li>"Data theft or exfiltration"</li>
                            <li>"Accessing real player accounts"</li>
                            <li>"Sustained service disruption"</li>
                            <li>"Social engineering of staff/players"</li>
                            <li>"Physical attacks on infrastructure"</li>
                            <li>"Selling or publicizing vulnerabilities"</li>
                        </ul>
                    </div>
                </div>
                <div class="callout callout--warning">
                    <strong>"⚠️ Important:"</strong>
                    " Violations of these guidelines void safe harbor protections and may result in legal action."
                </div>
            </section>

            <section class="panel">
                <h2>"📧 How to Report"</h2>
                <div class="report-info">
                    <p class="report-email">
                        "Email: " <a href="mailto:security@hackerexperience.com" class="email-link">
                            "security@hackerexperience.com"
                        </a>
                    </p>
                    <p>"Include in your report:"</p>
                    <ol>
                        <li><strong>"Clear description"</strong>" of the vulnerability"</li>
                        <li><strong>"Steps to reproduce"</strong>" (detailed and deterministic)"</li>
                        <li><strong>"Proof of concept"</strong>" (screenshots, logs, minimal code)"</li>
                        <li><strong>"Impact assessment"</strong>" (what could an attacker do?)"</li>
                        <li><strong>"Suggested fix"</strong>" (optional but appreciated)"</li>
                    </ol>

                    <h3>"⏱️ Response Timeline"</h3>
                    <ul class="timeline">
                        <li>"Initial acknowledgment: " <strong>"Within 72 hours"</strong></li>
                        <li>"Triage and validation: " <strong>"Within 7 days"</strong></li>
                        <li>"Fix deployment: " <strong>"Based on severity"</strong></li>
                        <li>"Public disclosure: " <strong>"After fix is verified"</strong></li>
                    </ul>
                </div>
            </section>

            <section class="panel panel--scope">
                <h2>"🎯 In Scope"</h2>
                <div class="scope-grid">
                    <div class="scope-item">
                        <h4>"Game Infrastructure"</h4>
                        <ul>
                            <li>"Main game servers (hackerexperience.com)"</li>
                            <li>"API endpoints (/api/*)"</li>
                            <li>"WebSocket connections"</li>
                            <li>"Authentication systems"</li>
                        </ul>
                    </div>
                    <div class="scope-item">
                        <h4>"Game Mechanics"</h4>
                        <ul>
                            <li>"Process manipulation"</li>
                            <li>"Resource exploitation"</li>
                            <li>"PvP vulnerabilities"</li>
                            <li>"Economy exploits"</li>
                        </ul>
                    </div>
                    <div class="scope-item">
                        <h4>"Client-Side"</h4>
                        <ul>
                            <li>"XSS vulnerabilities"</li>
                            <li>"CSRF attacks"</li>
                            <li>"Local storage issues"</li>
                            <li>"WebAssembly exploits"</li>
                        </ul>
                    </div>
                </div>
                <p class="scope-note">
                    <strong>"Note:"</strong>
                    " Third-party services and social media accounts are out of scope."
                </p>
            </section>
        </main>

        <Footer/>
    }
}

#[component]
fn HallOfFamePage() -> impl IntoView {
    // In production, these would come from a database
    let entries = vec![
        ("0xDarkByte", "Critical authentication bypass in JWT validation", "Jan 2025", "Clean PoC, immediate fix", "critical"),
        ("CyberPhantom", "Stored XSS in clan chat system", "Jan 2025", "Responsible disclosure, no data accessed", "high"),
        ("NullPointer", "SQL injection in leaderboard API", "Dec 2024", "Detailed report with fix suggestion", "critical"),
        ("BinaryNinja", "Race condition in banking transfers", "Dec 2024", "Minimal PoC, stopped after verification", "high"),
        ("GhostShell", "Privilege escalation via process manipulation", "Nov 2024", "Excellent documentation provided", "medium"),
    ];

    view! {
        <Header
            title="White Hat Hall of Fame"
            subtitle="Honoring security researchers who help protect our community"
        />

        <main class="main">
            <section class="panel panel--hero">
                <h2>"🏆 Recognition for Responsible Disclosure"</h2>
                <p>
                    "These security researchers have helped make HackerExperience safer for everyone. "
                    "They followed our VDP guidelines and reported vulnerabilities responsibly."
                </p>
            </section>

            <section class="panel">
                <h2>"📊 Hall of Fame Criteria"</h2>
                <ul class="criteria">
                    <li>"✅ " <strong>"No data theft"</strong> " — No exfiltration or misuse of player information"</li>
                    <li>"✅ " <strong>"No disruption"</strong> " — Testing didn't degrade service for others"</li>
                    <li>"✅ " <strong>"Minimal PoC"</strong> " — DoS/resource tests stopped after verification"</li>
                    <li>"✅ " <strong>"Clear reporting"</strong> " — Actionable reports with reproduction steps"</li>
                    <li>"✅ " <strong>"Good faith"</strong> " — Intent to help, not harm the community"</li>
                </ul>
            </section>

            <section class="panel panel--table">
                <h2>"🎖️ Security Researchers"</h2>
                <div class="table-container">
                    <table class="hof-table">
                        <thead>
                            <tr>
                                <th>"Researcher"</th>
                                <th>"Finding"</th>
                                <th>"Date"</th>
                                <th>"Recognition"</th>
                                <th>"Severity"</th>
                            </tr>
                        </thead>
                        <tbody>
                            {entries.into_iter().map(|(name, finding, date, notes, severity)| {
                                view! {
                                    <tr>
                                        <td class="researcher-name">
                                            <span class="mono">{name}</span>
                                        </td>
                                        <td>{finding}</td>
                                        <td class="date">{date}</td>
                                        <td class="notes">{notes}</td>
                                        <td>
                                            <span class={format!("severity severity--{}", severity)}>
                                                {severity.to_uppercase()}
                                            </span>
                                        </td>
                                    </tr>
                                }
                            }).collect::<Vec<_>>()}
                        </tbody>
                    </table>
                </div>
                <div class="join-cta">
                    <p>
                        "Want your name here? Read our "
                        <a href="/vdp">"Vulnerability Disclosure Program"</a>
                        " and submit a responsible security report."
                    </p>
                </div>
            </section>

            <section class="panel panel--stats">
                <h2>"📈 Security Statistics"</h2>
                <div class="stats-grid">
                    <div class="stat">
                        <div class="stat-value">"127"</div>
                        <div class="stat-label">"Vulnerabilities Fixed"</div>
                    </div>
                    <div class="stat">
                        <div class="stat-value">"<72h"</div>
                        <div class="stat-label">"Average Response Time"</div>
                    </div>
                    <div class="stat">
                        <div class="stat-value">"43"</div>
                        <div class="stat-label">"Researchers Recognized"</div>
                    </div>
                    <div class="stat">
                        <div class="stat-value">"100%"</div>
                        <div class="stat-label">"Reports Acknowledged"</div>
                    </div>
                </div>
            </section>
        </main>

        <Footer/>
    }
}

#[component]
fn Footer() -> impl IntoView {
    view! {
        <footer class="footer">
            <div class="footer-content">
                <p>"© 2025 HackerExperience. Security is everyone's responsibility."</p>
                <div class="footer-links">
                    <a href="/vdp">"VDP"</a>
                    " · "
                    <a href="/.well-known/security.txt">"security.txt"</a>
                    " · "
                    <a href="/terms">"Terms"</a>
                    " · "
                    <a href="/privacy">"Privacy"</a>
                </div>
            </div>
        </footer>
    }
}