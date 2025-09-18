//! Integration tests for he-core crate

use he_core::*;

#[test]
fn test_core_initialization() {
    // Test that the library can be imported and constants are accessible
    assert_eq!(VERSION, "0.8");
    assert_eq!(VERSION_STATUS, " BETA");
    assert_eq!(GAME_TITLE, "Hacker Experience 0.8 BETA");
}

#[test]
fn test_process_actions_completeness() {
    // Test that all expected process actions are available
    let expected_actions = vec![
        (1, ProcessAction::Download),
        (2, ProcessAction::Upload),
        (3, ProcessAction::Delete),
        (4, ProcessAction::Hide),
        (5, ProcessAction::Seek),
        (7, ProcessAction::Av),
        (8, ProcessAction::ELog),
        (10, ProcessAction::Format),
        (11, ProcessAction::Hack),
        (12, ProcessAction::BankHack),
        (13, ProcessAction::Install),
        (14, ProcessAction::Uninstall),
        (15, ProcessAction::PortScan),
        (16, ProcessAction::HackXp),
        (17, ProcessAction::Research),
        (27, ProcessAction::Ddos),
    ];

    for (value, expected_action) in expected_actions {
        let action = ProcessAction::from_i32(value);
        assert_eq!(action, Some(expected_action), "Failed for value {}", value);
    }
}

#[test]
fn test_software_types_completeness() {
    // Test that all software types are correctly mapped
    let expected_types = vec![
        (1, SoftwareType::Cracker),
        (2, SoftwareType::Hasher),
        (3, SoftwareType::PortScan),
        (4, SoftwareType::Firewall),
        (5, SoftwareType::Hidder),
        (6, SoftwareType::Seeker),
    ];

    for (value, expected_type) in expected_types {
        let software_type = SoftwareType::from_i32(value);
        assert_eq!(software_type, Some(expected_type), "Failed for value {}", value);
    }
}

#[test]
fn test_process_timing_constraints() {
    let timing = ProcessTiming::default();

    // Test that min times are less than max times for all actions
    assert!(timing.hack_min < timing.hack_max);
    assert!(timing.download_min < timing.download_max);
    assert!(timing.upload_min < timing.upload_max);
    assert!(timing.delete_min < timing.delete_max);
    assert!(timing.hide_min < timing.hide_max);
    assert!(timing.seek_min < timing.seek_max);
    assert!(timing.install_min < timing.install_max);
    assert!(timing.av_min < timing.av_max);
    assert!(timing.log_min < timing.log_max);
    assert!(timing.format_min < timing.format_max);

    // Test that all times are positive
    assert!(timing.hack_min > 0);
    assert!(timing.download_min > 0);
    assert!(timing.upload_min > 0);
    assert!(timing.delete_min > 0);
}

#[test]
fn test_type_conversions() {
    // Test bidirectional conversions for ProcessAction
    for i in 1..30 {
        if let Some(action) = ProcessAction::from_i32(i) {
            assert_eq!(action.as_i32(), i);
        }
    }

    // Test bidirectional conversions for SoftwareType
    for i in 1..30 {
        if let Some(software_type) = SoftwareType::from_i32(i) {
            assert_eq!(software_type.as_i32(), i);
        }
    }
}

#[test]
fn test_json_serialization() {
    use serde_json;

    // Test ProcessAction serialization
    let action = ProcessAction::Hack;
    let json = serde_json::to_string(&action).expect("Failed to serialize ProcessAction");
    let deserialized: ProcessAction = serde_json::from_str(&json).expect("Failed to deserialize ProcessAction");
    assert_eq!(action, deserialized);

    // Test ProcessTiming serialization
    let timing = ProcessTiming::default();
    let json = serde_json::to_string(&timing).expect("Failed to serialize ProcessTiming");
    let deserialized: ProcessTiming = serde_json::from_str(&json).expect("Failed to deserialize ProcessTiming");
    assert_eq!(timing.hack_min, deserialized.hack_min);
    assert_eq!(timing.hack_max, deserialized.hack_max);
}

#[test]
fn test_software_extensions() {
    // Test software extension mappings
    assert_eq!(SoftwareExtension::from_type(1), Some(SoftwareExtension::Crc));
    assert_eq!(SoftwareExtension::from_type(4), Some(SoftwareExtension::Fwl));
    assert_eq!(SoftwareExtension::from_type(5), Some(SoftwareExtension::Hdr));

    // Test extension string representations
    assert_eq!(SoftwareExtension::Crc.as_str(), "crc");
    assert_eq!(SoftwareExtension::Fwl.as_str(), "fwl");
    assert_eq!(SoftwareExtension::Hdr.as_str(), "hdr");
}

#[test]
fn test_deprecated_actions() {
    // Ensure deprecated actions are still available for backward compatibility
    assert_eq!(ProcessAction::from_i32(6), Some(ProcessAction::Collect));
    assert_eq!(ProcessAction::from_i32(9), Some(ProcessAction::DLog));

    // Verify they convert back correctly
    assert_eq!(ProcessAction::Collect.as_i32(), 6);
    assert_eq!(ProcessAction::DLog.as_i32(), 9);
}

#[test]
fn test_id_types() {
    // Test that ID types work correctly
    let user_id: UserId = 42;
    let process_id: ProcessId = 100;
    let software_id: SoftwareId = 200;
    let hardware_id: HardwareId = 300;
    let clan_id: ClanId = 400;
    let ip: IpAddress = "192.168.1.1".to_string();

    assert_eq!(user_id, 42);
    assert_eq!(process_id, 100);
    assert_eq!(software_id, 200);
    assert_eq!(hardware_id, 300);
    assert_eq!(clan_id, 400);
    assert_eq!(ip, "192.168.1.1");
}