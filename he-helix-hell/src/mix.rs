//! Mix-like functionality for Rust

pub struct MixTask {
    pub name: String,
    pub description: String,
}

impl MixTask {
    pub fn new(name: impl Into<String>, description: impl Into<String>) -> Self {
        Self {
            name: name.into(),
            description: description.into(),
        }
    }
}