use super::{
    AccountApiClient, AccountWatchlistActionStatus, ClientError, DelegatedAccess,
    INVENTORY_WRITE_SCOPE, LISTINGS_READ_SCOPE, invalid_input, invalid_response, is_action_id,
};
use crate::is_listing_id;
use reqwest::Method;
use serde::{Deserialize, Serialize};

#[derive(Debug, Deserialize)]
pub struct InventoryReview {
    pub listing_id: String,
    pub revision: i64,
    pub available_quantity: i32,
    pub checkout_reserved_quantity: i64,
    pub is_sold_out: bool,
    pub can_set_stock: bool,
    pub can_record_external_sale: bool,
    pub pending_offers_affected: i64,
}
#[derive(Debug, Clone, Copy, Serialize)]
#[serde(tag = "kind", rename_all = "snake_case")]
pub enum StockOperation {
    SetAvailable { quantity: i32 },
    RecordExternalSale { expected_pending_offers: i64 },
}
#[derive(Debug, Serialize)]
pub struct InventoryChange {
    pub expected_revision: i64,
    pub expected_quantity: i32,
    pub expected_reserved: i64,
    pub change: StockOperation,
}
#[derive(Debug, Deserialize)]
pub struct InventoryReceipt {
    pub action_id: String,
    pub listing_id: String,
    pub operation: String,
    pub status: AccountWatchlistActionStatus,
}
impl AccountApiClient {
    pub async fn listing_inventory(
        &self,
        access: &DelegatedAccess,
        id: &str,
    ) -> Result<InventoryReview, ClientError> {
        const OP: &str = "get_my_listing_inventory";
        self.require_scope(access, LISTINGS_READ_SCOPE, OP)?;
        if !is_listing_id(id) || !self.write_permissions().inventory {
            return Err(invalid_input(
                OP,
                "Invalid listing or inventory unavailable",
            ));
        }
        let r: InventoryReview = self
            .get(
                OP,
                &format!("mcp/account/listings/{id}/inventory"),
                access,
                &[],
            )
            .await?;
        if r.listing_id != id
            || r.revision <= 0
            || r.available_quantity < 0
            || r.checkout_reserved_quantity < 0
            || r.pending_offers_affected < 0
            || (r.is_sold_out && r.available_quantity != 0)
        {
            return Err(invalid_response(OP));
        }
        Ok(r)
    }
    pub async fn change_inventory(
        &self,
        access: &DelegatedAccess,
        id: &str,
        input: &InventoryChange,
        key: &str,
    ) -> Result<InventoryReceipt, ClientError> {
        let (op, action, valid) = match input.change {
            StockOperation::SetAvailable { quantity } => (
                "set_my_listing_stock",
                "set_listing_stock",
                (1..=10000).contains(&quantity),
            ),
            StockOperation::RecordExternalSale {
                expected_pending_offers,
            } => (
                "record_my_listing_sold_elsewhere",
                "record_external_sale",
                input.expected_quantity > 0 && expected_pending_offers >= 0,
            ),
        };
        if !is_listing_id(id)
            || input.expected_revision < 1
            || input.expected_quantity < 0
            || input.expected_reserved < 0
            || !valid
        {
            return Err(invalid_input(op, "Invalid reviewed stock or quantity"));
        }
        let request = self
            .action_request(
                op,
                INVENTORY_WRITE_SCOPE,
                access,
                Method::POST,
                &format!("mcp/account/listings/{id}/inventory"),
                key,
            )?
            .json(input);
        self.direct_receipt(op, request, |r: &InventoryReceipt| {
            r.listing_id == id && r.operation == action && is_action_id(&r.action_id)
        })
        .await
    }
}
