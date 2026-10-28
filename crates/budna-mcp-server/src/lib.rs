#![cfg_attr(test, recursion_limit = "256")]

//! MCP routing, schemas, bounded projections, and public Explore capability
//! enforcement for Budna.

mod account;
mod mcp_apps;
mod output;
mod params;
mod tools;

pub use account::AccountMcpServer;
pub use tools::BudnaMcpServer;
