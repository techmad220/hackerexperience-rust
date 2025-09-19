use axum::{
    extract::Query,
    http::StatusCode,
    response::Html,
    Extension,
};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use he_core::session::PhpSession;
use sqlx::PgPool;

/// Certificate data structure - 1:1 port of static $certs array from certs.php
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Certificate {
    pub name: String,
    pub desc: String,
    pub price: String,
}

/// Certificate storage - static equivalent of PHP $certs array
pub fn get_certificates() -> HashMap<String, Certificate> {
    let mut certs = HashMap::new();
    
    // Certificate 1 - Basic tutorial
    certs.insert("1".to_string(), Certificate {
        name: "Basic tutorial".to_string(),
        desc: "Follow this tutorial to learn how to play".to_string(),
        price: "0".to_string(),
    });
    
    // Certificate 2 - Hacking 101
    certs.insert("2".to_string(), Certificate {
        name: "Hacking 101".to_string(),
        desc: "Learn the basics of hacking".to_string(),
        price: "0".to_string(),
    });
    
    // Certificate 3 - Intermediate hacking
    certs.insert("3".to_string(), Certificate {
        name: "Intermediate hacking".to_string(),
        desc: "Improve your hacking techniques".to_string(),
        price: "50".to_string(),
    });
    
    // Certificate 4 - Advanced hacking
    certs.insert("4".to_string(), Certificate {
        name: "Advanced hacking".to_string(),
        desc: "Meet the wonders of DDoS world".to_string(),
        price: "200".to_string(),
    });
    
    // Certificate 5 - DDoS security
    certs.insert("5".to_string(), Certificate {
        name: "DDoS security".to_string(),
        desc: "Learn to protect yourself from DDoS".to_string(),
        price: "500".to_string(),
    });
    
    certs
}

/// Query parameters for certs page access
#[derive(Debug, Deserialize)]
pub struct CertsQuery {
    pub id: Option<String>,
    pub action: Option<String>,
}

/// Certs page handler - 1:1 port of certs.php
/// 
/// Original: Certificate definitions and data structure
/// Features:
/// - Provides static certificate data (5 certificates)
/// - Each certificate has NAME, DESC, and PRICE fields
/// - Used by other parts of the game for certificate purchases and display
/// - Contains tutorial, hacking courses, and security training certificates
/// 
/// Note: Original was just a data file, but this implementation provides
/// both data access and potential web endpoint functionality
pub async fn certs_handler(
    Extension(db): Extension<PgPool>,
    Extension(session): Extension<PhpSession>,
    Query(params): Query<CertsQuery>,
) -> Result<Html<String>, StatusCode> {
    
    // Get certificates data
    let certificates = get_certificates();
    
    // Check if specific certificate is requested
    if let Some(cert_id) = &params.id {
        if let Some(cert) = certificates.get(cert_id) {
            // Display specific certificate
            let html = format!(
                r#"
                <html>
                <head>
                    <title>Certificate: {} - Hacker Experience</title>
                </head>
                <body>
                    <h2>Certificate Details</h2>
                    <div class="certificate">
                        <h3>{}</h3>
                        <p><strong>Description:</strong> {}</p>
                        <p><strong>Price:</strong> ${}</p>
                        <br/>
                        <a href="certs.php">← Back to certificates</a>
                    </div>
                </body>
                </html>
                "#,
                cert.name, cert.name, cert.desc, cert.price
            );
            return Ok(Html(html));
        } else {
            // Certificate not found
            let html = r#"
                <html>
                <head>
                    <title>Certificate Not Found - Hacker Experience</title>
                </head>
                <body>
                    <h2>Certificate Not Found</h2>
                    <p>The requested certificate does not exist.</p>
                    <a href="certs.php">← Back to certificates</a>
                </body>
                </html>
            "#;
            return Ok(Html(html.to_string()));
        }
    }
    
    // Display all certificates (default view)
    let mut cert_list = String::new();
    for (id, cert) in &certificates {
        cert_list.push_str(&format!(
            r#"
            <tr>
                <td>{}</td>
                <td><a href="?id={}">{}</a></td>
                <td>{}</td>
                <td>${}</td>
            </tr>
            "#,
            id, id, cert.name, cert.desc, cert.price
        ));
    }
    
    let html = format!(
        r#"
        <html>
        <head>
            <title>Certificates - Hacker Experience</title>
            <style>
                table {{
                    border-collapse: collapse;
                    width: 100%;
                }}
                th, td {{
                    border: 1px solid #ddd;
                    padding: 8px;
                    text-align: left;
                }}
                th {{
                    background-color: #f2f2f2;
                }}
            </style>
        </head>
        <body>
            <h2>Available Certificates</h2>
            <table>
                <thead>
                    <tr>
                        <th>ID</th>
                        <th>Name</th>
                        <th>Description</th>
                        <th>Price</th>
                    </tr>
                </thead>
                <tbody>
                    {}
                </tbody>
            </table>
        </body>
        </html>
        "#,
        cert_list
    );
    
    Ok(Html(html))
}

/// Get certificate by ID - utility function for other modules
pub fn get_certificate_by_id(id: &str) -> Option<Certificate> {
    let certificates = get_certificates();
    certificates.get(id).cloned()
}

/// Get all certificate IDs - utility function
pub fn get_certificate_ids() -> Vec<String> {
    let certificates = get_certificates();
    certificates.keys().cloned().collect()
}

/// Check if certificate exists - utility function
pub fn certificate_exists(id: &str) -> bool {
    let certificates = get_certificates();
    certificates.contains_key(id)
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_certificates_data_integrity() {
        let certs = get_certificates();
        
        // Test that all expected certificates exist
        assert_eq!(certs.len(), 5);
        assert!(certs.contains_key("1"));
        assert!(certs.contains_key("2"));
        assert!(certs.contains_key("3"));
        assert!(certs.contains_key("4"));
        assert!(certs.contains_key("5"));
        
        // Test specific certificate data matches original PHP
        let basic_tutorial = certs.get("1").unwrap();
        assert_eq!(basic_tutorial.name, "Basic tutorial");
        assert_eq!(basic_tutorial.desc, "Follow this tutorial to learn how to play");
        assert_eq!(basic_tutorial.price, "0");
        
        let hacking_101 = certs.get("2").unwrap();
        assert_eq!(hacking_101.name, "Hacking 101");
        assert_eq!(hacking_101.desc, "Learn the basics of hacking");
        assert_eq!(hacking_101.price, "0");
        
        let intermediate = certs.get("3").unwrap();
        assert_eq!(intermediate.name, "Intermediate hacking");
        assert_eq!(intermediate.desc, "Improve your hacking techniques");
        assert_eq!(intermediate.price, "50");
        
        let advanced = certs.get("4").unwrap();
        assert_eq!(advanced.name, "Advanced hacking");
        assert_eq!(advanced.desc, "Meet the wonders of DDoS world");
        assert_eq!(advanced.price, "200");
        
        let ddos_security = certs.get("5").unwrap();
        assert_eq!(ddos_security.name, "DDoS security");
        assert_eq!(ddos_security.desc, "Learn to protect yourself from DDoS");
        assert_eq!(ddos_security.price, "500");
    }
    
    #[test]
    fn test_utility_functions() {
        // Test get_certificate_by_id
        let cert = get_certificate_by_id("1").unwrap();
        assert_eq!(cert.name, "Basic tutorial");
        
        let non_existent = get_certificate_by_id("999");
        assert!(non_existent.is_none());
        
        // Test certificate_exists
        assert!(certificate_exists("1"));
        assert!(certificate_exists("5"));
        assert!(!certificate_exists("999"));
        assert!(!certificate_exists("0"));
        
        // Test get_certificate_ids
        let ids = get_certificate_ids();
        assert_eq!(ids.len(), 5);
        assert!(ids.contains(&"1".to_string()));
        assert!(ids.contains(&"5".to_string()));
    }
    
    #[test]
    fn test_certificate_prices() {
        let certs = get_certificates();
        
        // Free certificates
        assert_eq!(certs.get("1").unwrap().price, "0");
        assert_eq!(certs.get("2").unwrap().price, "0");
        
        // Paid certificates  
        assert_eq!(certs.get("3").unwrap().price, "50");
        assert_eq!(certs.get("4").unwrap().price, "200");
        assert_eq!(certs.get("5").unwrap().price, "500");
    }
}