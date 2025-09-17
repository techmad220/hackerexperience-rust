//! Intrusion Detection System (IDS) for HackerExperience

use dashmap::DashMap;
use std::collections::VecDeque;
use std::net::IpAddr;
use std::sync::Arc;
use std::time::{Duration, Instant};
use tracing::{warn, error};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ThreatLevel {
    Low,
    Medium,
    High,
    Critical,
}

#[derive(Debug, Clone)]
pub struct SuspiciousPattern {
    pub pattern_type: String,
    pub occurrences: u32,
    pub first_seen: Instant,
    pub last_seen: Instant,
    pub details: Vec<String>,
}

#[derive(Debug, Clone)]
pub struct ThreatActor {
    pub ip: IpAddr,
    pub threat_score: f64,
    pub patterns: Vec<SuspiciousPattern>,
    pub blocked: bool,
    pub block_expiry: Option<Instant>,
}

pub struct IntrusionDetector {
    actors: Arc<DashMap<IpAddr, ThreatActor>>,
    patterns: Arc<DashMap<String, regex::Regex>>,
    thresholds: IntrusionThresholds,
}

#[derive(Debug, Clone)]
pub struct IntrusionThresholds {
    pub failed_login_threshold: u32,       // Block after N failed logins
    pub rapid_request_threshold: u32,      // Requests per second
    pub sql_injection_score: f64,          // Threat score for SQL injection
    pub xss_attempt_score: f64,            // Threat score for XSS
    pub path_traversal_score: f64,         // Threat score for path traversal
    pub brute_force_score: f64,            // Threat score for brute force
    pub port_scan_score: f64,              // Threat score for port scanning
    pub block_threshold_score: f64,        // Auto-block at this score
    pub block_duration_minutes: u64,       // How long to block
}

impl Default for IntrusionThresholds {
    fn default() -> Self {
        Self {
            failed_login_threshold: 5,
            rapid_request_threshold: 100,
            sql_injection_score: 50.0,
            xss_attempt_score: 30.0,
            path_traversal_score: 40.0,
            brute_force_score: 20.0,
            port_scan_score: 60.0,
            block_threshold_score: 100.0,
            block_duration_minutes: 60,
        }
    }
}

impl IntrusionDetector {
    pub fn new() -> Self {
        let patterns = Arc::new(DashMap::new());

        // Precompile common attack patterns
        patterns.insert(
            "sql_injection".to_string(),
            regex::Regex::new(r"(?i)(union.*select|select.*from|insert.*into|delete.*from|drop.*table|';|--|\bor\b.*=|exec\(|execute\(|script>|<script)").unwrap()
        );

        patterns.insert(
            "xss".to_string(),
            regex::Regex::new(r"(?i)(<script|javascript:|onerror=|onload=|alert\(|prompt\(|confirm\(|<iframe|<object|<embed|<svg)").unwrap()
        );

        patterns.insert(
            "path_traversal".to_string(),
            regex::Regex::new(r"(\.\./|\.\.\%2[fF]|\.\.\\|\.\.\%5[cC]|\.\.;)").unwrap()
        );

        patterns.insert(
            "command_injection".to_string(),
            regex::Regex::new(r"(;|\||&&|\$\(|`|\bwget\b|\bcurl\b|\bsh\b|\bbash\b|\bpython\b|\bperl\b)").unwrap()
        );

        patterns.insert(
            "ldap_injection".to_string(),
            regex::Regex::new(r"[\*\(\)\\\x00]").unwrap()
        );

        Self {
            actors: Arc::new(DashMap::new()),
            patterns,
            thresholds: IntrusionThresholds::default(),
        }
    }

    pub fn check_request(&self, ip: IpAddr, path: &str, params: &str, headers: &str) -> ThreatLevel {
        let mut threat_score = 0.0;
        let now = Instant::now();

        // Check if IP is already blocked
        if let Some(actor) = self.actors.get(&ip) {
            if actor.blocked {
                if let Some(expiry) = actor.block_expiry {
                    if now < expiry {
                        return ThreatLevel::Critical; // Still blocked
                    }
                }
            }
        }

        // Check for attack patterns in request
        let mut detected_patterns = Vec::new();

        for pattern_entry in self.patterns.iter() {
            let pattern_name = pattern_entry.key();
            let regex = pattern_entry.value();

            let mut matches = false;
            let mut match_details = Vec::new();

            if regex.is_match(path) {
                matches = true;
                match_details.push(format!("Path: {}", path));
            }
            if regex.is_match(params) {
                matches = true;
                match_details.push(format!("Params: {}", params));
            }
            if regex.is_match(headers) {
                matches = true;
                match_details.push(format!("Headers: {}", headers));
            }

            if matches {
                detected_patterns.push(SuspiciousPattern {
                    pattern_type: pattern_name.clone(),
                    occurrences: 1,
                    first_seen: now,
                    last_seen: now,
                    details: match_details,
                });

                // Add threat score based on pattern type
                threat_score += match pattern_name.as_str() {
                    "sql_injection" => self.thresholds.sql_injection_score,
                    "xss" => self.thresholds.xss_attempt_score,
                    "path_traversal" => self.thresholds.path_traversal_score,
                    "command_injection" => 70.0,
                    "ldap_injection" => 40.0,
                    _ => 10.0,
                };
            }
        }

        // Update or create threat actor profile
        self.actors.entry(ip)
            .and_modify(|actor| {
                actor.threat_score += threat_score;
                for pattern in &detected_patterns {
                    // Update existing pattern or add new
                    if let Some(existing) = actor.patterns.iter_mut()
                        .find(|p| p.pattern_type == pattern.pattern_type) {
                        existing.occurrences += 1;
                        existing.last_seen = now;
                        existing.details.extend(pattern.details.clone());
                    } else {
                        actor.patterns.push(pattern.clone());
                    }
                }

                // Check if should block
                if actor.threat_score >= self.thresholds.block_threshold_score {
                    actor.blocked = true;
                    actor.block_expiry = Some(now + Duration::from_secs(self.thresholds.block_duration_minutes * 60));
                    warn!("Blocking IP {} due to threat score: {}", ip, actor.threat_score);
                }
            })
            .or_insert_with(|| ThreatActor {
                ip,
                threat_score,
                patterns: detected_patterns,
                blocked: false,
                block_expiry: None,
            });

        // Determine threat level
        self.calculate_threat_level(threat_score)
    }

    pub fn report_failed_login(&self, ip: IpAddr, username: &str) {
        let now = Instant::now();

        self.actors.entry(ip)
            .and_modify(|actor| {
                // Add brute force pattern
                if let Some(pattern) = actor.patterns.iter_mut()
                    .find(|p| p.pattern_type == "brute_force") {
                    pattern.occurrences += 1;
                    pattern.last_seen = now;
                    pattern.details.push(format!("Username: {}", username));
                } else {
                    actor.patterns.push(SuspiciousPattern {
                        pattern_type: "brute_force".to_string(),
                        occurrences: 1,
                        first_seen: now,
                        last_seen: now,
                        details: vec![format!("Username: {}", username)],
                    });
                }

                actor.threat_score += self.thresholds.brute_force_score;

                // Check if exceeded failed login threshold
                if let Some(pattern) = actor.patterns.iter()
                    .find(|p| p.pattern_type == "brute_force") {
                    if pattern.occurrences >= self.thresholds.failed_login_threshold {
                        actor.blocked = true;
                        actor.block_expiry = Some(now + Duration::from_secs(self.thresholds.block_duration_minutes * 60));
                        warn!("Blocking IP {} due to {} failed login attempts", ip, pattern.occurrences);
                    }
                }
            })
            .or_insert_with(|| {
                let mut actor = ThreatActor {
                    ip,
                    threat_score: self.thresholds.brute_force_score,
                    patterns: vec![SuspiciousPattern {
                        pattern_type: "brute_force".to_string(),
                        occurrences: 1,
                        first_seen: now,
                        last_seen: now,
                        details: vec![format!("Username: {}", username)],
                    }],
                    blocked: false,
                    block_expiry: None,
                };
                actor
            });
    }

    pub fn check_rate_anomaly(&self, ip: IpAddr, requests_per_second: f64) -> bool {
        if requests_per_second > self.thresholds.rapid_request_threshold as f64 {
            self.actors.entry(ip)
                .and_modify(|actor| {
                    actor.threat_score += 30.0;
                    actor.patterns.push(SuspiciousPattern {
                        pattern_type: "rate_anomaly".to_string(),
                        occurrences: 1,
                        first_seen: Instant::now(),
                        last_seen: Instant::now(),
                        details: vec![format!("Rate: {} req/s", requests_per_second)],
                    });
                })
                .or_insert_with(|| ThreatActor {
                    ip,
                    threat_score: 30.0,
                    patterns: vec![SuspiciousPattern {
                        pattern_type: "rate_anomaly".to_string(),
                        occurrences: 1,
                        first_seen: Instant::now(),
                        last_seen: Instant::now(),
                        details: vec![format!("Rate: {} req/s", requests_per_second)],
                    }],
                    blocked: false,
                    block_expiry: None,
                });
            return true;
        }
        false
    }

    pub fn is_blocked(&self, ip: IpAddr) -> bool {
        if let Some(actor) = self.actors.get(&ip) {
            if actor.blocked {
                if let Some(expiry) = actor.block_expiry {
                    if Instant::now() < expiry {
                        return true;
                    }
                }
            }
        }
        false
    }

    pub fn get_threat_level(&self, ip: IpAddr) -> ThreatLevel {
        if let Some(actor) = self.actors.get(&ip) {
            self.calculate_threat_level(actor.threat_score)
        } else {
            ThreatLevel::Low
        }
    }

    fn calculate_threat_level(&self, score: f64) -> ThreatLevel {
        if score >= self.thresholds.block_threshold_score {
            ThreatLevel::Critical
        } else if score >= 70.0 {
            ThreatLevel::High
        } else if score >= 40.0 {
            ThreatLevel::Medium
        } else {
            ThreatLevel::Low
        }
    }

    pub fn cleanup_old_actors(&self, age_minutes: u64) {
        let cutoff = Instant::now() - Duration::from_secs(age_minutes * 60);

        self.actors.retain(|_, actor| {
            // Keep if blocked and not expired
            if actor.blocked {
                if let Some(expiry) = actor.block_expiry {
                    return Instant::now() < expiry;
                }
            }

            // Keep if seen recently
            actor.patterns.iter().any(|p| p.last_seen > cutoff)
        });
    }
}