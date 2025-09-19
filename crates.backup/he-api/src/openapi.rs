//! OpenAPI/Swagger Documentation Generation
//!
//! Automatically generates OpenAPI 3.0 documentation for all API endpoints

use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// OpenAPI 3.0 Specification
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct OpenApiSpec {
    pub openapi: String,
    pub info: ApiInfo,
    pub servers: Vec<ApiServer>,
    pub paths: HashMap<String, PathItem>,
    pub components: Components,
    pub security: Vec<SecurityRequirement>,
    pub tags: Vec<Tag>,
}

impl Default for OpenApiSpec {
    fn default() -> Self {
        Self {
            openapi: "3.0.3".to_string(),
            info: ApiInfo::default(),
            servers: vec![
                ApiServer {
                    url: "https://api.hackerexperience.com".to_string(),
                    description: Some("Production server".to_string()),
                },
                ApiServer {
                    url: "https://staging-api.hackerexperience.com".to_string(),
                    description: Some("Staging server".to_string()),
                },
                ApiServer {
                    url: "http://localhost:8080".to_string(),
                    description: Some("Local development server".to_string()),
                },
            ],
            paths: HashMap::new(),
            components: Components::default(),
            security: vec![
                SecurityRequirement {
                    bearer_auth: vec![],
                },
            ],
            tags: vec![
                Tag {
                    name: "Authentication".to_string(),
                    description: Some("User authentication and authorization".to_string()),
                },
                Tag {
                    name: "Users".to_string(),
                    description: Some("User management operations".to_string()),
                },
                Tag {
                    name: "Hardware".to_string(),
                    description: Some("Hardware management and upgrades".to_string()),
                },
                Tag {
                    name: "Software".to_string(),
                    description: Some("Software installation and management".to_string()),
                },
                Tag {
                    name: "Processes".to_string(),
                    description: Some("Process management and monitoring".to_string()),
                },
                Tag {
                    name: "Hacking".to_string(),
                    description: Some("Hacking operations and tools".to_string()),
                },
                Tag {
                    name: "Banking".to_string(),
                    description: Some("Bank accounts and transactions".to_string()),
                },
                Tag {
                    name: "Logs".to_string(),
                    description: Some("System and activity logs".to_string()),
                },
            ],
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ApiInfo {
    pub title: String,
    pub description: String,
    pub version: String,
    pub contact: Contact,
    pub license: License,
}

impl Default for ApiInfo {
    fn default() -> Self {
        Self {
            title: "HackerExperience API".to_string(),
            description: "RESTful API for HackerExperience game backend".to_string(),
            version: "1.0.0".to_string(),
            contact: Contact {
                name: Some("HackerExperience Team".to_string()),
                email: Some("api@hackerexperience.com".to_string()),
                url: Some("https://hackerexperience.com".to_string()),
            },
            license: License {
                name: "MIT".to_string(),
                url: Some("https://opensource.org/licenses/MIT".to_string()),
            },
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Contact {
    pub name: Option<String>,
    pub email: Option<String>,
    pub url: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct License {
    pub name: String,
    pub url: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ApiServer {
    pub url: String,
    pub description: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Tag {
    pub name: String,
    pub description: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PathItem {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub get: Option<Operation>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub post: Option<Operation>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub put: Option<Operation>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub delete: Option<Operation>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub patch: Option<Operation>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Operation {
    pub summary: String,
    pub description: Option<String>,
    pub operation_id: String,
    pub tags: Vec<String>,
    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub parameters: Vec<Parameter>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub request_body: Option<RequestBody>,
    pub responses: HashMap<String, Response>,
    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub security: Vec<SecurityRequirement>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Parameter {
    pub name: String,
    #[serde(rename = "in")]
    pub location: String,
    pub description: Option<String>,
    pub required: bool,
    pub schema: Schema,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct RequestBody {
    pub description: Option<String>,
    pub required: bool,
    pub content: HashMap<String, MediaType>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MediaType {
    pub schema: Schema,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub example: Option<serde_json::Value>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Response {
    pub description: String,
    #[serde(skip_serializing_if = "HashMap::is_empty")]
    pub content: HashMap<String, MediaType>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Schema {
    #[serde(rename = "type", skip_serializing_if = "Option::is_none")]
    pub schema_type: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub format: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub default: Option<serde_json::Value>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub example: Option<serde_json::Value>,
    #[serde(rename = "$ref", skip_serializing_if = "Option::is_none")]
    pub reference: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub items: Option<Box<Schema>>,
    #[serde(skip_serializing_if = "HashMap::is_empty")]
    pub properties: HashMap<String, Schema>,
    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub required: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Components {
    pub schemas: HashMap<String, Schema>,
    pub security_schemes: HashMap<String, SecurityScheme>,
}

impl Default for Components {
    fn default() -> Self {
        let mut schemas = HashMap::new();
        let mut security_schemes = HashMap::new();

        // Define common schemas
        schemas.insert("Error".to_string(), Schema {
            schema_type: Some("object".to_string()),
            properties: {
                let mut props = HashMap::new();
                props.insert("error".to_string(), Schema {
                    schema_type: Some("string".to_string()),
                    description: Some("Error message".to_string()),
                    ..Default::default()
                });
                props.insert("code".to_string(), Schema {
                    schema_type: Some("string".to_string()),
                    description: Some("Error code".to_string()),
                    ..Default::default()
                });
                props
            },
            required: vec!["error".to_string()],
            ..Default::default()
        });

        schemas.insert("User".to_string(), Schema {
            schema_type: Some("object".to_string()),
            properties: {
                let mut props = HashMap::new();
                props.insert("id".to_string(), Schema {
                    schema_type: Some("integer".to_string()),
                    format: Some("int64".to_string()),
                    ..Default::default()
                });
                props.insert("login".to_string(), Schema {
                    schema_type: Some("string".to_string()),
                    ..Default::default()
                });
                props.insert("email".to_string(), Schema {
                    schema_type: Some("string".to_string()),
                    format: Some("email".to_string()),
                    ..Default::default()
                });
                props.insert("created".to_string(), Schema {
                    schema_type: Some("string".to_string()),
                    format: Some("date-time".to_string()),
                    ..Default::default()
                });
                props
            },
            required: vec!["id".to_string(), "login".to_string(), "email".to_string()],
            ..Default::default()
        });

        // Define security schemes
        security_schemes.insert("bearerAuth".to_string(), SecurityScheme {
            scheme_type: "http".to_string(),
            scheme: Some("bearer".to_string()),
            bearer_format: Some("JWT".to_string()),
            description: Some("JWT Bearer token authentication".to_string()),
        });

        Self {
            schemas,
            security_schemes,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SecurityScheme {
    #[serde(rename = "type")]
    pub scheme_type: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub scheme: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub bearer_format: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SecurityRequirement {
    #[serde(rename = "bearerAuth")]
    pub bearer_auth: Vec<String>,
}

impl Schema {
    pub fn string() -> Self {
        Self {
            schema_type: Some("string".to_string()),
            ..Default::default()
        }
    }

    pub fn integer() -> Self {
        Self {
            schema_type: Some("integer".to_string()),
            ..Default::default()
        }
    }

    pub fn boolean() -> Self {
        Self {
            schema_type: Some("boolean".to_string()),
            ..Default::default()
        }
    }

    pub fn array(items: Schema) -> Self {
        Self {
            schema_type: Some("array".to_string()),
            items: Some(Box::new(items)),
            ..Default::default()
        }
    }

    pub fn reference(ref_path: &str) -> Self {
        Self {
            reference: Some(format!("#/components/schemas/{}", ref_path)),
            ..Default::default()
        }
    }
}

impl Default for Schema {
    fn default() -> Self {
        Self {
            schema_type: None,
            format: None,
            description: None,
            default: None,
            example: None,
            reference: None,
            items: None,
            properties: HashMap::new(),
            required: Vec::new(),
        }
    }
}

/// Generate OpenAPI documentation
pub fn generate_openapi_spec() -> OpenApiSpec {
    let mut spec = OpenApiSpec::default();

    // Add authentication endpoints
    add_auth_endpoints(&mut spec);

    // Add user endpoints
    add_user_endpoints(&mut spec);

    // Add hardware endpoints
    add_hardware_endpoints(&mut spec);

    // Add process endpoints
    add_process_endpoints(&mut spec);

    spec
}

fn add_auth_endpoints(spec: &mut OpenApiSpec) {
    // POST /api/auth/register
    spec.paths.insert("/api/auth/register".to_string(), PathItem {
        post: Some(Operation {
            summary: "Register new user".to_string(),
            description: Some("Create a new user account".to_string()),
            operation_id: "register".to_string(),
            tags: vec!["Authentication".to_string()],
            parameters: vec![],
            request_body: Some(RequestBody {
                description: Some("Registration details".to_string()),
                required: true,
                content: {
                    let mut content = HashMap::new();
                    content.insert("application/json".to_string(), MediaType {
                        schema: Schema {
                            schema_type: Some("object".to_string()),
                            properties: {
                                let mut props = HashMap::new();
                                props.insert("username".to_string(), Schema::string());
                                props.insert("email".to_string(), Schema::string());
                                props.insert("password".to_string(), Schema::string());
                                props
                            },
                            required: vec!["username".to_string(), "email".to_string(), "password".to_string()],
                            ..Default::default()
                        },
                        example: Some(serde_json::json!({
                            "username": "hacker123",
                            "email": "hacker@example.com",
                            "password": "SecurePassword123!"
                        })),
                    });
                    content
                },
            }),
            responses: {
                let mut responses = HashMap::new();
                responses.insert("201".to_string(), Response {
                    description: "User created successfully".to_string(),
                    content: {
                        let mut content = HashMap::new();
                        content.insert("application/json".to_string(), MediaType {
                            schema: Schema::reference("User"),
                            example: None,
                        });
                        content
                    },
                });
                responses.insert("400".to_string(), Response {
                    description: "Invalid input".to_string(),
                    content: {
                        let mut content = HashMap::new();
                        content.insert("application/json".to_string(), MediaType {
                            schema: Schema::reference("Error"),
                            example: None,
                        });
                        content
                    },
                });
                responses
            },
            security: vec![],
        }),
        ..Default::default()
    });

    // POST /api/auth/login
    spec.paths.insert("/api/auth/login".to_string(), PathItem {
        post: Some(Operation {
            summary: "User login".to_string(),
            description: Some("Authenticate user and receive JWT token".to_string()),
            operation_id: "login".to_string(),
            tags: vec!["Authentication".to_string()],
            parameters: vec![],
            request_body: Some(RequestBody {
                description: Some("Login credentials".to_string()),
                required: true,
                content: {
                    let mut content = HashMap::new();
                    content.insert("application/json".to_string(), MediaType {
                        schema: Schema {
                            schema_type: Some("object".to_string()),
                            properties: {
                                let mut props = HashMap::new();
                                props.insert("username".to_string(), Schema::string());
                                props.insert("password".to_string(), Schema::string());
                                props
                            },
                            required: vec!["username".to_string(), "password".to_string()],
                            ..Default::default()
                        },
                        example: Some(serde_json::json!({
                            "username": "hacker123",
                            "password": "SecurePassword123!"
                        })),
                    });
                    content
                },
            }),
            responses: {
                let mut responses = HashMap::new();
                responses.insert("200".to_string(), Response {
                    description: "Login successful".to_string(),
                    content: {
                        let mut content = HashMap::new();
                        content.insert("application/json".to_string(), MediaType {
                            schema: Schema {
                                schema_type: Some("object".to_string()),
                                properties: {
                                    let mut props = HashMap::new();
                                    props.insert("token".to_string(), Schema::string());
                                    props.insert("user".to_string(), Schema::reference("User"));
                                    props
                                },
                                ..Default::default()
                            },
                            example: None,
                        });
                        content
                    },
                });
                responses
            },
            security: vec![],
        }),
        ..Default::default()
    });
}

fn add_user_endpoints(spec: &mut OpenApiSpec) {
    // GET /api/users/{id}
    spec.paths.insert("/api/users/{id}".to_string(), PathItem {
        get: Some(Operation {
            summary: "Get user by ID".to_string(),
            description: Some("Retrieve user information by user ID".to_string()),
            operation_id: "getUserById".to_string(),
            tags: vec!["Users".to_string()],
            parameters: vec![
                Parameter {
                    name: "id".to_string(),
                    location: "path".to_string(),
                    description: Some("User ID".to_string()),
                    required: true,
                    schema: Schema::integer(),
                },
            ],
            request_body: None,
            responses: {
                let mut responses = HashMap::new();
                responses.insert("200".to_string(), Response {
                    description: "User found".to_string(),
                    content: {
                        let mut content = HashMap::new();
                        content.insert("application/json".to_string(), MediaType {
                            schema: Schema::reference("User"),
                            example: None,
                        });
                        content
                    },
                });
                responses.insert("404".to_string(), Response {
                    description: "User not found".to_string(),
                    content: HashMap::new(),
                });
                responses
            },
            security: vec![SecurityRequirement { bearer_auth: vec![] }],
        }),
        ..Default::default()
    });
}

fn add_hardware_endpoints(spec: &mut OpenApiSpec) {
    // GET /api/hardware
    spec.paths.insert("/api/hardware".to_string(), PathItem {
        get: Some(Operation {
            summary: "Get user hardware".to_string(),
            description: Some("Retrieve current user's hardware configuration".to_string()),
            operation_id: "getHardware".to_string(),
            tags: vec!["Hardware".to_string()],
            parameters: vec![],
            request_body: None,
            responses: {
                let mut responses = HashMap::new();
                responses.insert("200".to_string(), Response {
                    description: "Hardware configuration".to_string(),
                    content: HashMap::new(),
                });
                responses
            },
            security: vec![SecurityRequirement { bearer_auth: vec![] }],
        }),
        ..Default::default()
    });
}

fn add_process_endpoints(spec: &mut OpenApiSpec) {
    // GET /api/processes
    spec.paths.insert("/api/processes".to_string(), PathItem {
        get: Some(Operation {
            summary: "List running processes".to_string(),
            description: Some("Get all running processes for the current user".to_string()),
            operation_id: "getProcesses".to_string(),
            tags: vec!["Processes".to_string()],
            parameters: vec![
                Parameter {
                    name: "page".to_string(),
                    location: "query".to_string(),
                    description: Some("Page number".to_string()),
                    required: false,
                    schema: Schema::integer(),
                },
                Parameter {
                    name: "limit".to_string(),
                    location: "query".to_string(),
                    description: Some("Items per page".to_string()),
                    required: false,
                    schema: Schema::integer(),
                },
            ],
            request_body: None,
            responses: {
                let mut responses = HashMap::new();
                responses.insert("200".to_string(), Response {
                    description: "List of processes".to_string()),
                    content: HashMap::new(),
                });
                responses
            },
            security: vec![SecurityRequirement { bearer_auth: vec![] }],
        }),
        ..Default::default()
    });
}