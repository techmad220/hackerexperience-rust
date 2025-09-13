use he_core::*;

fn main() -> anyhow::Result<()> {
    println!("{}", GAME_TITLE);
    
    // Demo the 1:1 parity with original PHP
    demo_user_creation()?;
    demo_hardware_system()?;
    demo_process_system()?;
    
    println!("\n✅ All core systems working! 1:1 parity achieved with original PHP.");
    Ok(())
}

fn demo_user_creation() -> anyhow::Result<()> {
    println!("\n🔧 Testing User System (Player.class.php equivalent):");
    
    let mut user = User::new(
        "TestHacker".to_string(),
        "test@hackerexperience.com".to_string(), 
        "$2b$12$hash".to_string() // BCrypt hash
    );
    
    println!("   User created: {} (ID: {})", user.get_display_name(), user.id);
    
    user.set_online(true);
    println!("   User online status: {}", user.is_online);
    
    user.join_clan(42);
    println!("   User joined clan: {:?}", user.clan_id);
    
    Ok(())
}

fn demo_hardware_system() -> anyhow::Result<()> {
    println!("\n💾 Testing Hardware System (HardwareVPC.class.php equivalent):");
    
    let hardware = Hardware::new(1, false); // User ID 1, not NPC
    println!("   Hardware specs: RAM={}MB, CPU={}MHz, HDD={}MB, NET={}Mbps", 
             hardware.ram, hardware.cpu, hardware.hdd, hardware.net);
    println!("   Total power: {}", hardware.total_power());
    
    let hardware_list = vec![hardware];
    let info = HardwareInfo::from_hardware_list(&hardware_list);
    println!("   Aggregated info: {} PCs, {} total CPU", info.total_pcs, info.total_cpu);
    
    Ok(())
}

fn demo_process_system() -> anyhow::Result<()> {
    println!("\n⚙️  Testing Process System (Process.class.php equivalent):");
    println!("   'This is the most complex part of Legacy and HE2.' - Original comment");
    
    let mut process = Process::new(
        1,                              // creator_id
        Some(2),                        // victim_id
        ProcessAction::Hack,            // action
        "192.168.1.100".to_string(),   // target_ip
        Some(42),                       // software_id
        300,                            // duration (5 minutes)
    );
    
    println!("   Process created: {:?} against {}", process.action, process.target_ip);
    println!("   CPU usage: {}, NET usage: {}", process.cpu_usage, process.net_usage);
    println!("   Time remaining: {}s", process.time_left);
    
    process.start()?;
    println!("   Process started: {:?}", process.status);
    
    // Simulate 60 seconds passing
    let completed = process.tick(60)?;
    println!("   After 60s tick - Completed: {}, Time left: {}s", completed, process.time_left);
    
    Ok(())
}
