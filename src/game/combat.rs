use std::collections::HashMap;
use chrono::{DateTime, Utc, Duration};
use serde::{Deserialize, Serialize};
use rand::{Rng, thread_rng};
use crate::{UserId, IpAddress, SoftwareId, HeResult, HackerExperienceError};

/// Complete combat system with clan wars, DDoS battles, firewall vs cracker calculations
#[derive(Debug, Clone)]
pub struct CombatSystem {
    /// Active battles - battle_id -> Battle
    active_battles: HashMap<u64, Battle>,
    /// Clan war campaigns - war_id -> ClanWar
    clan_wars: HashMap<u64, ClanWar>,
    /// DDoS attack coordination - attack_id -> DDoSAttack
    ddos_attacks: HashMap<u64, DDoSAttack>,
    /// User combat statistics - user_id -> CombatStats
    user_stats: HashMap<UserId, CombatStats>,
    /// Clan combat rankings - clan_id -> ClanCombatRank
    clan_rankings: HashMap<u64, ClanCombatRank>,
    /// Defense matrix configurations - user_id -> DefenseMatrix
    defense_matrices: HashMap<UserId, DefenseMatrix>,
}

impl CombatSystem {
    pub fn new() -> Self {
        Self {
            active_battles: HashMap::new(),
            clan_wars: HashMap::new(),
            ddos_attacks: HashMap::new(),
            user_stats: HashMap::new(),
            clan_rankings: HashMap::new(),
            defense_matrices: HashMap::new(),
        }
    }

    /// Initiate combat between two users
    pub fn initiate_combat(
        &mut self,
        attacker_id: UserId,
        defender_id: UserId,
        attack_type: AttackType,
        attack_power: u32
    ) -> HeResult<CombatResult> {
        if attacker_id == defender_id {
            return Err(HackerExperienceError::CannotAttackSelf);
        }

        // Get defender's defense configuration
        let defense_config = self.get_or_create_defense_matrix(defender_id);
        
        // Calculate combat outcome
        let battle_result = self.calculate_combat_outcome(
            attacker_id,
            defender_id,
            attack_type,
            attack_power,
            &defense_config
        )?;

        // Create battle record
        let mut rng = thread_rng();
        let battle_id = rng.gen::<u64>();
        
        let battle = Battle {
            id: battle_id,
            attacker_id,
            defender_id,
            attack_type: attack_type.clone(),
            attack_power,
            defense_power: defense_config.total_defense_power(),
            outcome: battle_result.outcome.clone(),
            damage_dealt: battle_result.damage_dealt,
            damage_received: battle_result.damage_received,
            started_at: Utc::now(),
            completed_at: Some(Utc::now()),
            reputation_change: battle_result.reputation_change,
            experience_gained: battle_result.experience_gained,
            loot: battle_result.loot.clone(),
        };

        self.active_battles.insert(battle_id, battle);

        // Update user statistics
        self.update_combat_stats(attacker_id, &battle_result, true);
        self.update_combat_stats(defender_id, &battle_result, false);

        // Apply consequences
        self.apply_combat_consequences(&battle_result)?;

        Ok(battle_result)
    }

    /// Launch coordinated DDoS attack
    pub fn launch_ddos_attack(
        &mut self,
        coordinator_id: UserId,
        target_ip: &str,
        participating_users: Vec<UserId>,
        total_power: u32,
        duration: u32
    ) -> HeResult<DDoSResult> {
        let mut rng = thread_rng();
        let attack_id = rng.gen::<u64>();

        // Calculate attack effectiveness
        let network_power = self.calculate_network_attack_power(&participating_users, total_power);
        let target_defense = self.estimate_target_defense(target_ip);
        
        let success_probability = self.calculate_ddos_success_probability(network_power, target_defense);
        let success = rng.gen::<f64>() < success_probability;

        let impact = if success {
            self.calculate_ddos_impact(network_power, target_defense, duration)
        } else {
            DDoSImpact {
                service_disruption: 0.0,
                downtime_duration: 0,
                financial_damage: 0,
                reputation_damage: 0,
                infrastructure_damage: 0,
            }
        };

        // Create DDoS attack record
        let ddos_attack = DDoSAttack {
            id: attack_id,
            coordinator_id,
            target_ip: target_ip.to_string(),
            participating_users: participating_users.clone(),
            total_power: network_power,
            duration,
            started_at: Utc::now(),
            status: if success { DDoSStatus::Active } else { DDoSStatus::Failed },
            impact: impact.clone(),
            detection_risk: self.calculate_ddos_detection_risk(network_power, duration),
        };

        self.ddos_attacks.insert(attack_id, ddos_attack);

        // Distribute rewards/penalties
        let individual_reward = if success { 
            (network_power / participating_users.len() as u32) / 10 
        } else { 
            -(network_power / participating_users.len() as u32) / 20 
        } as i32;

        for user_id in &participating_users {
            self.update_ddos_participation_stats(*user_id, success, individual_reward);
        }

        Ok(DDoSResult {
            attack_id,
            success,
            network_power,
            target_defense,
            impact,
            participating_count: participating_users.len(),
            individual_reputation_change: individual_reward,
            detection_risk: ddos_attack.detection_risk,
        })
    }

    /// Declare clan war
    pub fn declare_clan_war(
        &mut self,
        declaring_clan_id: u64,
        target_clan_id: u64,
        war_type: WarType,
        duration_hours: u32
    ) -> HeResult<ClanWarResult> {
        if declaring_clan_id == target_clan_id {
            return Err(HackerExperienceError::CannotWarSelf);
        }

        let mut rng = thread_rng();
        let war_id = rng.gen::<u64>();

        let clan_war = ClanWar {
            id: war_id,
            declaring_clan_id,
            target_clan_id,
            war_type: war_type.clone(),
            started_at: Utc::now(),
            ends_at: Utc::now() + Duration::hours(duration_hours as i64),
            status: WarStatus::Active,
            declaring_clan_score: 0,
            target_clan_score: 0,
            battles: Vec::new(),
            objectives: self.generate_war_objectives(&war_type),
            rewards: self.calculate_war_rewards(&war_type, duration_hours),
        };

        self.clan_wars.insert(war_id, clan_war.clone());

        Ok(ClanWarResult {
            war_id,
            declaring_clan_id,
            target_clan_id,
            war_type,
            duration_hours,
            objectives: clan_war.objectives,
            estimated_rewards: clan_war.rewards,
        })
    }

    /// Calculate firewall vs cracker effectiveness
    pub fn calculate_firewall_effectiveness(
        &self,
        firewall_power: u32,
        firewall_type: FirewallType,
        cracker_power: u32,
        cracker_type: CrackerType,
        attack_method: AttackMethod
    ) -> FirewallAnalysis {
        // Base effectiveness calculations
        let firewall_base_strength = self.get_firewall_base_strength(&firewall_type);
        let cracker_base_effectiveness = self.get_cracker_base_effectiveness(&cracker_type);

        // Method-specific modifiers
        let method_modifier = self.get_attack_method_modifier(&attack_method, &firewall_type);
        
        // Power ratio calculation
        let effective_firewall_power = (firewall_power as f64 * firewall_base_strength * method_modifier) as u32;
        let effective_cracker_power = (cracker_power as f64 * cracker_base_effectiveness) as u32;

        let power_ratio = effective_cracker_power as f64 / effective_firewall_power as f64;

        // Calculate probabilities
        let penetration_probability = self.calculate_penetration_probability(power_ratio);
        let detection_probability = self.calculate_detection_probability(power_ratio, &attack_method);
        let damage_potential = self.calculate_damage_potential(power_ratio);

        // Time calculations
        let estimated_breach_time = self.calculate_breach_time(effective_firewall_power, effective_cracker_power);
        let detection_time = self.calculate_detection_time(detection_probability, estimated_breach_time);

        FirewallAnalysis {
            firewall_strength: effective_firewall_power,
            cracker_effectiveness: effective_cracker_power,
            penetration_probability,
            detection_probability,
            damage_potential,
            estimated_breach_time,
            detection_time,
            recommended_defense: self.recommend_defense_upgrade(&firewall_type, power_ratio),
            vulnerability_assessment: self.assess_vulnerabilities(&firewall_type, &attack_method),
        }
    }

    /// Configure user's defense matrix
    pub fn configure_defense_matrix(
        &mut self,
        user_id: UserId,
        firewall_config: FirewallConfig,
        intrusion_detection: IntrusionDetectionConfig,
        response_protocols: Vec<ResponseProtocol>
    ) -> HeResult<DefenseMatrix> {
        let defense_matrix = DefenseMatrix {
            user_id,
            firewall_config,
            intrusion_detection,
            response_protocols,
            last_updated: Utc::now(),
            effectiveness_rating: self.calculate_matrix_effectiveness(&firewall_config, &intrusion_detection, &response_protocols),
            active_threats: Vec::new(),
        };

        self.defense_matrices.insert(user_id, defense_matrix.clone());
        Ok(defense_matrix)
    }

    /// Process ongoing clan wars
    pub fn process_clan_wars(&mut self) -> HeResult<Vec<WarUpdate>> {
        let mut updates = Vec::new();
        let now = Utc::now();

        for (war_id, clan_war) in &mut self.clan_wars {
            if clan_war.status == WarStatus::Active && now >= clan_war.ends_at {
                // Determine winner
                let winner = if clan_war.declaring_clan_score > clan_war.target_clan_score {
                    Some(clan_war.declaring_clan_id)
                } else if clan_war.target_clan_score > clan_war.declaring_clan_score {
                    Some(clan_war.target_clan_id)
                } else {
                    None // Draw
                };

                clan_war.status = WarStatus::Completed;

                updates.push(WarUpdate {
                    war_id: *war_id,
                    update_type: WarUpdateType::Completed,
                    winner_clan_id: winner,
                    final_scores: (clan_war.declaring_clan_score, clan_war.target_clan_score),
                    rewards_distributed: self.distribute_war_rewards(clan_war)?,
                });

                // Update clan rankings
                self.update_clan_rankings_after_war(clan_war);
            }
        }

        Ok(updates)
    }

    /// Get user's combat statistics
    pub fn get_user_combat_stats(&self, user_id: UserId) -> Option<&CombatStats> {
        self.user_stats.get(&user_id)
    }

    /// Get clan combat rankings
    pub fn get_clan_rankings(&self) -> Vec<ClanCombatRank> {
        let mut rankings: Vec<ClanCombatRank> = self.clan_rankings.values().cloned().collect();
        rankings.sort_by(|a, b| b.total_score.cmp(&a.total_score));
        rankings
    }

    /// Simulate defense against incoming attack
    pub fn simulate_defense(
        &self,
        defender_id: UserId,
        attack_power: u32,
        attack_type: AttackType
    ) -> DefenseSimulation {
        let defense_matrix = self.defense_matrices.get(&defender_id)
            .cloned()
            .unwrap_or_else(|| self.create_default_defense_matrix(defender_id));

        let detection_chance = self.calculate_detection_chance(&defense_matrix, &attack_type);
        let mitigation_effectiveness = self.calculate_mitigation_effectiveness(&defense_matrix, attack_power);
        let response_time = self.calculate_response_time(&defense_matrix);

        let estimated_damage = if detection_chance > 0.7 {
            (attack_power as f64 * (1.0 - mitigation_effectiveness)).max(0.0) as u32
        } else {
            attack_power // Full damage if undetected
        };

        DefenseSimulation {
            detection_chance,
            mitigation_effectiveness,
            response_time,
            estimated_damage,
            recommended_actions: self.generate_defense_recommendations(&defense_matrix, &attack_type),
        }
    }

    // Private helper methods

    fn get_or_create_defense_matrix(&mut self, user_id: UserId) -> DefenseMatrix {
        self.defense_matrices.get(&user_id)
            .cloned()
            .unwrap_or_else(|| {
                let matrix = self.create_default_defense_matrix(user_id);
                self.defense_matrices.insert(user_id, matrix.clone());
                matrix
            })
    }

    fn create_default_defense_matrix(&self, user_id: UserId) -> DefenseMatrix {
        DefenseMatrix {
            user_id,
            firewall_config: FirewallConfig {
                firewall_type: FirewallType::Basic,
                power: 50,
                rules: vec![
                    FirewallRule {
                        rule_type: RuleType::Block,
                        condition: "suspicious_ips".to_string(),
                        action: "drop".to_string(),
                    }
                ],
                update_frequency: Duration::hours(24),
            },
            intrusion_detection: IntrusionDetectionConfig {
                sensitivity: DetectionSensitivity::Medium,
                monitoring_scope: vec![MonitoringScope::NetworkTraffic, MonitoringScope::SystemLogs],
                alert_threshold: 3,
                response_delay: Duration::minutes(5),
            },
            response_protocols: vec![
                ResponseProtocol {
                    trigger: TriggerCondition::HighThreatDetected,
                    action: ResponseAction::AutoBlock,
                    priority: 1,
                    enabled: true,
                }
            ],
            last_updated: Utc::now(),
            effectiveness_rating: 0.6, // 60% effectiveness for default
            active_threats: Vec::new(),
        }
    }

    fn calculate_combat_outcome(
        &self,
        attacker_id: UserId,
        defender_id: UserId,
        attack_type: AttackType,
        attack_power: u32,
        defense_config: &DefenseMatrix
    ) -> HeResult<CombatResult> {
        let mut rng = thread_rng();

        // Get attacker's combat stats for bonuses
        let attacker_stats = self.user_stats.get(&attacker_id);
        let attack_bonus = attacker_stats.map(|s| s.attack_bonus).unwrap_or(1.0);

        // Calculate effective powers
        let effective_attack_power = (attack_power as f64 * attack_bonus) as u32;
        let defense_power = defense_config.total_defense_power();

        // Determine outcome based on power ratio and attack type
        let power_ratio = effective_attack_power as f64 / defense_power as f64;
        let success_probability = self.calculate_attack_success_probability(power_ratio, &attack_type);

        let success = rng.gen::<f64>() < success_probability;

        let (outcome, damage_dealt, damage_received, reputation_change, experience_gained, loot) = 
            if success {
                let damage = self.calculate_damage(effective_attack_power, defense_power);
                let rep_gain = self.calculate_reputation_gain(&attack_type, damage);
                let exp_gain = self.calculate_experience_gain(&attack_type, damage);
                let loot_items = self.generate_loot(&attack_type, damage);

                (
                    BattleOutcome::AttackerVictory,
                    damage,
                    rng.gen_range(0..=(damage / 4)), // Attacker takes some damage
                    rep_gain,
                    exp_gain,
                    loot_items
                )
            } else {
                let counter_damage = self.calculate_counter_damage(defense_power, effective_attack_power);
                (
                    BattleOutcome::DefenderVictory,
                    rng.gen_range(0..=(defense_power / 10)), // Minimal damage to defender
                    counter_damage,
                    -(rep_gain / 2), // Reputation penalty for failed attack
                    exp_gain / 4,   // Minimal experience for attempt
                    Vec::new()
                )
            };

        Ok(CombatResult {
            attacker_id,
            defender_id,
            attack_type,
            outcome,
            damage_dealt,
            damage_received,
            reputation_change,
            experience_gained,
            loot,
            battle_duration: rng.gen_range(30..=300), // 30 seconds to 5 minutes
        })
    }

    fn calculate_attack_success_probability(&self, power_ratio: f64, attack_type: &AttackType) -> f64 {
        let base_probability = match power_ratio {
            x if x >= 3.0 => 0.90,
            x if x >= 2.0 => 0.75,
            x if x >= 1.5 => 0.60,
            x if x >= 1.0 => 0.45,
            x if x >= 0.75 => 0.30,
            x if x >= 0.5 => 0.15,
            _ => 0.05,
        };

        // Attack type modifiers
        let type_modifier = match attack_type {
            AttackType::Hack => 0.0,
            AttackType::Exploit => 0.1,   // Exploits are more effective
            AttackType::Virus => -0.05,   // Viruses are easier to detect
            AttackType::DDoS => 0.05,     // DDoS has raw power advantage
            AttackType::Social => 0.15,   // Social engineering bypasses tech defenses
        };

        (base_probability + type_modifier).max(0.01).min(0.95)
    }

    fn calculate_damage(&self, attack_power: u32, defense_power: u32) -> u32 {
        let base_damage = attack_power.saturating_sub(defense_power / 2);
        let mut rng = thread_rng();
        let variance = rng.gen_range(0.8..=1.2);
        ((base_damage as f64) * variance) as u32
    }

    fn calculate_counter_damage(&self, defense_power: u32, attack_power: u32) -> u32 {
        // Counter-attack damage is based on defense power
        let base_counter = defense_power / 4;
        let counter_efficiency = (defense_power as f64 / attack_power as f64).min(2.0);
        (base_counter as f64 * counter_efficiency) as u32
    }

    fn calculate_reputation_gain(&self, attack_type: &AttackType, damage: u32) -> i32 {
        let base_gain = (damage / 10) as i32;
        
        let type_multiplier = match attack_type {
            AttackType::Hack => 1.0,
            AttackType::Exploit => 1.5,
            AttackType::Virus => 0.8,
            AttackType::DDoS => 1.2,
            AttackType::Social => 2.0, // High skill requirement
        };

        ((base_gain as f64) * type_multiplier) as i32
    }

    fn calculate_experience_gain(&self, attack_type: &AttackType, damage: u32) -> i32 {
        let base_exp = (damage / 5) as i32;
        
        let type_multiplier = match attack_type {
            AttackType::Hack => 1.0,
            AttackType::Exploit => 1.3,
            AttackType::Virus => 1.1,
            AttackType::DDoS => 0.9,
            AttackType::Social => 1.8,
        };

        ((base_exp as f64) * type_multiplier) as i32
    }

    fn generate_loot(&self, attack_type: &AttackType, damage: u32) -> Vec<LootItem> {
        let mut loot = Vec::new();
        let mut rng = thread_rng();

        // Chance of getting loot based on damage
        let loot_chance = (damage as f64 / 1000.0).min(0.8);
        
        if rng.gen::<f64>() < loot_chance {
            match attack_type {
                AttackType::Hack => {
                    if rng.gen_bool(0.3) {
                        loot.push(LootItem {
                            item_type: LootType::Money,
                            value: rng.gen_range(100..=(damage as i64)),
                            description: "Stolen digital funds".to_string(),
                        });
                    }
                },
                AttackType::Virus => {
                    if rng.gen_bool(0.2) {
                        loot.push(LootItem {
                            item_type: LootType::Information,
                            value: rng.gen_range(1..=5),
                            description: "Collected system information".to_string(),
                        });
                    }
                },
                AttackType::Exploit => {
                    if rng.gen_bool(0.4) {
                        loot.push(LootItem {
                            item_type: LootType::Software,
                            value: rng.gen_range(1..=3),
                            description: "Captured security tools".to_string(),
                        });
                    }
                },
                _ => {},
            }
        }

        loot
    }

    fn update_combat_stats(&mut self, user_id: UserId, battle_result: &CombatResult, is_attacker: bool) {
        let stats = self.user_stats.entry(user_id).or_insert_with(|| CombatStats::new(user_id));

        if is_attacker {
            stats.attacks_initiated += 1;
            match battle_result.outcome {
                BattleOutcome::AttackerVictory => {
                    stats.attacks_won += 1;
                    stats.total_damage_dealt += battle_result.damage_dealt as i64;
                },
                _ => {
                    stats.attacks_lost += 1;
                }
            }
        } else {
            stats.defenses += 1;
            match battle_result.outcome {
                BattleOutcome::DefenderVictory => stats.defenses_successful += 1,
                _ => {}
            }
        }

        stats.reputation += battle_result.reputation_change as i64;
        stats.total_experience += battle_result.experience_gained as i64;
        stats.total_damage_received += battle_result.damage_received as i64;

        // Update derived stats
        stats.attack_success_rate = if stats.attacks_initiated > 0 {
            stats.attacks_won as f64 / stats.attacks_initiated as f64
        } else {
            0.0
        };

        stats.defense_success_rate = if stats.defenses > 0 {
            stats.defenses_successful as f64 / stats.defenses as f64
        } else {
            0.0
        };

        // Calculate attack bonus based on experience and success rate
        stats.attack_bonus = 1.0 + (stats.total_experience as f64 / 10000.0) * stats.attack_success_rate;
    }

    fn apply_combat_consequences(&mut self, _battle_result: &CombatResult) -> HeResult<()> {
        // Apply any additional consequences of combat
        // In a real implementation, this might affect hardware condition,
        // trigger security alerts, etc.
        Ok(())
    }

    fn calculate_network_attack_power(&self, users: &[UserId], base_power: u32) -> u32 {
        let coordination_efficiency = match users.len() {
            1 => 1.0,
            2..=5 => 0.95,
            6..=10 => 0.85,
            11..=20 => 0.75,
            _ => 0.6, // Large groups have coordination overhead
        };

        // Factor in individual user skills
        let skill_multiplier = users.iter()
            .filter_map(|&user_id| self.user_stats.get(&user_id))
            .map(|stats| stats.attack_bonus)
            .sum::<f64>() / users.len() as f64;

        ((base_power as f64) * coordination_efficiency * skill_multiplier) as u32
    }

    fn estimate_target_defense(&self, _target_ip: &str) -> u32 {
        // Simplified target defense estimation
        // In reality, would use reconnaissance data
        let mut rng = thread_rng();
        rng.gen_range(100..=1000)
    }

    fn calculate_ddos_success_probability(&self, attack_power: u32, defense_power: u32) -> f64 {
        let power_ratio = attack_power as f64 / defense_power as f64;
        
        match power_ratio {
            x if x >= 5.0 => 0.95,
            x if x >= 3.0 => 0.85,
            x if x >= 2.0 => 0.70,
            x if x >= 1.5 => 0.55,
            x if x >= 1.0 => 0.40,
            x if x >= 0.5 => 0.20,
            _ => 0.05,
        }
    }

    fn calculate_ddos_impact(&self, attack_power: u32, defense_power: u32, duration: u32) -> DDoSImpact {
        let power_ratio = attack_power as f64 / defense_power as f64;
        let service_disruption = (power_ratio / 3.0).min(1.0);
        
        DDoSImpact {
            service_disruption,
            downtime_duration: (duration as f64 * service_disruption) as u32,
            financial_damage: (attack_power as i64 * duration as i64 / 100).max(1000),
            reputation_damage: (attack_power / 50) as i32,
            infrastructure_damage: (attack_power / 100) as i32,
        }
    }

    fn calculate_ddos_detection_risk(&self, attack_power: u32, duration: u32) -> f64 {
        let base_risk = 0.1; // 10% base detection risk
        let power_factor = (attack_power as f64 / 1000.0) * 0.2; // Higher power = higher risk
        let duration_factor = (duration as f64 / 3600.0) * 0.3; // Longer attacks = higher risk
        
        (base_risk + power_factor + duration_factor).min(0.9)
    }

    fn update_ddos_participation_stats(&mut self, user_id: UserId, success: bool, reputation_change: i32) {
        let stats = self.user_stats.entry(user_id).or_insert_with(|| CombatStats::new(user_id));
        
        stats.ddos_participated += 1;
        if success {
            stats.ddos_successful += 1;
        }
        stats.reputation += reputation_change as i64;
    }

    fn generate_war_objectives(&self, war_type: &WarType) -> Vec<WarObjective> {
        match war_type {
            WarType::Domination => vec![
                WarObjective {
                    id: 1,
                    description: "Win 10 individual battles".to_string(),
                    target_value: 10,
                    current_progress: 0,
                    points_reward: 100,
                },
                WarObjective {
                    id: 2,
                    description: "Deal 50,000 total damage".to_string(),
                    target_value: 50000,
                    current_progress: 0,
                    points_reward: 150,
                },
            ],
            WarType::Economic => vec![
                WarObjective {
                    id: 1,
                    description: "Steal $100,000 in total".to_string(),
                    target_value: 100000,
                    current_progress: 0,
                    points_reward: 200,
                },
            ],
            WarType::Sabotage => vec![
                WarObjective {
                    id: 1,
                    description: "Successfully deploy 20 viruses".to_string(),
                    target_value: 20,
                    current_progress: 0,
                    points_reward: 120,
                },
            ],
        }
    }

    fn calculate_war_rewards(&self, war_type: &WarType, duration_hours: u32) -> WarRewards {
        let base_money = match war_type {
            WarType::Domination => 50000,
            WarType::Economic => 100000,
            WarType::Sabotage => 30000,
        };

        let duration_multiplier = (duration_hours as f64 / 24.0).min(3.0); // Max 3x for long wars

        WarRewards {
            winner_money: (base_money as f64 * duration_multiplier) as i64,
            winner_reputation: (1000.0 * duration_multiplier) as i32,
            participant_money: (base_money as f64 * 0.2 * duration_multiplier) as i64,
            participant_reputation: (200.0 * duration_multiplier) as i32,
        }
    }

    fn distribute_war_rewards(&mut self, _clan_war: &ClanWar) -> HeResult<bool> {
        // Distribute rewards to clan members
        // Implementation would involve actual clan member lookup and reward distribution
        Ok(true)
    }

    fn update_clan_rankings_after_war(&mut self, clan_war: &ClanWar) {
        // Update clan rankings based on war performance
        let winner_points = 100;
        let loser_points = -50;

        let (winner_clan, loser_clan) = if clan_war.declaring_clan_score > clan_war.target_clan_score {
            (clan_war.declaring_clan_id, clan_war.target_clan_id)
        } else {
            (clan_war.target_clan_id, clan_war.declaring_clan_id)
        };

        // Update winner
        let winner_rank = self.clan_rankings.entry(winner_clan).or_insert_with(|| ClanCombatRank {
            clan_id: winner_clan,
            total_score: 1000,
            wars_won: 0,
            wars_lost: 0,
            wars_participated: 0,
            last_activity: Utc::now(),
            rank_position: 0,
        });
        winner_rank.total_score += winner_points;
        winner_rank.wars_won += 1;
        winner_rank.wars_participated += 1;
        winner_rank.last_activity = Utc::now();

        // Update loser
        let loser_rank = self.clan_rankings.entry(loser_clan).or_insert_with(|| ClanCombatRank {
            clan_id: loser_clan,
            total_score: 1000,
            wars_won: 0,
            wars_lost: 0,
            wars_participated: 0,
            last_activity: Utc::now(),
            rank_position: 0,
        });
        loser_rank.total_score = (loser_rank.total_score + loser_points).max(0);
        loser_rank.wars_lost += 1;
        loser_rank.wars_participated += 1;
        loser_rank.last_activity = Utc::now();
    }

    // Firewall analysis methods

    fn get_firewall_base_strength(&self, firewall_type: &FirewallType) -> f64 {
        match firewall_type {
            FirewallType::Basic => 1.0,
            FirewallType::Advanced => 1.5,
            FirewallType::Enterprise => 2.0,
            FirewallType::Military => 3.0,
            FirewallType::Custom => 2.5,
        }
    }

    fn get_cracker_base_effectiveness(&self, cracker_type: &CrackerType) -> f64 {
        match cracker_type {
            CrackerType::Basic => 1.0,
            CrackerType::Advanced => 1.3,
            CrackerType::Professional => 1.6,
            CrackerType::Elite => 2.0,
            CrackerType::Custom => 1.8,
        }
    }

    fn get_attack_method_modifier(&self, method: &AttackMethod, firewall_type: &FirewallType) -> f64 {
        match (method, firewall_type) {
            (AttackMethod::BruteForce, FirewallType::Basic) => 1.2,
            (AttackMethod::BruteForce, _) => 0.8,
            (AttackMethod::Exploit, FirewallType::Advanced) => 1.3,
            (AttackMethod::Exploit, _) => 1.0,
            (AttackMethod::SocialEngineering, _) => 1.5, // Always effective
            (AttackMethod::ZeroDay, _) => 2.0, // Very effective
            (AttackMethod::NetworkScan, _) => 0.9,
        }
    }

    fn calculate_penetration_probability(&self, power_ratio: f64) -> f64 {
        match power_ratio {
            x if x >= 3.0 => 0.95,
            x if x >= 2.0 => 0.85,
            x if x >= 1.5 => 0.70,
            x if x >= 1.0 => 0.50,
            x if x >= 0.75 => 0.30,
            x if x >= 0.5 => 0.15,
            _ => 0.05,
        }
    }

    fn calculate_detection_probability(&self, power_ratio: f64, method: &AttackMethod) -> f64 {
        let base_detection = match method {
            AttackMethod::BruteForce => 0.8,
            AttackMethod::NetworkScan => 0.9,
            AttackMethod::Exploit => 0.4,
            AttackMethod::SocialEngineering => 0.1,
            AttackMethod::ZeroDay => 0.2,
        };

        // Lower power ratio = higher detection chance
        let power_modifier = if power_ratio < 1.0 {
            (1.0 - power_ratio) * 0.3
        } else {
            -(power_ratio - 1.0) * 0.2
        };

        (base_detection + power_modifier).max(0.05).min(0.95)
    }

    fn calculate_damage_potential(&self, power_ratio: f64) -> f64 {
        (power_ratio / 2.0).min(1.0)
    }

    fn calculate_breach_time(&self, firewall_power: u32, cracker_power: u32) -> u32 {
        let base_time = firewall_power * 10; // 10 seconds per firewall power unit
        let cracker_efficiency = cracker_power as f64 / firewall_power as f64;
        
        if cracker_efficiency >= 1.0 {
            (base_time as f64 / cracker_efficiency) as u32
        } else {
            base_time * 2 // Penalty for underpowered crackers
        }
    }

    fn calculate_detection_time(&self, detection_prob: f64, breach_time: u32) -> Option<u32> {
        if detection_prob > 0.5 {
            Some((breach_time as f64 * (1.0 - detection_prob) * 0.8) as u32)
        } else {
            None // Low chance of detection
        }
    }

    fn recommend_defense_upgrade(&self, current_type: &FirewallType, power_ratio: f64) -> Vec<String> {
        let mut recommendations = Vec::new();

        if power_ratio > 1.5 {
            recommendations.push("Consider upgrading firewall type".to_string());
            recommendations.push("Increase firewall power".to_string());
        }

        if power_ratio > 2.0 {
            recommendations.push("Implement intrusion detection system".to_string());
            recommendations.push("Add redundant security layers".to_string());
        }

        match current_type {
            FirewallType::Basic => {
                recommendations.push("Upgrade to Advanced firewall".to_string());
            },
            FirewallType::Advanced if power_ratio > 1.0 => {
                recommendations.push("Consider Enterprise-grade solution".to_string());
            },
            _ => {},
        }

        recommendations
    }

    fn assess_vulnerabilities(&self, firewall_type: &FirewallType, attack_method: &AttackMethod) -> Vec<String> {
        let mut vulnerabilities = Vec::new();

        match (firewall_type, attack_method) {
            (FirewallType::Basic, AttackMethod::BruteForce) => {
                vulnerabilities.push("Weak against brute force attacks".to_string());
            },
            (_, AttackMethod::SocialEngineering) => {
                vulnerabilities.push("Social engineering bypasses technical defenses".to_string());
            },
            (_, AttackMethod::ZeroDay) => {
                vulnerabilities.push("Unknown exploits cannot be defended against".to_string());
            },
            _ => {},
        }

        vulnerabilities
    }

    fn calculate_matrix_effectiveness(
        &self,
        firewall: &FirewallConfig,
        ids: &IntrusionDetectionConfig,
        protocols: &[ResponseProtocol]
    ) -> f64 {
        let firewall_score = match firewall.firewall_type {
            FirewallType::Basic => 0.3,
            FirewallType::Advanced => 0.5,
            FirewallType::Enterprise => 0.7,
            FirewallType::Military => 0.9,
            FirewallType::Custom => 0.6,
        };

        let ids_score = match ids.sensitivity {
            DetectionSensitivity::Low => 0.2,
            DetectionSensitivity::Medium => 0.5,
            DetectionSensitivity::High => 0.8,
            DetectionSensitivity::Maximum => 0.95,
        };

        let protocol_score = protocols.len() as f64 * 0.1;

        ((firewall_score + ids_score + protocol_score) / 3.0).min(1.0)
    }

    fn calculate_detection_chance(&self, matrix: &DefenseMatrix, attack_type: &AttackType) -> f64 {
        let base_chance = match matrix.intrusion_detection.sensitivity {
            DetectionSensitivity::Low => 0.2,
            DetectionSensitivity::Medium => 0.5,
            DetectionSensitivity::High => 0.8,
            DetectionSensitivity::Maximum => 0.95,
        };

        let attack_modifier = match attack_type {
            AttackType::Hack => 0.0,
            AttackType::Exploit => -0.1,
            AttackType::Virus => 0.1,
            AttackType::DDoS => 0.2,
            AttackType::Social => -0.3,
        };

        (base_chance + attack_modifier).max(0.05).min(0.95)
    }

    fn calculate_mitigation_effectiveness(&self, matrix: &DefenseMatrix, attack_power: u32) -> f64 {
        let firewall_effectiveness = match matrix.firewall_config.firewall_type {
            FirewallType::Basic => 0.3,
            FirewallType::Advanced => 0.5,
            FirewallType::Enterprise => 0.7,
            FirewallType::Military => 0.9,
            FirewallType::Custom => 0.6,
        };

        let power_factor = (matrix.firewall_config.power as f64 / attack_power as f64).min(2.0);
        firewall_effectiveness * power_factor
    }

    fn calculate_response_time(&self, matrix: &DefenseMatrix) -> u32 {
        let base_time = matrix.intrusion_detection.response_delay.num_seconds() as u32;
        let protocol_efficiency = matrix.response_protocols.len() as f64 * 0.1;
        
        (base_time as f64 / (1.0 + protocol_efficiency)) as u32
    }

    fn generate_defense_recommendations(&self, matrix: &DefenseMatrix, attack_type: &AttackType) -> Vec<String> {
        let mut recommendations = Vec::new();

        if matrix.effectiveness_rating < 0.7 {
            recommendations.push("Upgrade firewall to higher tier".to_string());
            recommendations.push("Increase intrusion detection sensitivity".to_string());
        }

        match attack_type {
            AttackType::DDoS => {
                recommendations.push("Implement DDoS protection service".to_string());
                recommendations.push("Configure rate limiting".to_string());
            },
            AttackType::Social => {
                recommendations.push("Implement user education programs".to_string());
                recommendations.push("Add verification procedures".to_string());
            },
            AttackType::Virus => {
                recommendations.push("Update antivirus signatures".to_string());
                recommendations.push("Implement behavioral analysis".to_string());
            },
            _ => {},
        }

        recommendations
    }
}

// Data structures (continued in next part due to length)

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Battle {
    pub id: u64,
    pub attacker_id: UserId,
    pub defender_id: UserId,
    pub attack_type: AttackType,
    pub attack_power: u32,
    pub defense_power: u32,
    pub outcome: BattleOutcome,
    pub damage_dealt: u32,
    pub damage_received: u32,
    pub started_at: DateTime<Utc>,
    pub completed_at: Option<DateTime<Utc>>,
    pub reputation_change: i32,
    pub experience_gained: i32,
    pub loot: Vec<LootItem>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum AttackType {
    Hack,
    Exploit,
    Virus,
    DDoS,
    Social,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum BattleOutcome {
    AttackerVictory,
    DefenderVictory,
    Draw,
    Interrupted,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CombatResult {
    pub attacker_id: UserId,
    pub defender_id: UserId,
    pub attack_type: AttackType,
    pub outcome: BattleOutcome,
    pub damage_dealt: u32,
    pub damage_received: u32,
    pub reputation_change: i32,
    pub experience_gained: i32,
    pub loot: Vec<LootItem>,
    pub battle_duration: u32, // in seconds
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LootItem {
    pub item_type: LootType,
    pub value: i64,
    pub description: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum LootType {
    Money,
    Software,
    Hardware,
    Information,
    Reputation,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CombatStats {
    pub user_id: UserId,
    pub attacks_initiated: u32,
    pub attacks_won: u32,
    pub attacks_lost: u32,
    pub defenses: u32,
    pub defenses_successful: u32,
    pub ddos_participated: u32,
    pub ddos_successful: u32,
    pub total_damage_dealt: i64,
    pub total_damage_received: i64,
    pub reputation: i64,
    pub total_experience: i64,
    pub attack_success_rate: f64,
    pub defense_success_rate: f64,
    pub attack_bonus: f64, // Multiplier based on experience
}

impl CombatStats {
    pub fn new(user_id: UserId) -> Self {
        Self {
            user_id,
            attacks_initiated: 0,
            attacks_won: 0,
            attacks_lost: 0,
            defenses: 0,
            defenses_successful: 0,
            ddos_participated: 0,
            ddos_successful: 0,
            total_damage_dealt: 0,
            total_damage_received: 0,
            reputation: 1000, // Starting reputation
            total_experience: 0,
            attack_success_rate: 0.0,
            defense_success_rate: 0.0,
            attack_bonus: 1.0,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ClanWar {
    pub id: u64,
    pub declaring_clan_id: u64,
    pub target_clan_id: u64,
    pub war_type: WarType,
    pub started_at: DateTime<Utc>,
    pub ends_at: DateTime<Utc>,
    pub status: WarStatus,
    pub declaring_clan_score: u32,
    pub target_clan_score: u32,
    pub battles: Vec<u64>, // Battle IDs
    pub objectives: Vec<WarObjective>,
    pub rewards: WarRewards,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum WarType {
    Domination,  // Most battles won
    Economic,    // Most money stolen
    Sabotage,    // Most infrastructure damage
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum WarStatus {
    Active,
    Completed,
    Cancelled,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WarObjective {
    pub id: u64,
    pub description: String,
    pub target_value: i64,
    pub current_progress: i64,
    pub points_reward: u32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WarRewards {
    pub winner_money: i64,
    pub winner_reputation: i32,
    pub participant_money: i64,
    pub participant_reputation: i32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DDoSAttack {
    pub id: u64,
    pub coordinator_id: UserId,
    pub target_ip: IpAddress,
    pub participating_users: Vec<UserId>,
    pub total_power: u32,
    pub duration: u32, // in seconds
    pub started_at: DateTime<Utc>,
    pub status: DDoSStatus,
    pub impact: DDoSImpact,
    pub detection_risk: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum DDoSStatus {
    Active,
    Completed,
    Failed,
    Detected,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DDoSImpact {
    pub service_disruption: f64, // 0.0 to 1.0
    pub downtime_duration: u32,  // in seconds
    pub financial_damage: i64,
    pub reputation_damage: i32,
    pub infrastructure_damage: i32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ClanCombatRank {
    pub clan_id: u64,
    pub total_score: i32,
    pub wars_won: u32,
    pub wars_lost: u32,
    pub wars_participated: u32,
    pub last_activity: DateTime<Utc>,
    pub rank_position: u32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DefenseMatrix {
    pub user_id: UserId,
    pub firewall_config: FirewallConfig,
    pub intrusion_detection: IntrusionDetectionConfig,
    pub response_protocols: Vec<ResponseProtocol>,
    pub last_updated: DateTime<Utc>,
    pub effectiveness_rating: f64, // 0.0 to 1.0
    pub active_threats: Vec<ThreatAlert>,
}

impl DefenseMatrix {
    pub fn total_defense_power(&self) -> u32 {
        let firewall_power = self.firewall_config.power;
        let ids_power = match self.intrusion_detection.sensitivity {
            DetectionSensitivity::Low => 10,
            DetectionSensitivity::Medium => 25,
            DetectionSensitivity::High => 50,
            DetectionSensitivity::Maximum => 100,
        };
        let protocol_power = self.response_protocols.len() as u32 * 5;
        
        firewall_power + ids_power + protocol_power
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FirewallConfig {
    pub firewall_type: FirewallType,
    pub power: u32,
    pub rules: Vec<FirewallRule>,
    pub update_frequency: Duration,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum FirewallType {
    Basic,
    Advanced,
    Enterprise,
    Military,
    Custom,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FirewallRule {
    pub rule_type: RuleType,
    pub condition: String,
    pub action: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum RuleType {
    Allow,
    Block,
    Log,
    Redirect,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct IntrusionDetectionConfig {
    pub sensitivity: DetectionSensitivity,
    pub monitoring_scope: Vec<MonitoringScope>,
    pub alert_threshold: u32,
    pub response_delay: Duration,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum DetectionSensitivity {
    Low,
    Medium,
    High,
    Maximum,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum MonitoringScope {
    NetworkTraffic,
    SystemLogs,
    FileSystem,
    ProcessActivity,
    UserBehavior,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ResponseProtocol {
    pub trigger: TriggerCondition,
    pub action: ResponseAction,
    pub priority: u32,
    pub enabled: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum TriggerCondition {
    HighThreatDetected,
    MultipleFailedLogins,
    SuspiciousNetworkActivity,
    UnauthorizedFileAccess,
    SystemResourceAbnormality,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ResponseAction {
    AutoBlock,
    AlertAdmin,
    IncreaseMonitoring,
    IsolateSystem,
    CounterAttack,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ThreatAlert {
    pub id: u64,
    pub threat_type: String,
    pub severity: ThreatSeverity,
    pub detected_at: DateTime<Utc>,
    pub source_ip: Option<IpAddress>,
    pub description: String,
    pub status: ThreatStatus,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ThreatSeverity {
    Low,
    Medium,
    High,
    Critical,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ThreatStatus {
    Active,
    Mitigated,
    Resolved,
    Ignored,
}

// Firewall analysis structures

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FirewallAnalysis {
    pub firewall_strength: u32,
    pub cracker_effectiveness: u32,
    pub penetration_probability: f64,
    pub detection_probability: f64,
    pub damage_potential: f64,
    pub estimated_breach_time: u32, // in seconds
    pub detection_time: Option<u32>, // in seconds, None if unlikely to detect
    pub recommended_defense: Vec<String>,
    pub vulnerability_assessment: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum CrackerType {
    Basic,
    Advanced,
    Professional,
    Elite,
    Custom,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum AttackMethod {
    BruteForce,
    Exploit,
    SocialEngineering,
    ZeroDay,
    NetworkScan,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct DefenseSimulation {
    pub detection_chance: f64,
    pub mitigation_effectiveness: f64,
    pub response_time: u32, // in seconds
    pub estimated_damage: u32,
    pub recommended_actions: Vec<String>,
}

// Result types

#[derive(Debug, Serialize, Deserialize)]
pub struct DDoSResult {
    pub attack_id: u64,
    pub success: bool,
    pub network_power: u32,
    pub target_defense: u32,
    pub impact: DDoSImpact,
    pub participating_count: usize,
    pub individual_reputation_change: i32,
    pub detection_risk: f64,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ClanWarResult {
    pub war_id: u64,
    pub declaring_clan_id: u64,
    pub target_clan_id: u64,
    pub war_type: WarType,
    pub duration_hours: u32,
    pub objectives: Vec<WarObjective>,
    pub estimated_rewards: WarRewards,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct WarUpdate {
    pub war_id: u64,
    pub update_type: WarUpdateType,
    pub winner_clan_id: Option<u64>,
    pub final_scores: (u32, u32), // (declaring clan score, target clan score)
    pub rewards_distributed: bool,
}

#[derive(Debug, Serialize, Deserialize)]
pub enum WarUpdateType {
    Completed,
    ObjectiveAchieved,
    Cancelled,
}

// Tests
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_combat_initiation() {
        let mut combat_system = CombatSystem::new();
        
        let result = combat_system.initiate_combat(
            1, 
            2, 
            AttackType::Hack, 
            100
        ).unwrap();
        
        assert_eq!(result.attacker_id, 1);
        assert_eq!(result.defender_id, 2);
        assert_eq!(result.attack_type, AttackType::Hack);
    }

    #[test]
    fn test_defense_matrix_creation() {
        let mut combat_system = CombatSystem::new();
        
        let firewall_config = FirewallConfig {
            firewall_type: FirewallType::Advanced,
            power: 100,
            rules: Vec::new(),
            update_frequency: Duration::hours(24),
        };
        
        let ids_config = IntrusionDetectionConfig {
            sensitivity: DetectionSensitivity::High,
            monitoring_scope: vec![MonitoringScope::NetworkTraffic],
            alert_threshold: 3,
            response_delay: Duration::minutes(5),
        };
        
        let matrix = combat_system.configure_defense_matrix(
            1, 
            firewall_config, 
            ids_config, 
            Vec::new()
        ).unwrap();
        
        assert_eq!(matrix.user_id, 1);
        assert!(matrix.effectiveness_rating > 0.0);
    }

    #[test]
    fn test_firewall_analysis() {
        let combat_system = CombatSystem::new();
        
        let analysis = combat_system.calculate_firewall_effectiveness(
            100,
            FirewallType::Advanced,
            120,
            CrackerType::Professional,
            AttackMethod::BruteForce
        );
        
        assert!(analysis.penetration_probability >= 0.0 && analysis.penetration_probability <= 1.0);
        assert!(analysis.detection_probability >= 0.0 && analysis.detection_probability <= 1.0);
        assert!(analysis.estimated_breach_time > 0);
    }

    #[test]
    fn test_clan_war_declaration() {
        let mut combat_system = CombatSystem::new();
        
        let result = combat_system.declare_clan_war(
            1, 
            2, 
            WarType::Domination, 
            24
        ).unwrap();
        
        assert_eq!(result.declaring_clan_id, 1);
        assert_eq!(result.target_clan_id, 2);
        assert_eq!(result.duration_hours, 24);
    }
}