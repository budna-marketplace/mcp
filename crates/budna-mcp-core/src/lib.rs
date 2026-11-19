//! Validated configuration and capability-profile types shared by Budna MCP
//! transports and servers.

pub mod account;
pub mod config;
pub mod policy;
pub mod write_permissions;
pub use write_permissions::AccountWritePermissions;

pub use account::{ACCOUNT_MCP_PATH, AccountSettings};
pub use config::{
    ConfigError, DEFAULT_API_URL, DEFAULT_HTTP_ALLOWED_HOSTS, DEFAULT_HTTP_ALLOWED_ORIGINS,
    DEFAULT_HTTP_PORT, DEFAULT_IMAGE_ORIGIN, DEFAULT_LISTING_ORIGIN, DEFAULT_REQUEST_TIMEOUT_SECS,
    HttpServerSettings, MAX_REQUEST_TIMEOUT_SECS, PublicUrlSettings, Settings, Transport,
};
pub use policy::{ToolCapability, ToolPolicy};

/// Formatting characters that hide or change the reading order of marketplace text.
pub const fn is_hidden_format(ch: char) -> bool {
    matches!(ch, '\u{00ad}' | '\u{034f}' | '\u{061c}' | '\u{180e}' | '\u{200b}'..='\u{200f}' | '\u{2028}'..='\u{202e}' | '\u{2060}'..='\u{206f}' | '\u{feff}' | '\u{fff9}'..='\u{fffb}' | '\u{e0000}'..='\u{e007f}')
}
