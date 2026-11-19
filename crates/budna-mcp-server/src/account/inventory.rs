use super::*;
use crate::output::inventory::{InventoryOutput, InventoryReceiptOutput};
use budna_mcp_client::{InventoryChange, LISTING_ID_PATTERN, StockOperation};
use rmcp::schemars;
use serde::Deserialize;
#[derive(Debug, Deserialize, schemars::JsonSchema)]
#[serde(deny_unknown_fields)]
struct StockListing {
    #[schemars(length(min=44,max=44),regex(pattern=LISTING_ID_PATTERN))]
    listing_id: String,
}
#[derive(Debug, Deserialize, schemars::JsonSchema)]
#[serde(deny_unknown_fields)]
struct ReviewedStock {
    #[schemars(length(min=44,max=44),regex(pattern=LISTING_ID_PATTERN))]
    listing_id: String,
    #[schemars(range(min = 1))]
    expected_revision: i64,
    #[schemars(range(min = 0))]
    expected_quantity: i32,
    #[schemars(range(min = 0))]
    expected_reserved: i64,
    #[schemars(length(min = 36, max = 36))]
    request_id: String,
}
impl ReviewedStock {
    fn change(&self, change: StockOperation) -> InventoryChange {
        InventoryChange {
            expected_revision: self.expected_revision,
            expected_quantity: self.expected_quantity,
            expected_reserved: self.expected_reserved,
            change,
        }
    }
}
#[derive(Debug, Deserialize, schemars::JsonSchema)]
#[serde(deny_unknown_fields)]
struct SetStockParams {
    reviewed: ReviewedStock,
    #[schemars(range(min = 1, max = 10000))]
    quantity: i32,
}
#[derive(Debug, Deserialize, schemars::JsonSchema)]
#[serde(deny_unknown_fields)]
struct ExternalSaleParams {
    reviewed: ReviewedStock,
    #[schemars(range(min = 0))]
    expected_pending_offers: i64,
}
#[tool_router(router=inventory_router,vis="pub(super)")]
impl AccountMcpServer {
    #[tool(
        title = "Review my listing inventory",
        description = "Review one owned listing’s available stock, checkout reservations, open offers affected and eligibility. Use the revision and quantities when asking to change its shelf level or record an outside sale. Requires listings:read.",
        annotations(
            read_only_hint = true,
            destructive_hint = false,
            idempotent_hint = true,
            open_world_hint = true
        )
    )]
    async fn get_my_listing_inventory(
        &self,
        Parameters(input): Parameters<SafeToolParams<StockListing>>,
        context: RequestContext<RoleServer>,
    ) -> Result<Json<InventoryOutput>, CallToolResult> {
        const OP: &str = "get_my_listing_inventory";
        let input = input.parse().map_err(|e| input_error(OP, &e))?;
        let access = request_access(&context, OP)?;
        self.execute(
            OP,
            self.client.listing_inventory(&access, &input.listing_id),
        )
        .await
    }
    #[tool(
        title = "Set the available stock on my listing",
        description = "Set the absolute shelf quantity for one eligible in-stock listing. A positive count on a sold-out shelf can reopen it for sale. Use the seller’s count and review the current checkout reservations first. Reuse the same request ID on retry. Requires inventory:write.",
        annotations(
            read_only_hint = false,
            destructive_hint = true,
            idempotent_hint = true,
            open_world_hint = true
        )
    )]
    async fn set_my_listing_stock(
        &self,
        Parameters(input): Parameters<SafeToolParams<SetStockParams>>,
        context: RequestContext<RoleServer>,
    ) -> Result<Json<InventoryReceiptOutput>, CallToolResult> {
        const OP: &str = "set_my_listing_stock";
        let input = input.parse().map_err(|e| input_error(OP, &e))?;
        let access = request_access(&context, OP)?;
        self.execute(
            OP,
            self.client.change_inventory(
                &access,
                &input.reviewed.listing_id,
                &input.reviewed.change(StockOperation::SetAvailable {
                    quantity: input.quantity,
                }),
                &input.reviewed.request_id,
            ),
        )
        .await
    }
    #[tool(
        title = "Record my remaining stock sold outside Budna",
        description = "On the seller’s explicit instruction record the entire reviewed remaining quantity as sold outside Budna. This ends the eligible listing and rejects its open offers. Review the offer count, listing and quantity with the seller first. A standing bid or ongoing purchase prevents this action. Requires inventory:write.",
        annotations(
            read_only_hint = false,
            destructive_hint = true,
            idempotent_hint = true,
            open_world_hint = true
        )
    )]
    async fn record_my_listing_sold_elsewhere(
        &self,
        Parameters(input): Parameters<SafeToolParams<ExternalSaleParams>>,
        context: RequestContext<RoleServer>,
    ) -> Result<Json<InventoryReceiptOutput>, CallToolResult> {
        const OP: &str = "record_my_listing_sold_elsewhere";
        let input = input.parse().map_err(|e| input_error(OP, &e))?;
        let access = request_access(&context, OP)?;
        self.execute(
            OP,
            self.client.change_inventory(
                &access,
                &input.reviewed.listing_id,
                &input.reviewed.change(StockOperation::RecordExternalSale {
                    expected_pending_offers: input.expected_pending_offers,
                }),
                &input.reviewed.request_id,
            ),
        )
        .await
    }
}
