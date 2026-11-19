use super::*;
use crate::output::fulfillment::{ReceiptOutput, TermsOutput};
use budna_mcp_client::{FulfillmentOperation as Op, OutboundTracking};
use rmcp::schemars;
use serde::Deserialize;

#[derive(Debug, Deserialize, schemars::JsonSchema)]
#[serde(deny_unknown_fields)]
struct Review {
    #[schemars(range(min = 1))]
    order_id: i64,
    operation: Op,
}
#[derive(Debug, Deserialize, schemars::JsonSchema)]
#[serde(tag = "operation", rename_all = "snake_case", deny_unknown_fields)]
enum SellerChange {
    MarkOrderDispatched {},
    SetOrderTracking { tracking: OutboundTracking },
    RecordMeetupHandover {},
}
#[derive(Debug, Deserialize, schemars::JsonSchema)]
#[serde(rename_all = "snake_case")]
enum ReceiptChoice {
    ConfirmOrderDelivery,
    ConfirmMeetupReceipt,
}
#[derive(Debug, Deserialize, schemars::JsonSchema)]
#[serde(deny_unknown_fields)]
struct Seller {
    #[schemars(range(min = 1))]
    order_id: i64,
    #[schemars(length(min = 43, max = 43))]
    expected_revision: String,
    #[schemars(length(min = 36, max = 36))]
    request_id: String,
    change: SellerChange,
}
#[derive(Debug, Deserialize, schemars::JsonSchema)]
#[serde(deny_unknown_fields)]
struct Booking {
    #[schemars(range(min = 1))]
    order_id: i64,
    #[schemars(length(min = 43, max = 43))]
    expected_revision: String,
    #[schemars(length(min = 36, max = 36))]
    request_id: String,
}
#[derive(Debug, Deserialize, schemars::JsonSchema)]
#[serde(deny_unknown_fields)]
struct Buyer {
    #[schemars(range(min = 1))]
    order_id: i64,
    #[schemars(length(min = 43, max = 43))]
    expected_revision: String,
    #[schemars(length(min = 36, max = 36))]
    request_id: String,
    operation: ReceiptChoice,
}

#[tool_router(router=fulfillment_review_router,vis="pub(super)")]
impl AccountMcpServer {
    #[tool(
        title = "Review delivery or pickup for my order",
        description = "Review the exact affected orders and listings, carrier, held balance, release timing and non-receipt disputes before a fulfillment action. Requires fulfillmentreview:read.",
        annotations(
            read_only_hint = true,
            destructive_hint = false,
            idempotent_hint = true,
            open_world_hint = true
        )
    )]
    async fn get_my_fulfillment_terms(
        &self,
        Parameters(input): Parameters<SafeToolParams<Review>>,
        context: RequestContext<RoleServer>,
    ) -> Result<Json<TermsOutput>, CallToolResult> {
        const TOOL: &str = "get_my_fulfillment_terms";
        let p = input.parse().map_err(|e| input_error(TOOL, &e))?;
        let access = request_access(&context, TOOL)?;
        self.execute(TOOL, async {
            let value = self
                .client
                .fulfillment_terms(&access, p.order_id, p.operation)
                .await?;
            Ok((value, self.public_urls.listing_origin()))
        })
        .await
    }
}

#[tool_router(router=seller_fulfillment_router,vis="pub(super)")]
impl AccountMcpServer {
    #[tool(
        title = "Update fulfillment on my sale",
        description = "Declare an actual dispatch or handover, or supply tracking for one eligible sale. A handover starts the buyer’s seven-day response window. Reuse the same request ID after interruption. Requires sellerfulfillment:write.",
        annotations(
            read_only_hint = false,
            destructive_hint = true,
            idempotent_hint = true,
            open_world_hint = true
        )
    )]
    async fn update_my_seller_fulfillment(
        &self,
        Parameters(input): Parameters<SafeToolParams<Seller>>,
        context: RequestContext<RoleServer>,
    ) -> Result<Json<ReceiptOutput>, CallToolResult> {
        const TOOL: &str = "update_my_seller_fulfillment";
        let p = input.parse().map_err(|e| input_error(TOOL, &e))?;
        let access = request_access(&context, TOOL)?;
        self.execute(TOOL, async {
            let (op, tracking) = match &p.change {
                SellerChange::MarkOrderDispatched {} => (Op::MarkOrderDispatched, None),
                SellerChange::SetOrderTracking { tracking } => {
                    (Op::SetOrderTracking, Some(tracking))
                }
                SellerChange::RecordMeetupHandover {} => (Op::RecordMeetupHandover, None),
            };
            let value = self
                .client
                .change_fulfillment(
                    &access,
                    p.order_id,
                    op,
                    &p.expected_revision,
                    tracking,
                    &p.request_id,
                )
                .await?;
            Ok((value, self.public_urls.listing_origin()))
        })
        .await
    }
}

#[tool_router(router=carrier_bookings_router,vis="pub(super)")]
impl AccountMcpServer {
    #[tool(
        title = "Book the selected carrier for my sale",
        description = "Book one already-funded order or its reviewed combined parcel using its saved carrier and destination. The shipping label is available on the Budna order page. Reuse the same request ID to check an interrupted booking. Requires carrierbookings:write.",
        annotations(
            read_only_hint = false,
            destructive_hint = true,
            idempotent_hint = true,
            open_world_hint = true
        )
    )]
    async fn book_my_order_shipment(
        &self,
        Parameters(input): Parameters<SafeToolParams<Booking>>,
        context: RequestContext<RoleServer>,
    ) -> Result<Json<ReceiptOutput>, CallToolResult> {
        const TOOL: &str = "book_my_order_shipment";
        let p = input.parse().map_err(|e| input_error(TOOL, &e))?;
        let access = request_access(&context, TOOL)?;
        self.execute(TOOL, async {
            let value = self
                .client
                .change_fulfillment(
                    &access,
                    p.order_id,
                    Op::BookOrderShipment,
                    &p.expected_revision,
                    None,
                    &p.request_id,
                )
                .await?;
            Ok((value, self.public_urls.listing_origin()))
        })
        .await
    }
}

#[tool_router(router=buyer_receipts_router,vis="pub(super)")]
impl AccountMcpServer {
    #[tool(
        title = "Request my approval for receipt of an order",
        description = "Open a first-party approval for the buyer to confirm actual delivery or pickup. A delivery starts its inspection clock. A pickup can immediately release held funds and close listed non-receipt disputes. Requires buyerreceipts:write.",
        annotations(
            read_only_hint = false,
            destructive_hint = true,
            idempotent_hint = true,
            open_world_hint = true
        )
    )]
    async fn confirm_my_order_receipt(
        &self,
        Parameters(input): Parameters<SafeToolParams<Buyer>>,
        context: RequestContext<RoleServer>,
    ) -> Result<Json<ReceiptOutput>, CallToolResult> {
        const TOOL: &str = "confirm_my_order_receipt";
        let p = input.parse().map_err(|e| input_error(TOOL, &e))?;
        let access = request_access(&context, TOOL)?;
        self.execute(TOOL, async {
            let op = match p.operation {
                ReceiptChoice::ConfirmOrderDelivery => Op::ConfirmOrderDelivery,
                ReceiptChoice::ConfirmMeetupReceipt => Op::ConfirmMeetupReceipt,
            };
            let value = self
                .client
                .change_fulfillment(
                    &access,
                    p.order_id,
                    op,
                    &p.expected_revision,
                    None,
                    &p.request_id,
                )
                .await?;
            Ok((value, self.public_urls.listing_origin()))
        })
        .await
    }
}
