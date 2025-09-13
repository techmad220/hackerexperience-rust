use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::process::{Command, Output, Stdio};
use std::fs;
use std::path::Path;
use thiserror::Error;

#[derive(Debug, Error)]
pub enum PythonError {
    #[error("Python execution failed: {0}")]
    ExecutionFailed(String),
    #[error("Script not found: {0}")]
    ScriptNotFound(String),
    #[error("Invalid script content: {0}")]
    InvalidScript(String),
    #[error("Environment error: {0}")]
    EnvironmentError(String),
    #[error("Timeout error: {0}")]
    TimeoutError(String),
    #[error("Permission denied: {0}")]
    PermissionDenied(String),
    #[error("IO error: {0}")]
    IoError(String),
    #[error("JSON parse error: {0}")]
    JsonParseError(String),
    #[error("Configuration error: {0}")]
    ConfigError(String),
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PythonConfig {
    pub python_executable: String,
    pub script_directory: String,
    pub virtual_env: Option<String>,
    pub timeout_seconds: u64,
    pub max_memory_mb: Option<u64>,
    pub allowed_modules: Vec<String>,
    pub blocked_modules: Vec<String>,
    pub enable_logging: bool,
    pub log_directory: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PythonScript {
    pub name: String,
    pub content: String,
    pub filepath: Option<String>,
    pub parameters: HashMap<String, String>,
    pub environment_vars: HashMap<String, String>,
    pub timeout: Option<u64>,
    pub working_directory: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PythonResult {
    pub success: bool,
    pub exit_code: i32,
    pub stdout: String,
    pub stderr: String,
    pub execution_time: u64,
    pub started_at: DateTime<Utc>,
    pub finished_at: DateTime<Utc>,
    pub memory_used: Option<u64>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum PythonMode {
    Script,      // Execute a .py file
    Code,        // Execute inline Python code
    Module,      // Execute a Python module with -m
    Interactive, // Start an interactive session
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PythonEnvironment {
    pub name: String,
    pub path: String,
    pub python_version: String,
    pub installed_packages: Vec<PythonPackage>,
    pub is_virtual: bool,
    pub is_active: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PythonPackage {
    pub name: String,
    pub version: String,
    pub location: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PythonProcessInfo {
    pub pid: u32,
    pub started_at: DateTime<Utc>,
    pub script_name: String,
    pub status: ProcessStatus,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ProcessStatus {
    Running,
    Completed,
    Failed,
    Killed,
    Timeout,
}

pub struct Python {
    config: PythonConfig,
    current_environment: Option<PythonEnvironment>,
    running_processes: HashMap<u32, PythonProcessInfo>,
    execution_history: Vec<PythonResult>,
    script_cache: HashMap<String, PythonScript>,
}

impl Default for PythonConfig {
    fn default() -> Self {
        Self {
            python_executable: "python3".to_string(),
            script_directory: "/tmp/python_scripts".to_string(),
            virtual_env: None,
            timeout_seconds: 30,
            max_memory_mb: Some(512),
            allowed_modules: vec![
                "json".to_string(),
                "datetime".to_string(),
                "math".to_string(),
                "random".to_string(),
                "hashlib".to_string(),
                "base64".to_string(),
                "urllib".to_string(),
                "requests".to_string(),
            ],
            blocked_modules: vec![
                "os".to_string(),
                "subprocess".to_string(),
                "sys".to_string(),
                "importlib".to_string(),
            ],
            enable_logging: true,
            log_directory: Some("/tmp/python_logs".to_string()),
        }
    }
}

impl Python {
    pub fn new(config: PythonConfig) -> Result<Self, PythonError> {
        let mut python = Self {
            config,
            current_environment: None,
            running_processes: HashMap::new(),
            execution_history: Vec::new(),
            script_cache: HashMap::new(),
        };

        python.initialize()?;
        Ok(python)
    }

    pub fn with_default() -> Result<Self, PythonError> {
        Self::new(PythonConfig::default())
    }

    pub fn initialize(&mut self) -> Result<(), PythonError> {
        // Check if Python executable exists
        self.check_python_installation()?;
        
        // Create script directory if it doesn't exist
        if let Err(e) = fs::create_dir_all(&self.config.script_directory) {
            return Err(PythonError::IoError(format!("Failed to create script directory: {}", e)));
        }

        // Set up virtual environment if specified
        if let Some(venv_path) = &self.config.virtual_env {
            self.setup_virtual_environment(venv_path)?;
        }

        Ok(())
    }

    pub fn execute_script(&mut self, script: &PythonScript, mode: PythonMode) -> Result<PythonResult, PythonError> {
        let started_at = Utc::now();
        
        match mode {
            PythonMode::Script => self.execute_script_file(script, started_at),
            PythonMode::Code => self.execute_inline_code(script, started_at),
            PythonMode::Module => self.execute_module(script, started_at),
            PythonMode::Interactive => Err(PythonError::ConfigError("Interactive mode not supported in this context".to_string())),
        }
    }

    pub fn execute_script_file(&mut self, script: &PythonScript, started_at: DateTime<Utc>) -> Result<PythonResult, PythonError> {
        let script_path = if let Some(filepath) = &script.filepath {
            filepath.clone()
        } else {
            // Create temporary script file
            let script_filename = format!("{}/{}.py", self.config.script_directory, script.name);
            fs::write(&script_filename, &script.content)
                .map_err(|e| PythonError::IoError(format!("Failed to write script file: {}", e)))?;
            script_filename
        };

        if !Path::new(&script_path).exists() {
            return Err(PythonError::ScriptNotFound(script_path));
        }

        let mut command = self.build_command(&[&script_path], script)?;
        self.execute_command(command, started_at)
    }

    pub fn execute_inline_code(&mut self, script: &PythonScript, started_at: DateTime<Utc>) -> Result<PythonResult, PythonError> {
        if script.content.is_empty() {
            return Err(PythonError::InvalidScript("Script content cannot be empty".to_string()));
        }

        let mut command = self.build_command(&["-c", &script.content], script)?;
        self.execute_command(command, started_at)
    }

    pub fn execute_module(&mut self, script: &PythonScript, started_at: DateTime<Utc>) -> Result<PythonResult, PythonError> {
        if script.name.is_empty() {
            return Err(PythonError::InvalidScript("Module name cannot be empty".to_string()));
        }

        let mut args = vec!["-m", &script.name];
        for (key, value) in &script.parameters {
            args.push("--");
            args.push(key);
            args.push(value);
        }

        let mut command = self.build_command(&args, script)?;
        self.execute_command(command, started_at)
    }

    pub fn create_script(&mut self, name: &str, content: &str, parameters: Option<HashMap<String, String>>) -> PythonScript {
        let script = PythonScript {
            name: name.to_string(),
            content: content.to_string(),
            filepath: None,
            parameters: parameters.unwrap_or_default(),
            environment_vars: HashMap::new(),
            timeout: None,
            working_directory: None,
        };

        self.script_cache.insert(name.to_string(), script.clone());
        script
    }

    pub fn load_script_from_file(&mut self, filepath: &str) -> Result<PythonScript, PythonError> {
        if !Path::new(filepath).exists() {
            return Err(PythonError::ScriptNotFound(filepath.to_string()));
        }

        let content = fs::read_to_string(filepath)
            .map_err(|e| PythonError::IoError(format!("Failed to read script file: {}", e)))?;

        let name = Path::new(filepath)
            .file_stem()
            .and_then(|s| s.to_str())
            .unwrap_or("unnamed")
            .to_string();

        let script = PythonScript {
            name,
            content,
            filepath: Some(filepath.to_string()),
            parameters: HashMap::new(),
            environment_vars: HashMap::new(),
            timeout: None,
            working_directory: None,
        };

        Ok(script)
    }

    pub fn validate_script(&self, script: &PythonScript) -> Result<(), PythonError> {
        if script.content.is_empty() && script.filepath.is_none() {
            return Err(PythonError::InvalidScript("Script must have content or filepath".to_string()));
        }

        // Check for blocked modules
        for blocked in &self.config.blocked_modules {
            if script.content.contains(&format!("import {}", blocked)) ||
               script.content.contains(&format!("from {} import", blocked)) {
                return Err(PythonError::InvalidScript(format!("Blocked module detected: {}", blocked)));
            }
        }

        Ok(())
    }

    pub fn get_python_version(&self) -> Result<String, PythonError> {
        let output = Command::new(&self.config.python_executable)
            .arg("--version")
            .output()
            .map_err(|e| PythonError::ExecutionFailed(format!("Failed to get Python version: {}", e)))?;

        if output.status.success() {
            let version = String::from_utf8_lossy(&output.stdout).trim().to_string();
            Ok(version)
        } else {
            Err(PythonError::ExecutionFailed("Failed to determine Python version".to_string()))
        }
    }

    pub fn install_package(&mut self, package_name: &str, version: Option<&str>) -> Result<(), PythonError> {
        let package_spec = if let Some(ver) = version {
            format!("{}=={}", package_name, ver)
        } else {
            package_name.to_string()
        };

        let output = Command::new(&self.config.python_executable)
            .args(&["-m", "pip", "install", &package_spec])
            .output()
            .map_err(|e| PythonError::ExecutionFailed(format!("Failed to install package: {}", e)))?;

        if output.status.success() {
            Ok(())
        } else {
            let error_msg = String::from_utf8_lossy(&output.stderr);
            Err(PythonError::ExecutionFailed(format!("Package installation failed: {}", error_msg)))
        }
    }

    pub fn list_installed_packages(&self) -> Result<Vec<PythonPackage>, PythonError> {
        let output = Command::new(&self.config.python_executable)
            .args(&["-m", "pip", "list", "--format=json"])
            .output()
            .map_err(|e| PythonError::ExecutionFailed(format!("Failed to list packages: {}", e)))?;

        if output.status.success() {
            let packages_json = String::from_utf8_lossy(&output.stdout);
            let packages: Vec<serde_json::Value> = serde_json::from_str(&packages_json)
                .map_err(|e| PythonError::JsonParseError(format!("Failed to parse package list: {}", e)))?;

            let mut result = Vec::new();
            for package in packages {
                if let (Some(name), Some(version)) = (package.get("name"), package.get("version")) {
                    result.push(PythonPackage {
                        name: name.as_str().unwrap_or("").to_string(),
                        version: version.as_str().unwrap_or("").to_string(),
                        location: "".to_string(), // pip list doesn't include location
                    });
                }
            }
            Ok(result)
        } else {
            Err(PythonError::ExecutionFailed("Failed to list installed packages".to_string()))
        }
    }

    pub fn get_execution_history(&self) -> &Vec<PythonResult> {
        &self.execution_history
    }

    pub fn get_running_processes(&self) -> &HashMap<u32, PythonProcessInfo> {
        &self.running_processes
    }

    pub fn get_cached_script(&self, name: &str) -> Option<&PythonScript> {
        self.script_cache.get(name)
    }

    pub fn clear_cache(&mut self) {
        self.script_cache.clear();
    }

    pub fn clear_history(&mut self) {
        self.execution_history.clear();
    }

    // Private helper methods
    fn check_python_installation(&self) -> Result<(), PythonError> {
        let output = Command::new(&self.config.python_executable)
            .arg("--version")
            .output()
            .map_err(|_| PythonError::EnvironmentError("Python executable not found".to_string()))?;

        if !output.status.success() {
            return Err(PythonError::EnvironmentError("Python installation check failed".to_string()));
        }

        Ok(())
    }

    fn setup_virtual_environment(&mut self, venv_path: &str) -> Result<(), PythonError> {
        if !Path::new(venv_path).exists() {
            return Err(PythonError::EnvironmentError(format!("Virtual environment not found: {}", venv_path)));
        }

        // Update Python executable to use the virtual environment
        let venv_python = format!("{}/bin/python", venv_path);
        if Path::new(&venv_python).exists() {
            self.config.python_executable = venv_python;
        }

        Ok(())
    }

    fn build_command(&self, args: &[&str], script: &PythonScript) -> Result<Command, PythonError> {
        let mut command = Command::new(&self.config.python_executable);
        
        // Add arguments
        for arg in args {
            command.arg(arg);
        }

        // Set environment variables
        for (key, value) in &script.environment_vars {
            command.env(key, value);
        }

        // Set working directory if specified
        if let Some(work_dir) = &script.working_directory {
            command.current_dir(work_dir);
        }

        // Configure stdio
        command.stdout(Stdio::piped())
               .stderr(Stdio::piped())
               .stdin(Stdio::null());

        Ok(command)
    }

    fn execute_command(&mut self, mut command: Command, started_at: DateTime<Utc>) -> Result<PythonResult, PythonError> {
        let output = command.output()
            .map_err(|e| PythonError::ExecutionFailed(format!("Failed to execute command: {}", e)))?;

        let finished_at = Utc::now();
        let execution_time = (finished_at - started_at).num_milliseconds() as u64;

        let result = PythonResult {
            success: output.status.success(),
            exit_code: output.status.code().unwrap_or(-1),
            stdout: String::from_utf8_lossy(&output.stdout).to_string(),
            stderr: String::from_utf8_lossy(&output.stderr).to_string(),
            execution_time,
            started_at,
            finished_at,
            memory_used: None, // Would need external monitoring for this
        };

        self.execution_history.push(result.clone());
        Ok(result)
    }
}

// Helper functions for common Python operations
pub fn create_simple_script(name: &str, code: &str) -> PythonScript {
    PythonScript {
        name: name.to_string(),
        content: code.to_string(),
        filepath: None,
        parameters: HashMap::new(),
        environment_vars: HashMap::new(),
        timeout: None,
        working_directory: None,
    }
}

pub fn create_script_with_params(name: &str, code: &str, params: HashMap<String, String>) -> PythonScript {
    PythonScript {
        name: name.to_string(),
        content: code.to_string(),
        filepath: None,
        parameters: params,
        environment_vars: HashMap::new(),
        timeout: None,
        working_directory: None,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_create_simple_script() {
        let script = create_simple_script("test", "print('Hello, World!')");
        
        assert_eq!(script.name, "test");
        assert_eq!(script.content, "print('Hello, World!')");
        assert!(script.parameters.is_empty());
    }

    #[test]
    fn test_script_validation() {
        let python = Python::with_default().unwrap();
        
        // Valid script
        let valid_script = create_simple_script("valid", "print('Hello')");
        assert!(python.validate_script(&valid_script).is_ok());
        
        // Script with blocked module
        let blocked_script = create_simple_script("blocked", "import os\nos.system('ls')");
        assert!(python.validate_script(&blocked_script).is_err());
        
        // Empty script
        let empty_script = create_simple_script("empty", "");
        assert!(python.validate_script(&empty_script).is_err());
    }

    #[test]
    fn test_default_config() {
        let config = PythonConfig::default();
        
        assert_eq!(config.python_executable, "python3");
        assert_eq!(config.timeout_seconds, 30);
        assert!(config.allowed_modules.contains(&"json".to_string()));
        assert!(config.blocked_modules.contains(&"os".to_string()));
    }

    #[test]
    fn test_create_script_with_params() {
        let mut params = HashMap::new();
        params.insert("input".to_string(), "test_value".to_string());
        
        let script = create_script_with_params("parameterized", "print(sys.argv[1])", params);
        
        assert_eq!(script.parameters.len(), 1);
        assert_eq!(script.parameters.get("input"), Some(&"test_value".to_string()));
    }
}