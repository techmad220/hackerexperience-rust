use anyhow::{anyhow, Result};
use regex::Regex;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use thiserror::Error;

#[derive(Debug, Error)]
pub enum PurifierError {
    #[error("Invalid configuration: {0}")]
    Configuration(String),
    #[error("Regex error: {0}")]
    Regex(#[from] regex::Error),
    #[error("Purification error: {0}")]
    Purification(String),
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PurifierConfig {
    pub allow_html: bool,
    pub allowed_tags: Vec<String>,
    pub allowed_attributes: HashMap<String, Vec<String>>,
    pub auto_paragraph: bool,
    pub cache_enabled: bool,
    pub cache_dir: Option<String>,
}

impl Default for PurifierConfig {
    fn default() -> Self {
        Self {
            allow_html: false,
            allowed_tags: vec![
                "p".to_string(),
                "br".to_string(),
                "strong".to_string(),
                "em".to_string(),
                "a".to_string(),
            ],
            allowed_attributes: {
                let mut attrs = HashMap::new();
                attrs.insert("a".to_string(), vec!["href".to_string(), "title".to_string()]);
                attrs
            },
            auto_paragraph: true,
            cache_enabled: false,
            cache_dir: None,
        }
    }
}

/// HTML Purifier ported from PHP HTMLPurifier library
/// Provides HTML sanitization and XSS protection
pub struct Purifier {
    config: PurifierConfig,
    dangerous_patterns: Vec<Regex>,
    tag_patterns: HashMap<String, Regex>,
}

impl Purifier {
    /// Create new purifier with configuration
    pub fn new(config: PurifierConfig) -> Result<Self, PurifierError> {
        let mut purifier = Self {
            config,
            dangerous_patterns: vec![],
            tag_patterns: HashMap::new(),
        };

        purifier.init_patterns()?;
        Ok(purifier)
    }

    /// Create purifier with default configuration
    pub fn default() -> Result<Self, PurifierError> {
        Self::new(PurifierConfig::default())
    }

    /// Initialize dangerous patterns and tag patterns
    fn init_patterns(&mut self) -> Result<(), PurifierError> {
        // Common XSS patterns
        let dangerous_patterns = vec![
            r"(?i)<script[^>]*>.*?</script>",
            r"(?i)javascript:",
            r"(?i)vbscript:",
            r"(?i)onload=",
            r"(?i)onerror=",
            r"(?i)onclick=",
            r"(?i)onmouseover=",
            r"(?i)onfocus=",
            r"(?i)onblur=",
            r"(?i)onkeyup=",
            r"(?i)onkeydown=",
            r"(?i)onchange=",
            r"(?i)onsubmit=",
            r"(?i)<iframe[^>]*>.*?</iframe>",
            r"(?i)<object[^>]*>.*?</object>",
            r"(?i)<embed[^>]*>.*?</embed>",
            r"(?i)<form[^>]*>.*?</form>",
            r"(?i)<input[^>]*>",
            r"(?i)<textarea[^>]*>.*?</textarea>",
            r"(?i)<link[^>]*>",
            r"(?i)<meta[^>]*>",
            r"(?i)<style[^>]*>.*?</style>",
        ];

        for pattern in dangerous_patterns {
            self.dangerous_patterns.push(Regex::new(pattern)?);
        }

        // Tag validation patterns
        for tag in &self.config.allowed_tags {
            let pattern = format!(r"(?i)<{}([^>]*)>(.*?)</{}>", tag, tag);
            self.tag_patterns.insert(tag.clone(), Regex::new(&pattern)?);
        }

        Ok(())
    }

    /// Purify HTML input and return clean output
    pub fn purify(&self, input: &str) -> Result<String, PurifierError> {
        let mut output = input.to_string();

        // Step 1: Remove dangerous patterns
        output = self.remove_dangerous_content(&output)?;

        // Step 2: If HTML is not allowed, escape all HTML
        if !self.config.allow_html {
            output = self.escape_html(&output);
        } else {
            // Step 3: Validate and clean allowed HTML tags
            output = self.clean_allowed_html(&output)?;
        }

        // Step 4: Auto-paragraph if enabled
        if self.config.auto_paragraph {
            output = self.auto_paragraph(&output);
        }

        // Step 5: Trim whitespace
        output = output.trim().to_string();

        Ok(output)
    }

    /// Remove dangerous content using regex patterns
    fn remove_dangerous_content(&self, input: &str) -> Result<String, PurifierError> {
        let mut output = input.to_string();

        for pattern in &self.dangerous_patterns {
            output = pattern.replace_all(&output, "").to_string();
        }

        Ok(output)
    }

    /// Escape HTML entities
    fn escape_html(&self, input: &str) -> String {
        input
            .replace("&", "&amp;")
            .replace("<", "&lt;")
            .replace(">", "&gt;")
            .replace("\"", "&quot;")
            .replace("'", "&#x27;")
            .replace("/", "&#x2F;")
    }

    /// Clean and validate allowed HTML tags
    fn clean_allowed_html(&self, input: &str) -> Result<String, PurifierError> {
        let mut output = input.to_string();

        // Remove all HTML tags first, then add back allowed ones
        let tag_regex = Regex::new(r"<[^>]+>")?;
        let mut cleaned = tag_regex.replace_all(&output, "").to_string();

        // Process allowed tags from original input
        for (tag, pattern) in &self.tag_patterns {
            for capture in pattern.captures_iter(input) {
                if let (Some(attributes), Some(content)) = (capture.get(1), capture.get(2)) {
                    let clean_attrs = self.clean_attributes(tag, attributes.as_str())?;
                    let clean_content = self.purify(content.as_str())?;
                    
                    let clean_tag = if clean_attrs.is_empty() {
                        format!("<{}>{}</{}>", tag, clean_content, tag)
                    } else {
                        format!("<{} {}>{}</{}>", tag, clean_attrs, clean_content, tag)
                    };
                    
                    cleaned = cleaned.replace(content.as_str(), &clean_tag);
                }
            }
        }

        Ok(cleaned)
    }

    /// Clean attributes for a specific tag
    fn clean_attributes(&self, tag: &str, attributes: &str) -> Result<String, PurifierError> {
        let mut clean_attrs = vec![];

        if let Some(allowed_attrs) = self.config.allowed_attributes.get(tag) {
            let attr_regex = Regex::new(r#"(\w+)=["']([^"']*?)["']"#)?;
            
            for capture in attr_regex.captures_iter(attributes) {
                if let (Some(attr_name), Some(attr_value)) = (capture.get(1), capture.get(2)) {
                    let attr_name = attr_name.as_str();
                    let attr_value = attr_value.as_str();
                    
                    if allowed_attrs.contains(&attr_name.to_string()) {
                        let clean_value = self.clean_attribute_value(attr_name, attr_value)?;
                        clean_attrs.push(format!("{}=\"{}\"", attr_name, clean_value));
                    }
                }
            }
        }

        Ok(clean_attrs.join(" "))
    }

    /// Clean individual attribute value
    fn clean_attribute_value(&self, attr_name: &str, value: &str) -> Result<String, PurifierError> {
        let mut clean_value = value.to_string();

        // Special handling for href attributes
        if attr_name == "href" {
            // Only allow http, https, mailto, and relative URLs
            let url_regex = Regex::new(r"^(https?://|mailto:|/|#)")?;
            if !url_regex.is_match(&clean_value) {
                clean_value = "#".to_string(); // Default to anchor
            }
        }

        // Remove any script-like content
        for pattern in &self.dangerous_patterns {
            clean_value = pattern.replace_all(&clean_value, "").to_string();
        }

        // Escape quotes and other dangerous characters
        clean_value = clean_value
            .replace("\"", "&quot;")
            .replace("'", "&#x27;");

        Ok(clean_value)
    }

    /// Convert line breaks to paragraphs
    fn auto_paragraph(&self, input: &str) -> String {
        if input.trim().is_empty() {
            return input.to_string();
        }

        let paragraphs: Vec<&str> = input.split("\n\n").collect();
        let mut output = vec![];

        for paragraph in paragraphs {
            let trimmed = paragraph.trim();
            if !trimmed.is_empty() {
                // Don't wrap if already contains block tags
                if trimmed.contains("<p>") || trimmed.contains("<div>") || trimmed.contains("<h") {
                    output.push(trimmed.to_string());
                } else {
                    // Convert single line breaks to <br> tags
                    let with_breaks = trimmed.replace("\n", "<br>");
                    output.push(format!("<p>{}</p>", with_breaks));
                }
            }
        }

        output.join("\n\n")
    }

    /// Purify URL for safe usage
    pub fn purify_url(&self, url: &str) -> Result<String, PurifierError> {
        let url_regex = Regex::new(r"^(https?://[a-zA-Z0-9\-._~:/?#[\]@!$&'()*+,;=%]+|/[a-zA-Z0-9\-._~:/?#[\]@!$&'()*+,;=%]*|#[a-zA-Z0-9\-._~:/?#[\]@!$&'()*+,;=%]*)$")?;
        
        if url_regex.is_match(url) {
            Ok(url.to_string())
        } else {
            Ok("#".to_string()) // Default safe URL
        }
    }

    /// Purify CSS for safe usage (basic implementation)
    pub fn purify_css(&self, css: &str) -> Result<String, PurifierError> {
        // Remove dangerous CSS patterns
        let dangerous_css_patterns = vec![
            r"(?i)javascript:",
            r"(?i)vbscript:",
            r"(?i)expression\s*\(",
            r"(?i)@import",
            r"(?i)behavior:",
            r"(?i)-moz-binding:",
        ];

        let mut output = css.to_string();
        
        for pattern_str in dangerous_css_patterns {
            let pattern = Regex::new(pattern_str)?;
            output = pattern.replace_all(&output, "").to_string();
        }

        Ok(output)
    }

    /// Get current configuration
    pub fn get_config(&self) -> &PurifierConfig {
        &self.config
    }

    /// Update configuration
    pub fn set_config(&mut self, config: PurifierConfig) -> Result<(), PurifierError> {
        self.config = config;
        self.dangerous_patterns.clear();
        self.tag_patterns.clear();
        self.init_patterns()?;
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_purifier_creation() {
        let purifier = Purifier::default();
        assert!(purifier.is_ok());
    }

    #[test]
    fn test_script_removal() {
        let purifier = Purifier::default().map_err(|e| anyhow::anyhow!("Error: {}", e))?;
        let input = "Hello <script>alert('xss')</script> World";
        let result = purifier.purify(input).map_err(|e| anyhow::anyhow!("Error: {}", e))?;
        assert!(!result.contains("<script>"));
        assert!(result.contains("Hello"));
        assert!(result.contains("World"));
    }

    #[test]
    fn test_html_escaping() {
        let purifier = Purifier::default().map_err(|e| anyhow::anyhow!("Error: {}", e))?;
        let input = "Hello <b>World</b>";
        let result = purifier.purify(input).map_err(|e| anyhow::anyhow!("Error: {}", e))?;
        assert!(result.contains("&lt;b&gt;"));
        assert!(result.contains("&lt;/b&gt;"));
    }

    #[test]
    fn test_auto_paragraph() {
        let purifier = Purifier::default().map_err(|e| anyhow::anyhow!("Error: {}", e))?;
        let input = "Line 1\n\nLine 2";
        let result = purifier.purify(input).map_err(|e| anyhow::anyhow!("Error: {}", e))?;
        assert!(result.contains("<p>Line 1</p>"));
        assert!(result.contains("<p>Line 2</p>"));
    }

    #[test]
    fn test_url_purification() {
        let purifier = Purifier::default().map_err(|e| anyhow::anyhow!("Error: {}", e))?;
        
        assert_eq!(purifier.purify_url("https://example.com").map_err(|e| anyhow::anyhow!("Error: {}", e))?, "https://example.com");
        assert_eq!(purifier.purify_url("/relative/path").map_err(|e| anyhow::anyhow!("Error: {}", e))?, "/relative/path");
        assert_eq!(purifier.purify_url("javascript:alert('xss')").map_err(|e| anyhow::anyhow!("Error: {}", e))?, "#");
    }

    #[test]
    fn test_css_purification() {
        let purifier = Purifier::default().map_err(|e| anyhow::anyhow!("Error: {}", e))?;
        let input = "color: red; background: url(javascript:alert('xss'));";
        let result = purifier.purify_css(input).map_err(|e| anyhow::anyhow!("Error: {}", e))?;
        assert!(result.contains("color: red"));
        assert!(!result.contains("javascript:"));
    }
}