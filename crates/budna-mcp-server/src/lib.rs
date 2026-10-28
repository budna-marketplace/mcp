#![cfg_attr(test, recursion_limit = "256")]

//! MCP routing, schemas, bounded projections, and capability
//! enforcement for Budna.

mod account;
mod http_transport;
mod mcp_apps;
mod output;
mod params;
mod tools;

pub use account::AccountMcpServer;
pub use http_transport::{account_http_router, public_http_router, stateless_public_http_router};
pub use tools::BudnaMcpServer;
