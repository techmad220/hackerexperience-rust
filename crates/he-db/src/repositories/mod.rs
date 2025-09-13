// Repository pattern for database operations - replaces PHP class methods

pub mod user;
pub mod hardware;
pub mod process;
pub mod session;

// Re-export repositories
pub use user::*;
pub use hardware::*;
pub use process::*;
pub use session::*;