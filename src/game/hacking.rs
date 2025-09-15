use std::collections::HashMap;
use chrono::{DateTime, Utc, Duration};
use serde::{Deserialize, Serialize};
use rand::{Rng, thread_rng};
use crate::{UserId, IpAddress, SoftwareId, HeResult, HackerExperienceError};

/// Complete hacking system implementation matching HackerExperience business logic
#[derive(Debug, Clone)]
pub struct HackingSystem {
    /// Active hack attempts - IP -> HackAttempt
    active_attempts: HashMap<IpAddress, HackAttempt>,
    /// IP scan cache for performance
    scan_cache: HashMap<IpAddress, ScanResult>,
    /// Password cracking attempts
    crack_attempts: HashMap<IpAddress, CrackAttempt>,
    /// Log entries generated during hacking
    generated_logs: Vec<LogEntry>,
}

impl HackingSystem {
    pub fn new() -> Self {
        Self {
            active_attempts: HashMap::new(),
            scan_cache: HashMap::new(),
            crack_attempts: HashMap::new(),
            generated_logs: Vec::new(),
        }
    }

    /// Discover random IP addresses for hacking targets
    pub fn discover_ips(&self, count: usize) -> Vec<IpAddress> {
        let mut rng = thread_rng();
        let mut ips = Vec::with_capacity(count);
        
        for _ in 0..count {
            // Generate realistic IP ranges (avoiding reserved ranges)
            let a = rng.gen_range(1..=223);
            let b = rng.gen_range(0..=255);
            let c = rng.gen_range(0..=255);
            let d = rng.gen_range(1..=254);
            
            // Skip reserved ranges
            if a == 10 || (a == 172 && (16..=31).contains(&b)) || (a == 192 && b == 168) {
                continue;
            }
            
            ips.push(format!("{}.{}.{}.{}", a, b, c, d));
        }
        
        ips
    }

    /// Scan target IP for open ports and services
    pub fn port_scan(&mut self, target_ip: &str, scanner_power: u32) -> HeResult<ScanResult> {
        // Check cache first (expires after 5 minutes)
        if let Some(cached) = self.scan_cache.get(target_ip) {
            if cached.scanned_at + Duration::minutes(5) > Utc::now() {
                return Ok(cached.clone());
            }
        }

        let mut rng = thread_rng();
        
        // Determine target type based on IP characteristics
        let target_type = self.determine_target_type(target_ip);
        
        // Generate ports based on target type and scanner power
        let open_ports = self.generate_open_ports(&target_type, scanner_power);
        
        // Generate services running on ports
        let services = self.generate_services(&open_ports, &target_type);
        
        // Calculate firewall strength (affects hack difficulty)
        let firewall_strength = self.calculate_firewall_strength(&target_type, scanner_power);
        
        let result = ScanResult {
            target_ip: target_ip.to_string(),
            target_type,
            open_ports,
            services,
            firewall_strength,
            scanned_at: Utc::now(),
        };
        
        self.scan_cache.insert(target_ip.to_string(), result.clone());
        Ok(result)
    }

    /// Attempt to hack a target using brute force
    pub fn bruteforce_hack(
        &mut self, 
        attacker_id: UserId,
        target_ip: &str,
        cracker_software_id: SoftwareId,
        cracker_power: u32
    ) -> HeResult<HackResult> {
        // Check if target is already being hacked
        if self.active_attempts.contains_key(target_ip) {
            return Err(HackerExperienceError::HackInProgress);
        }

        // Get scan result first
        let scan_result = self.port_scan(target_ip, cracker_power / 2)?;
        
        // Calculate hack probability based on multiple factors
        let hack_probability = self.calculate_hack_probability(
            cracker_power,
            scan_result.firewall_strength,
            &scan_result.target_type
        );

        let mut rng = thread_rng();
        let success = rng.gen::<f64>() < hack_probability;

        // Generate logs regardless of success
        self.generate_hack_logs(attacker_id, target_ip, success, cracker_software_id);

        let result = if success {
            let access_level = self.determine_access_level(cracker_power, scan_result.firewall_strength);
            
            HackResult {
                success: true,
                access_level: Some(access_level),
                password_found: self.try_find_password(cracker_power),
                money_found: self.calculate_money_found(&scan_result.target_type, cracker_power),
                files_accessible: self.generate_accessible_files(&scan_result.target_type, access_level),
                reputation_gained: self.calculate_reputation_gain(&scan_result.target_type),
                experience_gained: self.calculate_experience_gain(&scan_result.target_type),
            }
        } else {
            HackResult {
                success: false,
                access_level: None,
                password_found: None,
                money_found: 0,
                files_accessible: Vec::new(),
                reputation_gained: -5, // Penalty for failed hack
                experience_gained: 1, // Small XP for attempt
            }
        };

        Ok(result)
    }

    /// Attempt to hack using exploit method
    pub fn exploit_hack(
        &mut self,
        attacker_id: UserId,
        target_ip: &str,
        exploit_software_id: SoftwareId,
        exploit_power: u32
    ) -> HeResult<HackResult> {
        // Get scan result first
        let scan_result = self.port_scan(target_ip, exploit_power / 2)?;
        
        // Exploits are more effective against specific services
        let exploit_effectiveness = self.calculate_exploit_effectiveness(
            exploit_power,
            &scan_result.services
        );

        let hack_probability = (exploit_effectiveness / 100.0).min(0.95); // Max 95% chance

        let mut rng = thread_rng();
        let success = rng.gen::<f64>() < hack_probability;

        // Generate logs
        self.generate_exploit_logs(attacker_id, target_ip, success, exploit_software_id);

        let result = if success {
            let access_level = AccessLevel::Root; // Exploits typically give root access
            
            HackResult {
                success: true,
                access_level: Some(access_level),
                password_found: Some(self.generate_password()),
                money_found: self.calculate_money_found(&scan_result.target_type, exploit_power) * 2, // Exploits find more money
                files_accessible: self.generate_accessible_files(&scan_result.target_type, access_level),
                reputation_gained: self.calculate_reputation_gain(&scan_result.target_type) * 2,
                experience_gained: self.calculate_experience_gain(&scan_result.target_type) * 2,
            }
        } else {
            HackResult {
                success: false,
                access_level: None,
                password_found: None,
                money_found: 0,
                files_accessible: Vec::new(),
                reputation_gained: -10, // Higher penalty for failed exploit
                experience_gained: 2,
            }
        };

        Ok(result)
    }

    /// Password cracking with dictionary and brute force methods
    pub fn crack_password(
        &mut self,
        target_ip: &str,
        password_hash: &str,
        cracker_power: u32,
        use_dictionary: bool
    ) -> HeResult<CrackResult> {
        let crack_time = if use_dictionary {
            self.calculate_dictionary_crack_time(password_hash, cracker_power)
        } else {
            self.calculate_bruteforce_crack_time(password_hash, cracker_power)
        };

        let attempt = CrackAttempt {
            target_ip: target_ip.to_string(),
            password_hash: password_hash.to_string(),
            method: if use_dictionary { CrackMethod::Dictionary } else { CrackMethod::Bruteforce },
            started_at: Utc::now(),
            estimated_completion: Utc::now() + Duration::seconds(crack_time as i64),
            cracker_power,
        };

        self.crack_attempts.insert(target_ip.to_string(), attempt);

        Ok(CrackResult {
            estimated_time: crack_time,
            success_probability: self.calculate_crack_probability(password_hash, cracker_power, use_dictionary),
        })
    }

    /// Hide logs to avoid detection
    pub fn hide_logs(&mut self, target_ip: &str, log_ids: Vec<u64>, log_remover_power: u32) -> HeResult<HideLogsResult> {
        let mut hidden_count = 0;
        let mut failed_count = 0;

        for log_id in log_ids {
            // Calculate probability of successfully hiding this log
            let hide_probability = (log_remover_power as f64 / 100.0).min(0.98); // Max 98% chance
            
            let mut rng = thread_rng();
            if rng.gen::<f64>() < hide_probability {
                hidden_count += 1;
                // In real implementation, would mark log as hidden in database
            } else {
                failed_count += 1;
            }
        }

        Ok(HideLogsResult {
            total_logs: log_ids.len(),
            hidden_successfully: hidden_count,
            failed_to_hide: failed_count,
            detection_risk: self.calculate_detection_risk(failed_count, log_remover_power),
        })
    }

    /// Get all generated logs
    pub fn get_logs(&self) -> &Vec<LogEntry> {
        &self.generated_logs
    }

    /// Clear old logs (cleanup)
    pub fn cleanup_old_logs(&mut self, older_than: DateTime<Utc>) {
        self.generated_logs.retain(|log| log.timestamp > older_than);
    }

    // Private helper methods

    fn determine_target_type(&self, ip: &str) -> TargetType {
        let mut rng = thread_rng();
        
        // Simple IP-based heuristic for target type
        let ip_parts: Vec<&str> = ip.split('.').collect();
        let last_octet: u8 = ip_parts[3].parse().unwrap_or(1);
        
        match last_octet % 10 {
            0..=3 => TargetType::PersonalPC,
            4..=6 => TargetType::Corporation,
            7..=8 => TargetType::Government,
            _ => TargetType::Bank,
        }
    }

    fn generate_open_ports(&self, target_type: &TargetType, scanner_power: u32) -> Vec<u16> {
        let mut rng = thread_rng();
        let mut ports = Vec::new();
        
        // Common ports always present
        ports.extend_from_slice(&[22, 80, 443]);
        
        // Additional ports based on target type
        match target_type {
            TargetType::PersonalPC => {
                if rng.gen_bool(0.7) { ports.push(21); } // FTP
                if rng.gen_bool(0.5) { ports.push(25); } // SMTP
                if rng.gen_bool(0.3) { ports.push(110); } // POP3
            },
            TargetType::Corporation => {
                ports.extend_from_slice(&[21, 25, 53, 110, 143, 993, 995]);
                if rng.gen_bool(0.8) { ports.push(3389); } // RDP
                if rng.gen_bool(0.6) { ports.push(1433); } // SQL Server
            },
            TargetType::Government => {
                ports.extend_from_slice(&[21, 25, 53, 110, 143, 993, 995, 3389]);
                if rng.gen_bool(0.9) { ports.push(636); } // LDAPS
                if rng.gen_bool(0.7) { ports.push(88); }  // Kerberos
            },
            TargetType::Bank => {
                ports.extend_from_slice(&[25, 53, 110, 143, 993, 995, 636]);
                if rng.gen_bool(0.95) { ports.push(8443); } // Secure banking
                if rng.gen_bool(0.8) { ports.push(1521); }  // Oracle DB
            },
        }
        
        // Scanner power affects number of ports discovered
        let max_ports = ((scanner_power / 10).min(50)) as usize;
        if ports.len() > max_ports {
            ports.truncate(max_ports);
        }
        
        ports.sort();
        ports.dedup();
        ports
    }

    fn generate_services(&self, ports: &[u16], target_type: &TargetType) -> Vec<Service> {
        ports.iter().map(|&port| {
            let service_name = match port {
                21 => "FTP".to_string(),
                22 => "SSH".to_string(),
                25 => "SMTP".to_string(),
                53 => "DNS".to_string(),
                80 => "HTTP".to_string(),
                110 => "POP3".to_string(),
                143 => "IMAP".to_string(),
                443 => "HTTPS".to_string(),
                636 => "LDAPS".to_string(),
                993 => "IMAPS".to_string(),
                995 => "POP3S".to_string(),
                1433 => "MS-SQL".to_string(),
                1521 => "Oracle".to_string(),
                3389 => "RDP".to_string(),
                8443 => "HTTPS-Alt".to_string(),
                _ => format!("Unknown-{}", port),
            };

            let version = self.generate_service_version(&service_name);
            let vulnerability_level = self.calculate_service_vulnerability(&service_name, &version, target_type);

            Service {
                port,
                name: service_name,
                version,
                vulnerability_level,
            }
        }).collect()
    }

    fn generate_service_version(&self, service_name: &str) -> String {
        let mut rng = thread_rng();
        
        match service_name {
            "SSH" => format!("OpenSSH_{}.{}", rng.gen_range(6..=8), rng.gen_range(0..=9)),
            "HTTP" | "HTTPS" => format!("Apache/2.{}.{}", rng.gen_range(2..=4), rng.gen_range(0..=50)),
            "FTP" => format!("vsftpd_{}.{}.{}", rng.gen_range(2..=3), rng.gen_range(0..=5), rng.gen_range(0..=20)),
            "SMTP" => format!("Postfix_{}.{}.{}", rng.gen_range(2..=3), rng.gen_range(0..=11), rng.gen_range(0..=20)),
            _ => format!("{}_1.{}.{}", service_name, rng.gen_range(0..=9), rng.gen_range(0..=99)),
        }
    }

    fn calculate_service_vulnerability(&self, service_name: &str, version: &str, target_type: &TargetType) -> u8 {
        let mut rng = thread_rng();
        
        // Base vulnerability based on service type
        let base_vuln = match service_name {
            "FTP" => rng.gen_range(60..=90),
            "HTTP" | "HTTPS" => rng.gen_range(40..=80),
            "SSH" => rng.gen_range(20..=50),
            "SMTP" => rng.gen_range(30..=70),
            _ => rng.gen_range(10..=60),
        };

        // Modify based on target type (better targets have better security)
        let type_modifier = match target_type {
            TargetType::PersonalPC => 0,
            TargetType::Corporation => -10,
            TargetType::Government => -20,
            TargetType::Bank => -30,
        };

        ((base_vuln as i32 + type_modifier).max(5).min(95)) as u8
    }

    fn calculate_firewall_strength(&self, target_type: &TargetType, scanner_power: u32) -> u32 {
        let mut rng = thread_rng();
        
        let base_strength = match target_type {
            TargetType::PersonalPC => rng.gen_range(10..=50),
            TargetType::Corporation => rng.gen_range(40..=80),
            TargetType::Government => rng.gen_range(70..=120),
            TargetType::Bank => rng.gen_range(100..=200),
        };

        // Scanner power affects detected strength (more accurate with better scanner)
        let accuracy = (scanner_power as f64 / 100.0).min(1.0);
        let variation = (base_strength as f64 * (1.0 - accuracy) * 0.3) as u32;
        
        if rng.gen_bool(0.5) {
            base_strength + rng.gen_range(0..=variation)
        } else {
            (base_strength as i32 - rng.gen_range(0..=variation) as i32).max(5) as u32
        }
    }

    fn calculate_hack_probability(&self, cracker_power: u32, firewall_strength: u32, target_type: &TargetType) -> f64 {
        let power_ratio = cracker_power as f64 / firewall_strength as f64;
        
        // Base probability calculation
        let base_prob = match power_ratio {
            x if x >= 2.0 => 0.85,
            x if x >= 1.5 => 0.70,
            x if x >= 1.0 => 0.55,
            x if x >= 0.75 => 0.40,
            x if x >= 0.5 => 0.25,
            _ => 0.10,
        };

        // Target type modifier
        let type_modifier = match target_type {
            TargetType::PersonalPC => 0.05,
            TargetType::Corporation => 0.0,
            TargetType::Government => -0.05,
            TargetType::Bank => -0.10,
        };

        (base_prob + type_modifier).max(0.01).min(0.95)
    }

    fn calculate_exploit_effectiveness(&self, exploit_power: u32, services: &[Service]) -> u32 {
        let mut effectiveness = 0;
        
        for service in services {
            // Exploit effectiveness based on service vulnerability
            let service_effectiveness = (exploit_power * service.vulnerability_level as u32) / 100;
            effectiveness = effectiveness.max(service_effectiveness);
        }
        
        effectiveness.min(95) // Max 95% effectiveness
    }

    fn determine_access_level(&self, cracker_power: u32, firewall_strength: u32) -> AccessLevel {
        let power_ratio = cracker_power as f64 / firewall_strength as f64;
        let mut rng = thread_rng();
        
        match power_ratio {
            x if x >= 2.0 => {
                if rng.gen_bool(0.8) { AccessLevel::Root } else { AccessLevel::User }
            },
            x if x >= 1.5 => {
                if rng.gen_bool(0.6) { AccessLevel::User } else { AccessLevel::Guest }
            },
            x if x >= 1.0 => {
                if rng.gen_bool(0.7) { AccessLevel::Guest } else { AccessLevel::User }
            },
            _ => AccessLevel::Guest,
        }
    }

    fn try_find_password(&self, cracker_power: u32) -> Option<String> {
        let mut rng = thread_rng();
        let find_probability = (cracker_power as f64 / 150.0).min(0.8);
        
        if rng.gen::<f64>() < find_probability {
            Some(self.generate_password())
        } else {
            None
        }
    }

    fn generate_password(&self) -> String {
        let mut rng = thread_rng();
        let common_passwords = vec![
            "password", "123456", "admin", "root", "user", "guest",
            "qwerty", "letmein", "welcome", "password123", "admin123"
        ];
        
        if rng.gen_bool(0.6) {
            // Common password
            common_passwords[rng.gen_range(0..common_passwords.len())].to_string()
        } else {
            // Generated password
            let length = rng.gen_range(6..=12);
            (0..length)
                .map(|_| {
                    let chars = b"abcdefghijklmnopqrstuvwxyzABCDEFGHIJKLMNOPQRSTUVWXYZ0123456789";
                    chars[rng.gen_range(0..chars.len())] as char
                })
                .collect()
        }
    }

    fn calculate_money_found(&self, target_type: &TargetType, power: u32) -> i64 {
        let mut rng = thread_rng();
        
        let base_amount = match target_type {
            TargetType::PersonalPC => rng.gen_range(100..=5000),
            TargetType::Corporation => rng.gen_range(5000..=50000),
            TargetType::Government => rng.gen_range(10000..=100000),
            TargetType::Bank => rng.gen_range(50000..=1000000),
        };

        // Power affects money finding capability
        let power_multiplier = (power as f64 / 50.0).min(3.0);
        (base_amount as f64 * power_multiplier) as i64
    }

    fn generate_accessible_files(&self, target_type: &TargetType, access_level: AccessLevel) -> Vec<FileEntry> {
        let mut files = Vec::new();
        let mut rng = thread_rng();
        
        // Common files based on access level
        match access_level {
            AccessLevel::Guest => {
                files.push(FileEntry {
                    name: "readme.txt".to_string(),
                    size: rng.gen_range(1..=10),
                    file_type: FileType::Document,
                });
                files.push(FileEntry {
                    name: "public_info.txt".to_string(),
                    size: rng.gen_range(1..=5),
                    file_type: FileType::Document,
                });
            },
            AccessLevel::User => {
                files.extend_from_slice(&[
                    FileEntry {
                        name: "documents.zip".to_string(),
                        size: rng.gen_range(10..=100),
                        file_type: FileType::Archive,
                    },
                    FileEntry {
                        name: "passwords.txt".to_string(),
                        size: rng.gen_range(1..=5),
                        file_type: FileType::Document,
                    },
                ]);
            },
            AccessLevel::Root => {
                files.extend_from_slice(&[
                    FileEntry {
                        name: "shadow".to_string(),
                        size: rng.gen_range(5..=20),
                        file_type: FileType::System,
                    },
                    FileEntry {
                        name: "database.sql".to_string(),
                        size: rng.gen_range(100..=1000),
                        file_type: FileType::Database,
                    },
                    FileEntry {
                        name: "config.conf".to_string(),
                        size: rng.gen_range(5..=50),
                        file_type: FileType::Config,
                    },
                ]);
            },
        }

        // Add target-specific files
        match target_type {
            TargetType::Bank => {
                files.push(FileEntry {
                    name: "transactions.db".to_string(),
                    size: rng.gen_range(500..=5000),
                    file_type: FileType::Database,
                });
            },
            TargetType::Government => {
                files.push(FileEntry {
                    name: "classified.enc".to_string(),
                    size: rng.gen_range(50..=500),
                    file_type: FileType::Encrypted,
                });
            },
            _ => {},
        }

        files
    }

    fn calculate_reputation_gain(&self, target_type: &TargetType) -> i32 {
        let mut rng = thread_rng();
        
        let base_rep = match target_type {
            TargetType::PersonalPC => rng.gen_range(1..=5),
            TargetType::Corporation => rng.gen_range(5..=15),
            TargetType::Government => rng.gen_range(10..=25),
            TargetType::Bank => rng.gen_range(20..=50),
        };

        base_rep
    }

    fn calculate_experience_gain(&self, target_type: &TargetType) -> i32 {
        let mut rng = thread_rng();
        
        let base_xp = match target_type {
            TargetType::PersonalPC => rng.gen_range(10..=50),
            TargetType::Corporation => rng.gen_range(50..=150),
            TargetType::Government => rng.gen_range(100..=300),
            TargetType::Bank => rng.gen_range(200..=500),
        };

        base_xp
    }

    fn calculate_dictionary_crack_time(&self, password_hash: &str, cracker_power: u32) -> u32 {
        let mut rng = thread_rng();
        
        // Simplified hash complexity (in real implementation, would analyze actual hash)
        let hash_complexity = password_hash.len() as u32 * 10;
        let base_time = hash_complexity / cracker_power.max(1);
        
        // Dictionary attacks are faster
        let time_seconds = (base_time / 3).max(30); // Minimum 30 seconds
        time_seconds + rng.gen_range(0..=(time_seconds / 4)) // Add some randomness
    }

    fn calculate_bruteforce_crack_time(&self, password_hash: &str, cracker_power: u32) -> u32 {
        let mut rng = thread_rng();
        
        let hash_complexity = password_hash.len() as u32 * 20; // Brute force is harder
        let base_time = hash_complexity / cracker_power.max(1);
        
        let time_seconds = base_time.max(300); // Minimum 5 minutes
        time_seconds + rng.gen_range(0..=(time_seconds / 2))
    }

    fn calculate_crack_probability(&self, password_hash: &str, cracker_power: u32, use_dictionary: bool) -> f64 {
        let hash_strength = password_hash.len() as f64 * 5.0; // Simplified
        let power_ratio = cracker_power as f64 / hash_strength;
        
        let base_prob = if use_dictionary {
            match power_ratio {
                x if x >= 3.0 => 0.95,
                x if x >= 2.0 => 0.80,
                x if x >= 1.0 => 0.60,
                x if x >= 0.5 => 0.40,
                _ => 0.20,
            }
        } else {
            match power_ratio {
                x if x >= 4.0 => 0.90,
                x if x >= 3.0 => 0.70,
                x if x >= 2.0 => 0.50,
                x if x >= 1.0 => 0.30,
                _ => 0.10,
            }
        };

        base_prob
    }

    fn calculate_detection_risk(&self, failed_hides: usize, log_remover_power: u32) -> f64 {
        if failed_hides == 0 {
            return 0.0;
        }

        let base_risk = failed_hides as f64 * 0.1; // 10% risk per failed hide
        let power_reduction = (log_remover_power as f64 / 100.0) * 0.5;
        
        (base_risk - power_reduction).max(0.0).min(1.0)
    }

    fn generate_hack_logs(&mut self, attacker_id: UserId, target_ip: &str, success: bool, software_id: SoftwareId) {
        let mut rng = thread_rng();
        let log_id = rng.gen::<u64>();
        
        let message = if success {
            format!("Successful login from {}", self.generate_fake_ip())
        } else {
            format!("Failed login attempt from {}", self.generate_fake_ip())
        };

        let log_entry = LogEntry {
            id: log_id,
            target_ip: target_ip.to_string(),
            attacker_id: Some(attacker_id),
            log_type: LogType::Access,
            message,
            timestamp: Utc::now(),
            software_used: Some(software_id),
            severity: if success { LogSeverity::Info } else { LogSeverity::Warning },
        };

        self.generated_logs.push(log_entry);

        // Generate additional logs for successful hacks
        if success {
            let file_access_log = LogEntry {
                id: rng.gen::<u64>(),
                target_ip: target_ip.to_string(),
                attacker_id: Some(attacker_id),
                log_type: LogType::FileAccess,
                message: "File system accessed".to_string(),
                timestamp: Utc::now() + Duration::seconds(rng.gen_range(1..=30)),
                software_used: Some(software_id),
                severity: LogSeverity::Info,
            };
            self.generated_logs.push(file_access_log);
        }
    }

    fn generate_exploit_logs(&mut self, attacker_id: UserId, target_ip: &str, success: bool, software_id: SoftwareId) {
        let mut rng = thread_rng();
        let log_id = rng.gen::<u64>();
        
        let message = if success {
            "System vulnerability exploited successfully".to_string()
        } else {
            "Exploit attempt detected and blocked".to_string()
        };

        let log_entry = LogEntry {
            id: log_id,
            target_ip: target_ip.to_string(),
            attacker_id: Some(attacker_id),
            log_type: LogType::Security,
            message,
            timestamp: Utc::now(),
            software_used: Some(software_id),
            severity: if success { LogSeverity::Critical } else { LogSeverity::High },
        };

        self.generated_logs.push(log_entry);
    }

    fn generate_fake_ip(&self) -> String {
        let mut rng = thread_rng();
        format!("{}.{}.{}.{}", 
            rng.gen_range(1..=255),
            rng.gen_range(0..=255),
            rng.gen_range(0..=255),
            rng.gen_range(1..=255)
        )
    }
}

// Data structures

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HackAttempt {
    pub target_ip: IpAddress,
    pub attacker_id: UserId,
    pub started_at: DateTime<Utc>,
    pub estimated_completion: DateTime<Utc>,
    pub method: HackMethod,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum HackMethod {
    Bruteforce,
    Exploit,
    Dictionary,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ScanResult {
    pub target_ip: IpAddress,
    pub target_type: TargetType,
    pub open_ports: Vec<u16>,
    pub services: Vec<Service>,
    pub firewall_strength: u32,
    pub scanned_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum TargetType {
    PersonalPC,
    Corporation,
    Government,
    Bank,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Service {
    pub port: u16,
    pub name: String,
    pub version: String,
    pub vulnerability_level: u8, // 0-100
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HackResult {
    pub success: bool,
    pub access_level: Option<AccessLevel>,
    pub password_found: Option<String>,
    pub money_found: i64,
    pub files_accessible: Vec<FileEntry>,
    pub reputation_gained: i32,
    pub experience_gained: i32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum AccessLevel {
    Guest,
    User,
    Root,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FileEntry {
    pub name: String,
    pub size: u64, // in KB
    pub file_type: FileType,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum FileType {
    Document,
    Archive,
    System,
    Database,
    Config,
    Encrypted,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CrackAttempt {
    pub target_ip: IpAddress,
    pub password_hash: String,
    pub method: CrackMethod,
    pub started_at: DateTime<Utc>,
    pub estimated_completion: DateTime<Utc>,
    pub cracker_power: u32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum CrackMethod {
    Dictionary,
    Bruteforce,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CrackResult {
    pub estimated_time: u32, // in seconds
    pub success_probability: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HideLogsResult {
    pub total_logs: usize,
    pub hidden_successfully: usize,
    pub failed_to_hide: usize,
    pub detection_risk: f64, // 0.0 to 1.0
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LogEntry {
    pub id: u64,
    pub target_ip: IpAddress,
    pub attacker_id: Option<UserId>,
    pub log_type: LogType,
    pub message: String,
    pub timestamp: DateTime<Utc>,
    pub software_used: Option<SoftwareId>,
    pub severity: LogSeverity,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum LogType {
    Access,
    FileAccess,
    Security,
    System,
    Network,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum LogSeverity {
    Info,
    Warning,
    High,
    Critical,
}

// Tests
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_ip_discovery() {
        let hacking_system = HackingSystem::new();
        let ips = hacking_system.discover_ips(10);
        assert_eq!(ips.len(), 10);
        
        for ip in &ips {
            assert!(ip.matches('.').count() == 3); // Valid IP format
        }
    }

    #[test]
    fn test_port_scan() {
        let mut hacking_system = HackingSystem::new();
        let result = hacking_system.port_scan("192.168.1.1", 50).unwrap();
        
        assert!(!result.open_ports.is_empty());
        assert!(!result.services.is_empty());
        assert!(result.firewall_strength > 0);
    }

    #[test]
    fn test_hack_attempt() {
        let mut hacking_system = HackingSystem::new();
        let result = hacking_system.bruteforce_hack(1, "192.168.1.100", 1, 75).unwrap();
        
        // Should always return a result (success or failure)
        assert!(result.reputation_gained != 0); // Always gain or lose reputation
    }
}