//! sloppy: Fast regex-based detection of AI prose tells.
//!
//! No LLM calls, no heavy NLP. Pure regex. Runs in <10ms per piece.

pub mod checks;
pub mod config;
pub mod detector;
pub mod models;
pub mod voice;

pub use config::{Config, load_config};
pub use detector::analyze;
pub use models::{CheckScore, SlopFlag, SlopResult};
pub use voice::{generate_chat_prompt, generate_voice_directive};
