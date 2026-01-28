//! PACT Plugin SDK for Rust
//!
//! This SDK enables building PACT Protocol domain plugins in Rust.
//!
//! # Features
//!
//! - Plugin interfaces and base implementations
//! - Type-safe rule and condition builders
//! - Event mapping utilities
//! - Validation helpers
//! - HTTP client for PACT Core
//!
//! # Example
//!
//! ```rust,no_run
//! use pact_sdk::{BasePlugin, PluginManifest, PactEntityType, PactRule};
//!
//! struct MyPlugin {
//!     base: BasePlugin,
//! }
//!
//! impl MyPlugin {
//!     fn new() -> Self {
//!         Self {
//!             base: BasePlugin::new(
//!                 PluginManifest {
//!                     id: "my-plugin".to_string(),
//!                     name: "My Plugin".to_string(),
//!                     version: "1.0.0".to_string(),
//!                     author: "My Company".to_string(),
//!                     description: "My custom PACT plugin".to_string(),
//!                     entity_types: vec!["MyEntity".to_string()],
//!                     event_types: vec!["MY_EVENT".to_string()],
//!                     dependencies: vec![],
//!                     pact_core_version: "0.1.0".to_string(),
//!                     config_schema: None,
//!                 },
//!                 vec![],
//!                 vec![],
//!             ),
//!         }
//!     }
//! }
//! ```

pub mod types;
pub mod plugin;
pub mod builder;
pub mod registry;
pub mod client;

pub use types::*;
pub use plugin::*;
pub use builder::*;
pub use registry::*;
pub use client::*;
