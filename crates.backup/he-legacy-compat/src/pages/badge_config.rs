use std::collections::HashMap;
use serde::{Deserialize, Serialize};

/// Badge configuration structure - matches original PHP badge arrays
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Badge {
    pub name: String,
    pub desc: String,
}

/// User badge configuration - 1:1 port of $badgeArrayUser from badge_config.php
/// 
/// Original: Static array defining all available user badges with their names and descriptions
/// This configuration is used throughout the application to display badge information
pub fn get_user_badges() -> HashMap<u32, Badge> {
    let mut badges = HashMap::new();
    
    // Badge ID 1: Developer
    badges.insert(1, Badge {
        name: "Developer".to_string(),
        desc: "".to_string(),
    });
    
    // Badge ID 2: Administrator  
    badges.insert(2, Badge {
        name: "Administrator".to_string(),
        desc: "".to_string(),
    });
    
    // Badge ID 3: Moderator
    badges.insert(3, Badge {
        name: "Moderator".to_string(),
        desc: "".to_string(),
    });
    
    // Badge ID 4: Translator
    badges.insert(4, Badge {
        name: "Translator".to_string(),
        desc: "".to_string(),
    });
    
    // Badge ID 5: Premium
    badges.insert(5, Badge {
        name: "Premium".to_string(),
        desc: "".to_string(),
    });
    
    // Badge ID 6: Beta tester
    badges.insert(6, Badge {
        name: "Beta tester".to_string(),
        desc: "".to_string(),
    });
    
    // Badge ID 7: Best player - First ranked at the end of the round
    badges.insert(7, Badge {
        name: "Best player".to_string(),
        desc: "First ranked at the end of the round".to_string(),
    });
    
    // Badge ID 8: Runner up - Second ranked at the end of the round
    badges.insert(8, Badge {
        name: "Runner up".to_string(),
        desc: "Second ranked at the end of the round".to_string(),
    });
    
    // Badge ID 9: 3rd placed - 3rd placed at the end of the round
    badges.insert(9, Badge {
        name: "3rd placed".to_string(),
        desc: "3rd placed at the end of the round".to_string(),
    });
    
    // Badge ID 10: Almost there - Ranked among 10 top players of the round
    badges.insert(10, Badge {
        name: "Almost there".to_string(),
        desc: "Ranked among 10 top players of the round".to_string(),
    });
    
    // Badge ID 11: Doom! - Launched a doom attack
    badges.insert(11, Badge {
        name: "Doom!".to_string(),
        desc: "Launched a doom attack".to_string(),
    });
    
    // Badge ID 12: ANTI-Doom! - Disabled a doom attack
    badges.insert(12, Badge {
        name: "ANTI-Doom!".to_string(),
        desc: "Disabled a doom attack".to_string(),
    });
    
    // Badge ID 13: DDoS! - Launched a DDoS attack
    badges.insert(13, Badge {
        name: "DDoS!".to_string(),
        desc: "Launched a DDoS attack".to_string(),
    });
    
    badges
}

/// Clan badge configuration - 1:1 port of $badgeArrayClan from badge_config.php
/// 
/// Original: Static array defining all available clan badges with their names and descriptions
/// This configuration is used for clan-related badge displays
pub fn get_clan_badges() -> HashMap<u32, Badge> {
    let mut badges = HashMap::new();
    
    // Badge ID 1: Best clan - Winner
    badges.insert(1, Badge {
        name: "Best clan".to_string(),
        desc: "Winner".to_string(),
    });
    
    // Badge ID 2: Runner up - Almost winner
    badges.insert(2, Badge {
        name: "Runner up".to_string(),
        desc: "Almost winner".to_string(),
    });
    
    // Badge ID 3: 3rd placed - Close..
    badges.insert(3, Badge {
        name: "3rd placed".to_string(),
        desc: "Close..".to_string(),
    });
    
    badges
}

/// Get user badge by ID - utility function for badge lookups
pub fn get_user_badge(id: u32) -> Option<Badge> {
    get_user_badges().get(&id).cloned()
}

/// Get clan badge by ID - utility function for badge lookups  
pub fn get_clan_badge(id: u32) -> Option<Badge> {
    get_clan_badges().get(&id).cloned()
}

/// Get all user badge IDs - utility function for iteration
pub fn get_user_badge_ids() -> Vec<u32> {
    get_user_badges().keys().cloned().collect()
}

/// Get all clan badge IDs - utility function for iteration
pub fn get_clan_badge_ids() -> Vec<u32> {
    get_clan_badges().keys().cloned().collect()
}

/// Badge type enumeration for type safety
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum BadgeType {
    User,
    Clan,
}

/// Unified badge lookup function that works with both user and clan badges
pub fn get_badge(badge_type: BadgeType, id: u32) -> Option<Badge> {
    match badge_type {
        BadgeType::User => get_user_badge(id),
        BadgeType::Clan => get_clan_badge(id),
    }
}

/// Badge validation - check if a badge ID exists for the given type
pub fn is_valid_badge(badge_type: BadgeType, id: u32) -> bool {
    match badge_type {
        BadgeType::User => get_user_badges().contains_key(&id),
        BadgeType::Clan => get_clan_badges().contains_key(&id),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_user_badges_complete() {
        let user_badges = get_user_badges();
        
        // Verify all expected badge IDs are present (1-13)
        for i in 1..=13 {
            assert!(user_badges.contains_key(&i), "User badge {} should exist", i);
        }
        
        // Test specific badge content
        let developer = user_badges.get(&1).unwrap();
        assert_eq!(developer.name, "Developer");
        assert_eq!(developer.desc, "");
        
        let best_player = user_badges.get(&7).unwrap();
        assert_eq!(best_player.name, "Best player");
        assert_eq!(best_player.desc, "First ranked at the end of the round");
        
        let doom = user_badges.get(&11).unwrap();
        assert_eq!(doom.name, "Doom!");
        assert_eq!(doom.desc, "Launched a doom attack");
    }
    
    #[test]
    fn test_clan_badges_complete() {
        let clan_badges = get_clan_badges();
        
        // Verify all expected badge IDs are present (1-3)
        for i in 1..=3 {
            assert!(clan_badges.contains_key(&i), "Clan badge {} should exist", i);
        }
        
        // Test specific badge content
        let best_clan = clan_badges.get(&1).unwrap();
        assert_eq!(best_clan.name, "Best clan");
        assert_eq!(best_clan.desc, "Winner");
        
        let runner_up = clan_badges.get(&2).unwrap();
        assert_eq!(runner_up.name, "Runner up");
        assert_eq!(runner_up.desc, "Almost winner");
        
        let third_place = clan_badges.get(&3).unwrap();
        assert_eq!(third_place.name, "3rd placed");
        assert_eq!(third_place.desc, "Close..");
    }
    
    #[test]
    fn test_badge_lookup_functions() {
        // Test user badge lookup
        let developer = get_user_badge(1);
        assert!(developer.is_some());
        assert_eq!(developer.unwrap().name, "Developer");
        
        let invalid_user = get_user_badge(999);
        assert!(invalid_user.is_none());
        
        // Test clan badge lookup
        let best_clan = get_clan_badge(1);
        assert!(best_clan.is_some());
        assert_eq!(best_clan.unwrap().name, "Best clan");
        
        let invalid_clan = get_clan_badge(999);
        assert!(invalid_clan.is_none());
    }
    
    #[test]
    fn test_unified_badge_lookup() {
        // Test unified lookup function
        let user_badge = get_badge(BadgeType::User, 1);
        assert!(user_badge.is_some());
        assert_eq!(user_badge.unwrap().name, "Developer");
        
        let clan_badge = get_badge(BadgeType::Clan, 1);
        assert!(clan_badge.is_some());
        assert_eq!(clan_badge.unwrap().name, "Best clan");
    }
    
    #[test]
    fn test_badge_validation() {
        // Test user badge validation
        assert!(is_valid_badge(BadgeType::User, 1));
        assert!(is_valid_badge(BadgeType::User, 13));
        assert!(!is_valid_badge(BadgeType::User, 14));
        assert!(!is_valid_badge(BadgeType::User, 0));
        
        // Test clan badge validation
        assert!(is_valid_badge(BadgeType::Clan, 1));
        assert!(is_valid_badge(BadgeType::Clan, 3));
        assert!(!is_valid_badge(BadgeType::Clan, 4));
        assert!(!is_valid_badge(BadgeType::Clan, 0));
    }
    
    #[test]
    fn test_badge_id_collections() {
        let user_ids = get_user_badge_ids();
        assert_eq!(user_ids.len(), 13);
        assert!(user_ids.contains(&1));
        assert!(user_ids.contains(&13));
        
        let clan_ids = get_clan_badge_ids();
        assert_eq!(clan_ids.len(), 3);
        assert!(clan_ids.contains(&1));
        assert!(clan_ids.contains(&3));
    }
}