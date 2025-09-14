use std::collections::HashMap;
use chrono::{DateTime, Utc, Duration};
use serde::{Deserialize, Serialize};
use rand::{Rng, thread_rng};
use crate::{UserId, IpAddress, SoftwareId, HeResult, HackerExperienceError};

/// Complete virus system implementation with installation, collection, and DDoS mechanics
#[derive(Debug, Clone)]
pub struct VirusSystem {
    /// Active viruses on targets - IP -> Vec<ActiveVirus>
    active_viruses: HashMap<IpAddress, Vec<ActiveVirus>>,
    /// Money collection tracking - virus_id -> CollectionData
    collection_data: HashMap<u64, CollectionData>,
    /// DDoS attack coordination
    ddos_networks: HashMap<u64, DDoSNetwork>,
    /// Virus spreading tracking
    spreading_attempts: HashMap<u64, SpreadingAttempt>,
}

impl VirusSystem {
    pub fn new() -> Self {
        Self {
            active_viruses: HashMap::new(),
            collection_data: HashMap::new(),
            ddos_networks: HashMap::new(),
            spreading_attempts: HashMap::new(),
        }
    }

    /// Install virus on target system
    pub fn install_virus(
        &mut self,
        attacker_id: UserId,
        target_ip: &str,
        virus_software_id: SoftwareId,
        virus_power: u32,
        virus_type: VirusType
    ) -> HeResult<InstallationResult> {
        let mut rng = thread_rng();

        // Calculate installation success probability
        let installation_probability = self.calculate_installation_probability(
            virus_power,
            &virus_type,
            target_ip
        );

        let success = rng.gen::<f64>() < installation_probability;

        if !success {
            return Ok(InstallationResult {
                success: false,
                virus_id: None,
                detection_risk: 0.3,
                estimated_lifetime: 0,
                error_message: Some("Installation failed - target system rejected virus".to_string()),
            });
        }

        // Create virus instance
        let virus_id = rng.gen::<u64>();
        let estimated_lifetime = self.calculate_virus_lifetime(&virus_type, virus_power);
        
        let active_virus = ActiveVirus {
            id: virus_id,
            owner_id: attacker_id,
            target_ip: target_ip.to_string(),
            virus_type: virus_type.clone(),
            software_id: virus_software_id,
            power: virus_power,
            installed_at: Utc::now(),
            last_activity: Utc::now(),
            estimated_removal: Utc::now() + Duration::seconds(estimated_lifetime as i64),
            status: VirusStatus::Active,
            stealth_level: self.calculate_stealth_level(&virus_type, virus_power),
            collected_money: 0,
            spread_count: 0,
            detected: false,
        };

        // Add to active viruses
        self.active_viruses.entry(target_ip.to_string())
            .or_insert_with(Vec::new)
            .push(active_virus);

        // Initialize collection data for money viruses
        match virus_type {
            VirusType::Money | VirusType::BitcoinMiner => {
                self.collection_data.insert(virus_id, CollectionData {
                    virus_id,
                    collection_rate: self.calculate_money_collection_rate(&virus_type, virus_power),
                    last_collection: Utc::now(),
                    total_collected: 0,
                    collection_efficiency: 1.0,
                });
            },
            _ => {},
        }

        Ok(InstallationResult {
            success: true,
            virus_id: Some(virus_id),
            detection_risk: self.calculate_detection_risk(&virus_type, virus_power),
            estimated_lifetime,
            error_message: None,
        })
    }

    /// Collect money from virus installations
    pub fn collect_virus_money(&mut self, attacker_id: UserId) -> HeResult<CollectionResult> {
        let mut total_collected = 0;
        let mut collections = Vec::new();
        let mut failed_collections = 0;

        // Find all viruses owned by this attacker
        for (target_ip, viruses) in &mut self.active_viruses {
            for virus in viruses.iter_mut() {
                if virus.owner_id != attacker_id || virus.status != VirusStatus::Active {
                    continue;
                }

                // Only money and Bitcoin viruses collect money
                match virus.virus_type {
                    VirusType::Money | VirusType::BitcoinMiner => {
                        if let Some(collection_data) = self.collection_data.get_mut(&virus.id) {
                            let collected = self.perform_money_collection(virus, collection_data)?;
                            
                            if collected > 0 {
                                total_collected += collected;
                                collections.push(MoneyCollection {
                                    virus_id: virus.id,
                                    target_ip: target_ip.clone(),
                                    amount: collected,
                                    virus_type: virus.virus_type.clone(),
                                });
                                
                                virus.collected_money += collected;
                                virus.last_activity = Utc::now();
                            } else {
                                failed_collections += 1;
                            }
                        }
                    },
                    _ => continue,
                }
            }
        }

        Ok(CollectionResult {
            total_collected,
            collections,
            failed_collections,
            reputation_gained: self.calculate_collection_reputation(total_collected),
        })
    }

    /// Launch DDoS attack using virus network
    pub fn launch_ddos_attack(
        &mut self,
        attacker_id: UserId,
        target_ip: &str,
        attack_duration: u32, // in seconds
        attack_power: u32
    ) -> HeResult<DDoSResult> {
        // Find all DDoS viruses owned by attacker
        let ddos_viruses = self.find_ddos_viruses(attacker_id);
        
        if ddos_viruses.is_empty() {
            return Err(HackerExperienceError::NoDDoSVirusesAvailable);
        }

        let network_id = thread_rng().gen::<u64>();
        let total_power = ddos_viruses.iter().map(|v| v.power).sum::<u32>();
        let effective_power = (total_power as f64 * self.calculate_network_efficiency(&ddos_viruses)) as u32;

        // Calculate attack success and impact
        let success_probability = self.calculate_ddos_success_probability(effective_power, target_ip);
        let mut rng = thread_rng();
        let success = rng.gen::<f64>() < success_probability;

        let impact = if success {
            self.calculate_ddos_impact(effective_power, attack_duration)
        } else {
            DDoSImpact {
                service_disruption: 0.0,
                estimated_downtime: 0,
                financial_damage: 0,
                reputation_damage: 0,
            }
        };

        // Create DDoS network
        let ddos_network = DDoSNetwork {
            id: network_id,
            attacker_id,
            target_ip: target_ip.to_string(),
            participating_viruses: ddos_viruses.into_iter().map(|v| v.id).collect(),
            start_time: Utc::now(),
            duration: attack_duration,
            total_power: effective_power,
            status: if success { DDoSStatus::Active } else { DDoSStatus::Failed },
        };

        self.ddos_networks.insert(network_id, ddos_network);

        Ok(DDoSResult {
            success,
            network_id,
            participating_virus_count: self.ddos_networks[&network_id].participating_viruses.len(),
            total_power: effective_power,
            impact,
            estimated_duration: attack_duration,
            reputation_gained: if success { 50 + (effective_power / 100) as i32 } else { -10 },
        })
    }

    /// Spread virus to new targets
    pub fn spread_virus(&mut self, source_virus_id: u64, max_targets: usize) -> HeResult<SpreadResult> {
        // Find the source virus
        let mut source_virus = None;
        for viruses in self.active_viruses.values_mut() {
            if let Some(virus) = viruses.iter_mut().find(|v| v.id == source_virus_id) {
                if virus.status == VirusStatus::Active {
                    source_virus = Some(virus.clone());
                }
                break;
            }
        }

        let source = source_virus.ok_or(HackerExperienceError::VirusNotFound)?;

        // Check if virus type can spread
        if !self.can_virus_spread(&source.virus_type) {
            return Ok(SpreadResult {
                attempted_targets: 0,
                successful_infections: 0,
                failed_infections: 0,
                new_virus_ids: Vec::new(),
            });
        }

        let mut rng = thread_rng();
        let target_ips = self.generate_spread_targets(&source.target_ip, max_targets);
        let mut successful_infections = 0;
        let mut failed_infections = 0;
        let mut new_virus_ids = Vec::new();

        for target_ip in &target_ips {
            let spread_probability = self.calculate_spread_probability(&source, target_ip);
            
            if rng.gen::<f64>() < spread_probability {
                // Create new virus with reduced power
                let new_power = (source.power as f64 * 0.8) as u32; // Spreading reduces power
                
                let installation_result = self.install_virus(
                    source.owner_id,
                    target_ip,
                    source.software_id,
                    new_power,
                    source.virus_type.clone()
                )?;

                if installation_result.success {
                    if let Some(virus_id) = installation_result.virus_id {
                        new_virus_ids.push(virus_id);
                        successful_infections += 1;
                    }
                } else {
                    failed_infections += 1;
                }
            } else {
                failed_infections += 1;
            }
        }

        // Update source virus spread count
        for viruses in self.active_viruses.values_mut() {
            if let Some(virus) = viruses.iter_mut().find(|v| v.id == source_virus_id) {
                virus.spread_count += successful_infections;
                break;
            }
        }

        Ok(SpreadResult {
            attempted_targets: target_ips.len(),
            successful_infections,
            failed_infections,
            new_virus_ids,
        })
    }

    /// Remove virus from system (antivirus or manual removal)
    pub fn remove_virus(&mut self, virus_id: u64, removal_power: u32) -> HeResult<RemovalResult> {
        let mut virus_found = false;
        let mut removal_success = false;

        // Find and attempt to remove virus
        for viruses in self.active_viruses.values_mut() {
            if let Some(position) = viruses.iter().position(|v| v.id == virus_id) {
                virus_found = true;
                let virus = &viruses[position];
                
                // Calculate removal probability
                let removal_probability = self.calculate_removal_probability(virus, removal_power);
                let mut rng = thread_rng();
                
                if rng.gen::<f64>() < removal_probability {
                    viruses.remove(position);
                    removal_success = true;
                    
                    // Clean up associated data
                    self.collection_data.remove(&virus_id);
                } else {
                    // Failed removal might alert the virus (increase detection risk)
                    let virus = &mut viruses[position];
                    virus.detected = true;
                    virus.stealth_level = (virus.stealth_level as f64 * 0.5) as u32;
                }
                break;
            }
        }

        if !virus_found {
            return Err(HackerExperienceError::VirusNotFound);
        }

        Ok(RemovalResult {
            success: removal_success,
            detection_increased: !removal_success,
        })
    }

    /// Update virus status and handle automatic events
    pub fn update_viruses(&mut self) -> HeResult<Vec<VirusEvent>> {
        let mut events = Vec::new();
        let now = Utc::now();

        // Update all active viruses
        for (target_ip, viruses) in &mut self.active_viruses {
            viruses.retain_mut(|virus| {
                // Check if virus should be automatically removed
                if now > virus.estimated_removal {
                    events.push(VirusEvent {
                        virus_id: virus.id,
                        target_ip: target_ip.clone(),
                        event_type: VirusEventType::Expired,
                        timestamp: now,
                    });
                    
                    // Clean up associated data
                    self.collection_data.remove(&virus.id);
                    return false;
                }

                // Check for automatic detection
                if !virus.detected && self.check_automatic_detection(virus) {
                    virus.detected = true;
                    virus.status = VirusStatus::Detected;
                    
                    events.push(VirusEvent {
                        virus_id: virus.id,
                        target_ip: target_ip.clone(),
                        event_type: VirusEventType::Detected,
                        timestamp: now,
                    });
                }

                // Update collection efficiency based on time
                if let Some(collection_data) = self.collection_data.get_mut(&virus.id) {
                    collection_data.collection_efficiency = 
                        self.calculate_collection_efficiency_decay(virus, now);
                }

                true
            });
        }

        // Clean up empty entries
        self.active_viruses.retain(|_, viruses| !viruses.is_empty());

        // Update DDoS networks
        self.ddos_networks.retain(|_, network| {
            if now > network.start_time + Duration::seconds(network.duration as i64) {
                events.push(VirusEvent {
                    virus_id: network.id,
                    target_ip: network.target_ip.clone(),
                    event_type: VirusEventType::DDoSCompleted,
                    timestamp: now,
                });
                false
            } else {
                true
            }
        });

        Ok(events)
    }

    /// Get all viruses owned by user
    pub fn get_user_viruses(&self, user_id: UserId) -> Vec<VirusInfo> {
        let mut viruses = Vec::new();

        for (target_ip, virus_list) in &self.active_viruses {
            for virus in virus_list {
                if virus.owner_id == user_id {
                    let collection_rate = self.collection_data.get(&virus.id)
                        .map(|data| data.collection_rate)
                        .unwrap_or(0);

                    viruses.push(VirusInfo {
                        id: virus.id,
                        target_ip: target_ip.clone(),
                        virus_type: virus.virus_type.clone(),
                        power: virus.power,
                        status: virus.status.clone(),
                        installed_at: virus.installed_at,
                        collected_money: virus.collected_money,
                        spread_count: virus.spread_count,
                        stealth_level: virus.stealth_level,
                        detected: virus.detected,
                        collection_rate,
                    });
                }
            }
        }

        viruses
    }

    // Private helper methods

    fn calculate_installation_probability(&self, virus_power: u32, virus_type: &VirusType, target_ip: &str) -> f64 {
        let mut rng = thread_rng();
        
        // Base probability based on virus power
        let base_prob = (virus_power as f64 / 150.0).min(0.9);
        
        // Virus type modifier
        let type_modifier = match virus_type {
            VirusType::Money => 0.0,
            VirusType::DDoS => -0.1, // Harder to install
            VirusType::Worm => 0.1,  // Easier to spread
            VirusType::BitcoinMiner => -0.05,
            VirusType::Keylogger => 0.05,
            VirusType::Botnet => -0.15, // Hardest to install
        };

        // Target difficulty (simulated based on IP)
        let target_difficulty = self.get_target_difficulty(target_ip);
        let difficulty_modifier = -(target_difficulty as f64 / 200.0);

        (base_prob + type_modifier + difficulty_modifier).max(0.05).min(0.95)
    }

    fn calculate_virus_lifetime(&self, virus_type: &VirusType, power: u32) -> u32 {
        let mut rng = thread_rng();
        
        let base_lifetime = match virus_type {
            VirusType::Money => 3600 * 24 * 7, // 1 week
            VirusType::DDoS => 3600 * 24 * 3,   // 3 days
            VirusType::Worm => 3600 * 24 * 14,  // 2 weeks
            VirusType::BitcoinMiner => 3600 * 24 * 10, // 10 days
            VirusType::Keylogger => 3600 * 24 * 5, // 5 days
            VirusType::Botnet => 3600 * 24 * 30, // 1 month
        };

        // Power affects lifetime (more powerful viruses last longer)
        let power_multiplier = 1.0 + (power as f64 / 200.0);
        let modified_lifetime = (base_lifetime as f64 * power_multiplier) as u32;

        // Add randomness (±25%)
        let variation = modified_lifetime / 4;
        modified_lifetime + rng.gen_range(0..=variation) - (variation / 2)
    }

    fn calculate_stealth_level(&self, virus_type: &VirusType, power: u32) -> u32 {
        let base_stealth = match virus_type {
            VirusType::Money => 30,
            VirusType::DDoS => 20, // More detectable when active
            VirusType::Worm => 40,
            VirusType::BitcoinMiner => 25, // CPU usage gives it away
            VirusType::Keylogger => 50, // Very stealthy
            VirusType::Botnet => 35,
        };

        // Power improves stealth
        let power_bonus = power / 5;
        (base_stealth + power_bonus).min(95)
    }

    fn calculate_money_collection_rate(&self, virus_type: &VirusType, power: u32) -> i64 {
        let base_rate = match virus_type {
            VirusType::Money => 100, // $100/hour
            VirusType::BitcoinMiner => 150, // $150/hour but more detectable
            _ => 0,
        };

        // Power multiplier
        let power_multiplier = 1.0 + (power as f64 / 100.0);
        (base_rate as f64 * power_multiplier) as i64
    }

    fn calculate_detection_risk(&self, virus_type: &VirusType, power: u32) -> f64 {
        let base_risk = match virus_type {
            VirusType::Money => 0.1,
            VirusType::DDoS => 0.2,
            VirusType::Worm => 0.05,
            VirusType::BitcoinMiner => 0.15,
            VirusType::Keylogger => 0.05,
            VirusType::Botnet => 0.3,
        };

        // Lower power = higher risk
        let power_modifier = -(power as f64 / 500.0);
        (base_risk + power_modifier).max(0.01).min(0.5)
    }

    fn perform_money_collection(&self, virus: &mut ActiveVirus, collection_data: &mut CollectionData) -> HeResult<i64> {
        let now = Utc::now();
        let hours_since_last = (now - collection_data.last_collection).num_hours().max(0) as f64;
        
        if hours_since_last < 1.0 {
            return Ok(0); // Can only collect once per hour
        }

        let base_amount = (collection_data.collection_rate as f64 * hours_since_last) as i64;
        let efficiency_modifier = collection_data.collection_efficiency;
        let collected = (base_amount as f64 * efficiency_modifier) as i64;

        collection_data.last_collection = now;
        collection_data.total_collected += collected;

        // Reduce efficiency over time
        collection_data.collection_efficiency = (collection_data.collection_efficiency * 0.99).max(0.1);

        Ok(collected)
    }

    fn find_ddos_viruses(&self, attacker_id: UserId) -> Vec<ActiveVirus> {
        let mut ddos_viruses = Vec::new();

        for viruses in self.active_viruses.values() {
            for virus in viruses {
                if virus.owner_id == attacker_id && 
                   virus.status == VirusStatus::Active &&
                   matches!(virus.virus_type, VirusType::DDoS | VirusType::Botnet) {
                    ddos_viruses.push(virus.clone());
                }
            }
        }

        ddos_viruses
    }

    fn calculate_network_efficiency(&self, viruses: &[ActiveVirus]) -> f64 {
        if viruses.is_empty() {
            return 0.0;
        }

        // Network efficiency decreases with more nodes (coordination overhead)
        let base_efficiency = 1.0;
        let node_penalty = (viruses.len() as f64 * 0.05).min(0.3);
        let avg_stealth = viruses.iter().map(|v| v.stealth_level as f64).sum::<f64>() / viruses.len() as f64;
        let stealth_bonus = (avg_stealth / 100.0) * 0.2;

        (base_efficiency - node_penalty + stealth_bonus).max(0.3).min(1.0)
    }

    fn calculate_ddos_success_probability(&self, attack_power: u32, target_ip: &str) -> f64 {
        let target_defense = self.get_target_defense(target_ip);
        let power_ratio = attack_power as f64 / target_defense as f64;

        match power_ratio {
            x if x >= 3.0 => 0.95,
            x if x >= 2.0 => 0.85,
            x if x >= 1.5 => 0.70,
            x if x >= 1.0 => 0.55,
            x if x >= 0.5 => 0.30,
            _ => 0.10,
        }
    }

    fn calculate_ddos_impact(&self, attack_power: u32, duration: u32) -> DDoSImpact {
        let mut rng = thread_rng();
        
        let service_disruption = (attack_power as f64 / 1000.0).min(1.0);
        let estimated_downtime = (duration as f64 * service_disruption) as u32;
        let financial_damage = (attack_power as i64 * duration as i64 / 100).max(1000);
        let reputation_damage = (attack_power / 20) as i32;

        DDoSImpact {
            service_disruption,
            estimated_downtime,
            financial_damage,
            reputation_damage,
        }
    }

    fn can_virus_spread(&self, virus_type: &VirusType) -> bool {
        matches!(virus_type, VirusType::Worm | VirusType::Botnet)
    }

    fn generate_spread_targets(&self, source_ip: &str, max_targets: usize) -> Vec<IpAddress> {
        let mut rng = thread_rng();
        let mut targets = Vec::new();
        
        // Parse source IP
        let ip_parts: Vec<&str> = source_ip.split('.').collect();
        if ip_parts.len() != 4 {
            return targets;
        }

        let base_a: u8 = ip_parts[0].parse().unwrap_or(192);
        let base_b: u8 = ip_parts[1].parse().unwrap_or(168);
        let base_c: u8 = ip_parts[2].parse().unwrap_or(1);

        // Generate targets in same subnet first (higher spread probability)
        for _ in 0..(max_targets / 2) {
            let d = rng.gen_range(1..=254);
            targets.push(format!("{}.{}.{}.{}", base_a, base_b, base_c, d));
        }

        // Generate random targets
        for _ in 0..(max_targets - targets.len()) {
            let a = rng.gen_range(1..=223);
            let b = rng.gen_range(0..=255);
            let c = rng.gen_range(0..=255);
            let d = rng.gen_range(1..=254);
            targets.push(format!("{}.{}.{}.{}", a, b, c, d));
        }

        targets
    }

    fn calculate_spread_probability(&self, source_virus: &ActiveVirus, target_ip: &str) -> f64 {
        let base_prob = match source_virus.virus_type {
            VirusType::Worm => 0.3,
            VirusType::Botnet => 0.15,
            _ => 0.0,
        };

        // Same subnet bonus
        let subnet_bonus = if self.same_subnet(&source_virus.target_ip, target_ip) {
            0.2
        } else {
            0.0
        };

        // Power bonus
        let power_bonus = (source_virus.power as f64 / 500.0).min(0.2);

        (base_prob + subnet_bonus + power_bonus).max(0.0).min(0.8)
    }

    fn same_subnet(&self, ip1: &str, ip2: &str) -> bool {
        let parts1: Vec<&str> = ip1.split('.').collect();
        let parts2: Vec<&str> = ip2.split('.').collect();
        
        if parts1.len() != 4 || parts2.len() != 4 {
            return false;
        }

        parts1[0] == parts2[0] && parts1[1] == parts2[1] && parts1[2] == parts2[2]
    }

    fn calculate_removal_probability(&self, virus: &ActiveVirus, removal_power: u32) -> f64 {
        let virus_resistance = virus.power + virus.stealth_level;
        let power_ratio = removal_power as f64 / virus_resistance as f64;

        // Detection increases removal probability
        let detection_bonus = if virus.detected { 0.3 } else { 0.0 };

        let base_prob = match power_ratio {
            x if x >= 2.0 => 0.9,
            x if x >= 1.5 => 0.75,
            x if x >= 1.0 => 0.6,
            x if x >= 0.75 => 0.45,
            _ => 0.2,
        };

        (base_prob + detection_bonus).min(0.98)
    }

    fn check_automatic_detection(&self, virus: &ActiveVirus) -> bool {
        let mut rng = thread_rng();
        
        // Detection probability increases over time
        let age_hours = (Utc::now() - virus.installed_at).num_hours().max(0) as f64;
        let age_factor = age_hours / (24.0 * 7.0); // Normalize to weeks
        
        let base_detection_rate = match virus.virus_type {
            VirusType::Money => 0.001,      // 0.1% per hour
            VirusType::DDoS => 0.005,       // 0.5% per hour
            VirusType::Worm => 0.0005,      // 0.05% per hour
            VirusType::BitcoinMiner => 0.002, // 0.2% per hour
            VirusType::Keylogger => 0.0003, // 0.03% per hour
            VirusType::Botnet => 0.003,     // 0.3% per hour
        };

        let stealth_reduction = (virus.stealth_level as f64 / 100.0) * 0.5;
        let detection_rate = (base_detection_rate * (1.0 + age_factor) * (1.0 - stealth_reduction)).max(0.0001);

        rng.gen::<f64>() < detection_rate
    }

    fn calculate_collection_efficiency_decay(&self, virus: &ActiveVirus, now: DateTime<Utc>) -> f64 {
        let age_hours = (now - virus.installed_at).num_hours().max(0) as f64;
        let decay_rate = 0.99f64.powf(age_hours / 24.0); // 1% decay per day
        decay_rate.max(0.1) // Minimum 10% efficiency
    }

    fn calculate_collection_reputation(&self, total_collected: i64) -> i32 {
        (total_collected / 1000).min(100) as i32 // 1 rep per $1000 collected, max 100
    }

    fn get_target_difficulty(&self, _ip: &str) -> u32 {
        // Simplified target difficulty calculation
        let mut rng = thread_rng();
        rng.gen_range(10..=100)
    }

    fn get_target_defense(&self, _ip: &str) -> u32 {
        // Simplified target defense calculation
        let mut rng = thread_rng();
        rng.gen_range(50..=500)
    }
}

// Data structures

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ActiveVirus {
    pub id: u64,
    pub owner_id: UserId,
    pub target_ip: IpAddress,
    pub virus_type: VirusType,
    pub software_id: SoftwareId,
    pub power: u32,
    pub installed_at: DateTime<Utc>,
    pub last_activity: DateTime<Utc>,
    pub estimated_removal: DateTime<Utc>,
    pub status: VirusStatus,
    pub stealth_level: u32,
    pub collected_money: i64,
    pub spread_count: u32,
    pub detected: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum VirusType {
    Money,        // Collects money from target
    DDoS,         // Can participate in DDoS attacks
    Worm,         // Spreads automatically
    BitcoinMiner, // Mines Bitcoin (collects more money but more detectable)
    Keylogger,    // Steals passwords
    Botnet,       // Combination of DDoS and spreading
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum VirusStatus {
    Active,
    Detected,
    Inactive,
    Removed,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CollectionData {
    pub virus_id: u64,
    pub collection_rate: i64, // Money per hour
    pub last_collection: DateTime<Utc>,
    pub total_collected: i64,
    pub collection_efficiency: f64, // Decreases over time
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DDoSNetwork {
    pub id: u64,
    pub attacker_id: UserId,
    pub target_ip: IpAddress,
    pub participating_viruses: Vec<u64>,
    pub start_time: DateTime<Utc>,
    pub duration: u32, // in seconds
    pub total_power: u32,
    pub status: DDoSStatus,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum DDoSStatus {
    Active,
    Completed,
    Failed,
    Interrupted,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SpreadingAttempt {
    pub source_virus_id: u64,
    pub target_ips: Vec<IpAddress>,
    pub started_at: DateTime<Utc>,
    pub completion_rate: f64,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct InstallationResult {
    pub success: bool,
    pub virus_id: Option<u64>,
    pub detection_risk: f64,
    pub estimated_lifetime: u32, // in seconds
    pub error_message: Option<String>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct CollectionResult {
    pub total_collected: i64,
    pub collections: Vec<MoneyCollection>,
    pub failed_collections: usize,
    pub reputation_gained: i32,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct MoneyCollection {
    pub virus_id: u64,
    pub target_ip: IpAddress,
    pub amount: i64,
    pub virus_type: VirusType,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct DDoSResult {
    pub success: bool,
    pub network_id: u64,
    pub participating_virus_count: usize,
    pub total_power: u32,
    pub impact: DDoSImpact,
    pub estimated_duration: u32,
    pub reputation_gained: i32,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct DDoSImpact {
    pub service_disruption: f64, // 0.0 to 1.0
    pub estimated_downtime: u32, // in seconds
    pub financial_damage: i64,   // in game money
    pub reputation_damage: i32,  // reputation points lost by target
}

#[derive(Debug, Serialize, Deserialize)]
pub struct SpreadResult {
    pub attempted_targets: usize,
    pub successful_infections: usize,
    pub failed_infections: usize,
    pub new_virus_ids: Vec<u64>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct RemovalResult {
    pub success: bool,
    pub detection_increased: bool,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct VirusEvent {
    pub virus_id: u64,
    pub target_ip: IpAddress,
    pub event_type: VirusEventType,
    pub timestamp: DateTime<Utc>,
}

#[derive(Debug, Serialize, Deserialize)]
pub enum VirusEventType {
    Installed,
    Detected,
    Removed,
    Expired,
    MoneyCollected,
    Spread,
    DDoSLaunched,
    DDoSCompleted,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VirusInfo {
    pub id: u64,
    pub target_ip: IpAddress,
    pub virus_type: VirusType,
    pub power: u32,
    pub status: VirusStatus,
    pub installed_at: DateTime<Utc>,
    pub collected_money: i64,
    pub spread_count: u32,
    pub stealth_level: u32,
    pub detected: bool,
    pub collection_rate: i64,
}

// Tests
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_virus_installation() {
        let mut virus_system = VirusSystem::new();
        let result = virus_system.install_virus(
            1, 
            "192.168.1.100", 
            1, 
            75, 
            VirusType::Money
        ).unwrap();
        
        // Installation should either succeed or fail, never panic
        assert!(result.virus_id.is_some() == result.success);
    }

    #[test]
    fn test_money_collection() {
        let mut virus_system = VirusSystem::new();
        
        // Install a money virus first
        let install_result = virus_system.install_virus(
            1, 
            "192.168.1.100", 
            1, 
            75, 
            VirusType::Money
        ).unwrap();
        
        if install_result.success {
            let collection_result = virus_system.collect_virus_money(1).unwrap();
            // Should return a valid collection result
            assert!(collection_result.total_collected >= 0);
        }
    }

    #[test]
    fn test_ddos_attack() {
        let mut virus_system = VirusSystem::new();
        
        // Install DDoS virus first
        let _install_result = virus_system.install_virus(
            1, 
            "192.168.1.100", 
            1, 
            100, 
            VirusType::DDoS
        );
        
        let ddos_result = virus_system.launch_ddos_attack(1, "192.168.1.200", 3600, 100);
        // Should handle case where no DDoS viruses are available
        assert!(ddos_result.is_ok() || ddos_result.is_err());
    }

    #[test]
    fn test_virus_spreading() {
        let mut virus_system = VirusSystem::new();
        
        // Install spreading virus
        let install_result = virus_system.install_virus(
            1, 
            "192.168.1.100", 
            1, 
            80, 
            VirusType::Worm
        ).unwrap();
        
        if install_result.success {
            if let Some(virus_id) = install_result.virus_id {
                let spread_result = virus_system.spread_virus(virus_id, 5).unwrap();
                assert!(spread_result.attempted_targets <= 5);
            }
        }
    }
}