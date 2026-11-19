use super::account::WatchlistActionStatusOutput;
use budna_mcp_client::{InventoryReceipt, InventoryReview};
use rmcp::schemars;
use serde::Serialize;
#[derive(Debug, Serialize, schemars::JsonSchema)]
pub struct InventoryOutput {
    listing_id: String,
    revision: i64,
    available_quantity: i32,
    checkout_reserved_quantity: i64,
    is_sold_out: bool,
    can_set_stock: bool,
    can_record_external_sale: bool,
    pending_offers_affected: i64,
}
impl From<InventoryReview> for InventoryOutput {
    fn from(r: InventoryReview) -> Self {
        Self {
            listing_id: r.listing_id,
            revision: r.revision,
            available_quantity: r.available_quantity,
            checkout_reserved_quantity: r.checkout_reserved_quantity,
            is_sold_out: r.is_sold_out,
            can_set_stock: r.can_set_stock,
            can_record_external_sale: r.can_record_external_sale,
            pending_offers_affected: r.pending_offers_affected,
        }
    }
}
#[derive(Debug, Serialize, schemars::JsonSchema)]
pub struct InventoryReceiptOutput {
    listing_id: String,
    action_id: String,
    status: WatchlistActionStatusOutput,
    operation: String,
}
impl From<InventoryReceipt> for InventoryReceiptOutput {
    fn from(r: InventoryReceipt) -> Self {
        Self {
            listing_id: r.listing_id,
            action_id: r.action_id,
            status: r.status.into(),
            operation: r.operation,
        }
    }
}
