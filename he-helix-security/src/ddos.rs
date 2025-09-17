//! Advanced DDoS Protection System

use dashmap::DashMap;
use governor::{Quota, RateLimiter, Jitter};
use governor::clock::DefaultClock;
use governor::state::{InMemoryState, NotKeyed};
use nonzero_ext::*;
use std::collections::VecDeque;
use std::net::IpAddr;
use std::sync::Arc;
use std::time::{Duration, Instant};
use tokio::sync::RwLock;
use tracing::{warn, error, info};

#[derive(Debug, Clone)]
pub struct ConnectionInfo {
    pub ip: IpAddr,
    pub connections: u32,
    pub last_seen: Instant,
    pub syn_count: u32,           // SYN flood detection
    pub incomplete_requests: u32,  // Slowloris detection
    pub user_agent: Option<String>,
    pub suspicious_score: f64,
}

#[derive(Debug, Clone)]
pub struct DDoSMetrics {
    pub total_connections: u64,
    pub connections_per_second: f64,
    pub unique_ips: usize,
    pub blocked_ips: usize,
    pub syn_flood_detected: bool,
    pub slowloris_detected: bool,
    pub amplification_detected: bool,
}

pub struct DDoSProtection {
    // Per-IP rate limiters
    ip_limiters: Arc<DashMap<IpAddr, Arc<RateLimiter<NotKeyed, InMemoryState, DefaultClock>>>>,

    // Global rate limiter
    global_limiter: Arc<RateLimiter<NotKeyed, InMemoryState, DefaultClock>>,

    // Connection tracking
    connections: Arc<DashMap<IpAddr, ConnectionInfo>>,

    // Blocked IPs
    blocklist: Arc<RwLock<Vec<IpAddr>>>,

    // Metrics
    metrics: Arc<RwLock<DDoSMetrics>>,

    // Configuration
    config: DDoSConfig,
}

#[derive(Debug, Clone)]
pub struct DDoSConfig {
    pub max_connections_per_ip: u32,
    pub max_global_connections: u32,
    pub requests_per_second_per_ip: u32,
    pub global_requests_per_second: u32,
    pub syn_flood_threshold: u32,
    pub slowloris_timeout_seconds: u64,
    pub block_duration_minutes: u64,
    pub amplification_ratio_threshold: f64,
}

impl Default for DDoSConfig {
    fn default() -> Self {
        Self {
            max_connections_per_ip: 100,
            max_global_connections: 10000,
            requests_per_second_per_ip: 50,
            global_requests_per_second: 5000,
            syn_flood_threshold: 100,
            slowloris_timeout_seconds: 30,
            block_duration_minutes: 60,
            amplification_ratio_threshold: 10.0,
        }
    }
}

impl DDoSProtection {
    pub fn new(config: DDoSConfig) -> Self {
        // Create global rate limiter
        let global_limiter = Arc::new(
            RateLimiter::direct(
                Quota::per_second(nonzero!(config.global_requests_per_second))
            )
        );

        Self {
            ip_limiters: Arc::new(DashMap::new()),
            global_limiter,
            connections: Arc::new(DashMap::new()),
            blocklist: Arc::new(RwLock::new(Vec::new())),
            metrics: Arc::new(RwLock::new(DDoSMetrics {
                total_connections: 0,
                connections_per_second: 0.0,
                unique_ips: 0,
                blocked_ips: 0,
                syn_flood_detected: false,
                slowloris_detected: false,
                amplification_detected: false,
            })),
            config,
        }
    }

    pub async fn check_connection(&self, ip: IpAddr, user_agent: Option<String>) -> Result<(), String> {
        // Check if IP is blocked
        {
            let blocklist = self.blocklist.read().await;
            if blocklist.contains(&ip) {
                return Err("IP is blocked due to DDoS activity".to_string());
            }
        }

        // Check global rate limit
        if let Err(_) = self.global_limiter.check() {
            warn!("Global rate limit exceeded");
            self.trigger_ddos_mitigation().await;
            return Err("Server is under heavy load".to_string());
        }

        // Get or create per-IP rate limiter
        let ip_limiter = self.ip_limiters.entry(ip).or_insert_with(|| {
            Arc::new(
                RateLimiter::direct(
                    Quota::per_second(nonzero!(self.config.requests_per_second_per_ip))
                        .with_jitter(Jitter::new(Duration::from_millis(100)))
                )
            )
        }).clone();

        // Check per-IP rate limit
        if let Err(_) = ip_limiter.check() {
            warn!("Rate limit exceeded for IP: {}", ip);
            self.mark_suspicious(ip, 20.0).await;
            return Err("Rate limit exceeded".to_string());
        }

        // Update connection tracking
        let now = Instant::now();
        self.connections.entry(ip)
            .and_modify(|conn| {
                conn.connections += 1;
                conn.last_seen = now;
                conn.user_agent = user_agent.clone();
            })
            .or_insert(ConnectionInfo {
                ip,
                connections: 1,
                last_seen: now,
                syn_count: 0,
                incomplete_requests: 0,
                user_agent,
                suspicious_score: 0.0,
            });

        // Check for connection limit per IP
        if let Some(conn_info) = self.connections.get(&ip) {
            if conn_info.connections > self.config.max_connections_per_ip {
                warn!("Connection limit exceeded for IP: {}", ip);
                self.mark_suspicious(ip, 30.0).await;
                return Err("Connection limit exceeded".to_string());
            }
        }

        // Update metrics
        {
            let mut metrics = self.metrics.write().await;
            metrics.total_connections += 1;
            metrics.unique_ips = self.connections.len();
        }

        Ok(())
    }

    pub async fn report_syn_packet(&self, ip: IpAddr) {
        self.connections.entry(ip)
            .and_modify(|conn| {
                conn.syn_count += 1;

                // Check for SYN flood
                if conn.syn_count > self.config.syn_flood_threshold {
                    warn!("SYN flood detected from IP: {}", ip);
                    conn.suspicious_score += 50.0;
                }
            })
            .or_insert(ConnectionInfo {
                ip,
                connections: 0,
                last_seen: Instant::now(),
                syn_count: 1,
                incomplete_requests: 0,
                user_agent: None,
                suspicious_score: 0.0,
            });

        // Check if should block
        if let Some(conn) = self.connections.get(&ip) {
            if conn.suspicious_score >= 100.0 {
                self.block_ip(ip).await;
            }
        }
    }

    pub async fn report_incomplete_request(&self, ip: IpAddr) {
        let now = Instant::now();

        self.connections.entry(ip)
            .and_modify(|conn| {
                conn.incomplete_requests += 1;

                // Check for Slowloris attack
                let time_since_start = now.duration_since(conn.last_seen);
                if time_since_start > Duration::from_secs(self.config.slowloris_timeout_seconds) {
                    warn!("Slowloris attack detected from IP: {}", ip);
                    conn.suspicious_score += 40.0;
                }
            });

        // Update metrics
        let incomplete_count: u32 = self.connections.iter()
            .map(|entry| entry.incomplete_requests)
            .sum();

        if incomplete_count > 1000 {
            let mut metrics = self.metrics.write().await;
            metrics.slowloris_detected = true;
            warn!("Slowloris attack in progress - {} incomplete requests", incomplete_count);
        }
    }

    pub async fn check_amplification(&self, request_size: usize, response_size: usize) -> bool {
        let ratio = response_size as f64 / request_size.max(1) as f64;

        if ratio > self.config.amplification_ratio_threshold {
            warn!("Amplification attack detected - ratio: {:.2}", ratio);

            let mut metrics = self.metrics.write().await;
            metrics.amplification_detected = true;

            return true;
        }

        false
    }

    async fn mark_suspicious(&self, ip: IpAddr, score: f64) {
        self.connections.entry(ip)
            .and_modify(|conn| {
                conn.suspicious_score += score;

                if conn.suspicious_score >= 100.0 {
                    warn!("Blocking suspicious IP: {} (score: {})", ip, conn.suspicious_score);
                }
            });

        // Check if should block
        if let Some(conn) = self.connections.get(&ip) {
            if conn.suspicious_score >= 100.0 {
                self.block_ip(ip).await;
            }
        }
    }

    async fn block_ip(&self, ip: IpAddr) {
        let mut blocklist = self.blocklist.write().await;
        if !blocklist.contains(&ip) {
            blocklist.push(ip);
            error!("BLOCKED IP: {} for DDoS activity", ip);

            let mut metrics = self.metrics.write().await;
            metrics.blocked_ips = blocklist.len();
        }

        // Schedule unblock
        let blocklist_clone = self.blocklist.clone();
        let duration = Duration::from_secs(self.config.block_duration_minutes * 60);

        tokio::spawn(async move {
            tokio::time::sleep(duration).await;
            let mut blocklist = blocklist_clone.write().await;
            blocklist.retain(|&blocked_ip| blocked_ip != ip);
            info!("Unblocked IP: {}", ip);
        });
    }

    async fn trigger_ddos_mitigation(&self) {
        error!("DDoS MITIGATION TRIGGERED");

        // Analyze attack pattern
        let top_offenders = self.get_top_offenders(10).await;

        // Block top offenders
        for (ip, score) in top_offenders {
            if score > 50.0 {
                self.block_ip(ip).await;
            }
        }

        // Enable stricter rate limiting
        // This would trigger additional protective measures
        let mut metrics = self.metrics.write().await;
        if self.detect_syn_flood().await {
            metrics.syn_flood_detected = true;
            warn!("SYN flood mitigation activated");
        }
    }

    async fn get_top_offenders(&self, limit: usize) -> Vec<(IpAddr, f64)> {
        let mut offenders: Vec<_> = self.connections.iter()
            .map(|entry| (entry.key().clone(), entry.value().suspicious_score))
            .collect();

        offenders.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap());
        offenders.truncate(limit);
        offenders
    }

    async fn detect_syn_flood(&self) -> bool {
        let syn_total: u32 = self.connections.iter()
            .map(|entry| entry.syn_count)
            .sum();

        syn_total > self.config.syn_flood_threshold * 10
    }

    pub async fn cleanup_old_connections(&self, age_minutes: u64) {
        let cutoff = Instant::now() - Duration::from_secs(age_minutes * 60);

        self.connections.retain(|_, conn| conn.last_seen > cutoff);

        // Clean up old rate limiters
        let active_ips: Vec<IpAddr> = self.connections.iter()
            .map(|entry| entry.key().clone())
            .collect();

        self.ip_limiters.retain(|ip, _| active_ips.contains(ip));
    }

    pub async fn get_metrics(&self) -> DDoSMetrics {
        self.metrics.read().await.clone()
    }
}

#[derive(Clone)]
pub struct ConnectionThrottle {
    limiter: Arc<RateLimiter<NotKeyed, InMemoryState, DefaultClock>>,
}

impl ConnectionThrottle {
    pub fn new(max_per_second: u32) -> Self {
        Self {
            limiter: Arc::new(
                RateLimiter::direct(Quota::per_second(nonzero!(max_per_second)))
            ),
        }
    }

    pub async fn check(&self) -> bool {
        self.limiter.check().is_ok()
    }

    pub async fn wait(&self) {
        self.limiter.until_ready().await;
    }
}