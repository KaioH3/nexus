//! Nexus Protocol Core
//!
//! Types, traits, and message framing for the Nexus Protocol.
//! This crate is the foundation of the protocol - it defines the
//! type system, message types, and core abstractions.

pub mod message;
pub mod version;
pub mod capabilities;
pub mod sandbox_policy;
pub mod language;
pub mod error;
pub mod security_headers;
pub mod rate_limit;
pub mod prompt_guard;
pub mod binary_protocol;
pub mod connection_pool;
pub mod auth;

pub use message::Message;
pub use version::Version;
pub use capabilities::Capabilities;
pub use sandbox_policy::SandboxPolicy;
pub use language::Language;
pub use error::{Error, ErrorCode};
pub use security_headers::SecurityHeaders;
pub use rate_limit::{RateLimit, RateLimitTracker, TokenBucket};
pub use prompt_guard::{PromptInjectionGuard, PromptInjectionError};
pub use binary_protocol::{BinaryMessage, BinaryMsgType, BINARY_PROTOCOL_VERSION};
pub use connection_pool::{ConnectionPool, PoolConfig, PooledConnection};
pub use auth::ApiKeyConfig;
