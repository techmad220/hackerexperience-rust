// External integration classes (Priority 4)

pub mod facebook;
pub mod social;
pub mod mailer;
pub mod ses;
pub mod python;

// Re-export for convenience
pub use facebook::*;
pub use social::*;
pub use mailer::*;
pub use ses::*;
pub use python::*;