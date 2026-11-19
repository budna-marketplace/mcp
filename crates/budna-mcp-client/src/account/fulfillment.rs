use super::{
    AccountActionState, AccountApiClient, ApiEnvelope, ClientError, DelegatedAccess,
    actions::valid_action_text, invalid_input, invalid_response, is_action_id,
    shipping::valid_revision,
};
use crate::{Money, is_listing_id, money_is_valid};
use budna_mcp_core::write_permissions::{
    BUYER_RECEIPTS_SCOPE, CARRIER_BOOKINGS_SCOPE, FULFILLMENT_REVIEW_SCOPE,
    SELLER_FULFILLMENT_SCOPE,
};
use reqwest::{Method, header::AUTHORIZATION};
use serde::{Deserialize, Serialize};

#[derive(Clone, Copy, Debug, Deserialize, Serialize, schemars::JsonSchema, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum FulfillmentOperation {
    MarkOrderDispatched,
    SetOrderTracking,
    BookOrderShipment,
    RecordMeetupHandover,
    ConfirmOrderDelivery,
    ConfirmMeetupReceipt,
}
impl FulfillmentOperation {
    fn scope(self) -> &'static str {
        match self {
            Self::BookOrderShipment => CARRIER_BOOKINGS_SCOPE,
            Self::ConfirmOrderDelivery | Self::ConfirmMeetupReceipt => BUYER_RECEIPTS_SCOPE,
            _ => SELLER_FULFILLMENT_SCOPE,
        }
    }
    fn write_tool(self) -> &'static str {
        match self {
            Self::BookOrderShipment => "book_my_order_shipment",
            Self::ConfirmOrderDelivery | Self::ConfirmMeetupReceipt => "confirm_my_order_receipt",
            _ => "update_my_seller_fulfillment",
        }
    }
}
#[derive(Debug, Deserialize, Serialize, schemars::JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct OutboundTracking {
    #[schemars(length(min = 1, max = 64))]
    pub number: String,
    #[schemars(length(min = 1, max = 64))]
    pub carrier: Option<String>,
}
#[derive(Debug, Deserialize, Serialize, schemars::JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum OutboundCarrier {
    Dsv,
    Postnord,
    Dhl,
}
#[derive(Debug, Deserialize)]
pub struct FulfillmentMember {
    pub order_id: i64,
    pub listing_id: String,
}
#[derive(Debug, Deserialize)]
pub struct FulfillmentTerms {
    pub order_id: i64,
    pub group_id: Option<i64>,
    pub operation: FulfillmentOperation,
    pub eligible: bool,
    pub revision: String,
    pub members: Vec<FulfillmentMember>,
    pub has_tracking: bool,
    pub carrier: Option<OutboundCarrier>,
    pub funded_balance: Option<Money>,
    pub release_delay_hours: Option<i64>,
    pub handover_window_days: Option<i64>,
    pub closable_dispute_ids: Vec<i64>,
}
#[derive(Debug, Deserialize)]
pub struct FulfillmentReceipt {
    pub action_id: String,
    pub order_id: i64,
    pub operation: FulfillmentOperation,
    pub status: AccountActionState,
}

impl AccountApiClient {
    pub async fn fulfillment_terms(
        &self,
        access: &DelegatedAccess,
        order: i64,
        op: FulfillmentOperation,
    ) -> Result<FulfillmentTerms, ClientError> {
        const TOOL: &str = "get_my_fulfillment_terms";
        self.require_scope(access, FULFILLMENT_REVIEW_SCOPE, TOOL)?;
        if order <= 0 {
            return Err(invalid_input(TOOL, "Use a positive order ID"));
        }
        #[derive(Serialize)]
        struct Input {
            operation: FulfillmentOperation,
        }
        let request = self
            .http
            .post(self.endpoint(
                TOOL,
                &format!("mcp/account/orders/{order}/fulfillment-review"),
            )?)
            .header(AUTHORIZATION, access.credential.header(TOOL)?)
            .json(&Input { operation: op });
        let env: ApiEnvelope<FulfillmentTerms> = self.send(TOOL, request, 32768).await?;
        let q = env
            .data
            .filter(|_| env.success)
            .ok_or_else(|| invalid_response(TOOL))?;
        let members = &q.members;
        let delivery = op == FulfillmentOperation::ConfirmOrderDelivery;
        let meetup = op == FulfillmentOperation::ConfirmMeetupReceipt;
        let expected_delay = if delivery {
            Some(48)
        } else if meetup {
            Some(0)
        } else {
            None
        };
        if q.order_id != order
            || q.operation != op
            || !valid_revision(&q.revision)
            || q.group_id.is_some_and(|g| g <= 0)
            || members.is_empty()
            || members.len() > 25
            || (q.group_id.is_none() && members.len() != 1)
            || (!members.iter().any(|m| m.order_id == order))
            || members
                .iter()
                .any(|m| m.order_id <= 0 || !is_listing_id(&m.listing_id))
            || members.windows(2).any(|s| s[0].order_id >= s[1].order_id)
            || q.closable_dispute_ids.len() > 25
            || q.closable_dispute_ids.iter().any(|v| *v <= 0)
            || q.closable_dispute_ids.windows(2).any(|s| s[0] >= s[1])
            || (!meetup && !q.closable_dispute_ids.is_empty())
            || q.release_delay_hours != expected_delay
            || q.handover_window_days
                != (op == FulfillmentOperation::RecordMeetupHandover).then_some(7)
            || q.carrier.is_some() && op != FulfillmentOperation::BookOrderShipment
            || q.eligible && op == FulfillmentOperation::BookOrderShipment && q.carrier.is_none()
            || q.funded_balance.as_ref().is_some_and(|m| {
                !matches!(m.currency_code.as_str(), "SEK" | "EUR" | "NOK")
                    || m.amount.starts_with('-')
                    || !money_is_valid(m, &m.currency_code)
            })
            || q.eligible
                && (delivery || meetup)
                && !q
                    .funded_balance
                    .as_ref()
                    .is_some_and(|m| m.amount.bytes().any(|b| (b'1'..=b'9').contains(&b)))
        {
            return Err(invalid_response(TOOL));
        }
        Ok(q)
    }

    pub async fn change_fulfillment(
        &self,
        access: &DelegatedAccess,
        order: i64,
        op: FulfillmentOperation,
        revision: &str,
        tracking: Option<&OutboundTracking>,
        key: &str,
    ) -> Result<FulfillmentReceipt, ClientError> {
        let tool = op.write_tool();
        let valid = |s: &str| s == s.trim() && valid_action_text(s, 64);
        if order <= 0
            || !valid_revision(revision)
            || (op == FulfillmentOperation::SetOrderTracking) != tracking.is_some()
            || tracking.is_some_and(|t| {
                !valid(&t.number) || t.carrier.as_deref().is_some_and(|v| !valid(v))
            })
        {
            return Err(invalid_input(
                tool,
                "Use a current fulfillment revision and valid tracking information for the selected operation",
            ));
        }
        #[derive(Serialize)]
        struct Change<'a> {
            operation: FulfillmentOperation,
            expected_revision: &'a str,
            tracking: Option<&'a OutboundTracking>,
        }
        let req = self
            .action_request(
                tool,
                op.scope(),
                access,
                Method::POST,
                &format!("mcp/account/orders/{order}/fulfillment-actions"),
                key,
            )?
            .json(&Change {
                operation: op,
                expected_revision: revision,
                tracking,
            });
        self.direct_receipt(tool, req, |r: &FulfillmentReceipt| {
            r.order_id == order
                && r.operation == op
                && is_action_id(&r.action_id)
                && (op.scope() == BUYER_RECEIPTS_SCOPE
                    || r.status != AccountActionState::AwaitingConfirmation)
        })
        .await
    }
}
