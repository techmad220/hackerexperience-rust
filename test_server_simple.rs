// Simple test server for HackerExperience
// Compile: rustc test_server_simple.rs -o test_server
// Run: ./test_server

use std::io::prelude::*;
use std::net::{TcpListener, TcpStream};
use std::thread;

fn handle_client(mut stream: TcpStream) {
    let mut buffer = [0; 1024];
    stream.read(&mut buffer).unwrap();

    let request = String::from_utf8_lossy(&buffer[..]);

    // Parse the request path
    let path = request.lines().next()
        .and_then(|line| line.split_whitespace().nth(1))
        .unwrap_or("/");

    println!("Request: {}", path);

    // Generate response based on path
    let (status, content) = match path {
        "/health" => ("200 OK", r#"{"status":"healthy"}"#),

        "/api/hacking/scan" => ("200 OK", r#"{
            "success": true,
            "server_info": {
                "ip_address": "1.2.3.4",
                "hostname": "whois.first.org",
                "owner": "First International Bank",
                "server_type": "Web Server",
                "security_level": 50,
                "firewall_level": 30,
                "is_online": true
            },
            "message": "Scan complete"
        }"#),

        "/api/hacking/internet" => ("200 OK", r#"{
            "your_ip": "player_1.vpn",
            "known_servers": [
                {
                    "ip": "1.2.3.4",
                    "hostname": "whois.first.org",
                    "last_seen": "Recently",
                    "notes": "Tutorial server"
                }
            ],
            "recent_hacks": ["10.0.0.1 - Home PC"],
            "bounties": [
                {
                    "corporation": "MegaCorp",
                    "target_ip": "192.168.100.50",
                    "reward": 100000,
                    "difficulty": "Tier 3"
                }
            ]
        }"#),

        "/api/process/list" => ("200 OK", r#"{
            "processes": [
                {
                    "pid": 1,
                    "user_id": 1,
                    "process_type": "crack",
                    "state": "running",
                    "started_at": "2025-09-17T09:00:00Z",
                    "completed_at": null,
                    "source_server": "localhost",
                    "target_server": "192.168.1.100",
                    "priority": 2,
                    "cpu_usage": 350,
                    "ram_usage": 128,
                    "completion_percentage": 75.0,
                    "estimated_time_remaining": 15
                }
            ]
        }"#),

        "/api/software/list" => ("200 OK", r#"{
            "local_software": [
                {
                    "id": 1,
                    "name": "Basic Cracker",
                    "version": "1.0",
                    "software_type": "cracker",
                    "size": 100,
                    "cpu_usage": 200,
                    "ram_usage": 64,
                    "effectiveness": 50,
                    "is_hidden": false,
                    "is_installed": true,
                    "location": "local"
                }
            ],
            "external_software": [],
            "local_used": 500,
            "local_total": 100000,
            "external_used": 0,
            "external_total": 200000
        }"#),

        "/api/missions" => ("200 OK", r#"{
            "success": true,
            "missions": [
                {
                    "id": 1,
                    "mission_type": "tutorial",
                    "status": "active",
                    "reward_money": 1000,
                    "reward_xp": 50,
                    "progress": 2,
                    "total_steps": 3
                }
            ]
        }"#),

        "/api/missions/available" => ("200 OK", r#"[
            {
                "id": "1",
                "name": "First Score",
                "description": "Your first real hacking job",
                "mission_type": "hack",
                "difficulty": "Easy",
                "reward_money": 5000,
                "reward_xp": 100,
                "reward_items": [],
                "requirements": ["Level 1"],
                "objectives": ["Hack the target server", "Download files", "Delete logs"]
            }
        ]"#),

        _ => ("404 NOT FOUND", r#"{"error":"Not Found"}"#),
    };

    // Send response
    let response = format!(
        "HTTP/1.1 {}\r\n\
         Content-Type: application/json\r\n\
         Access-Control-Allow-Origin: *\r\n\
         Access-Control-Allow-Methods: GET, POST, PUT, DELETE, OPTIONS\r\n\
         Access-Control-Allow-Headers: Content-Type, Authorization\r\n\
         Content-Length: {}\r\n\
         \r\n\
         {}",
        status,
        content.len(),
        content
    );

    stream.write(response.as_bytes()).unwrap();
    stream.flush().unwrap();
}

fn main() {
    println!("🚀 Starting HackerExperience Test Server");
    println!("📡 API Server: http://localhost:3000");
    println!("");
    println!("Available endpoints:");
    println!("  GET  /health");
    println!("  POST /api/hacking/scan");
    println!("  GET  /api/hacking/internet");
    println!("  GET  /api/process/list");
    println!("  GET  /api/software/list");
    println!("  GET  /api/missions");
    println!("  GET  /api/missions/available");
    println!("");

    let listener = TcpListener::bind("0.0.0.0:3000").unwrap();
    println!("Server listening on port 3000...");

    for stream in listener.incoming() {
        match stream {
            Ok(stream) => {
                thread::spawn(|| {
                    handle_client(stream);
                });
            }
            Err(e) => {
                eprintln!("Connection failed: {}", e);
            }
        }
    }
}