use super::{
    AccountActionState, AccountApiClient, ApiEnvelope, BIDS_READ_SCOPE, BIDS_WRITE_SCOPE,
    ClientError, DelegatedAccess, invalid_input, invalid_response, is_action_id,
};
use crate::{
    Money, decimal_is_less_than_or_equal, is_exact_decimal, is_listing_id, is_live_stream_id,
    money_is_valid,
};
use reqwest::{Method, header::AUTHORIZATION};
use serde::{Deserialize, Serialize};

#[derive(Clone, Copy, Debug, Deserialize, Serialize, schemars::JsonSchema)]
pub enum BidCurrency {
    #[serde(rename = "SEK")]
    Sek,
}

#[derive(Debug, Deserialize, Serialize, schemars::JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct BidAmount {
    #[schemars(
        length(min = 1, max = 10),
        regex(pattern = r"^(0|[1-9][0-9]{0,6})(\.[0-9]{1,2})?$")
    )]
    pub amount: String,
    pub currency_code: BidCurrency,
}

impl BidAmount {
    fn valid(&self) -> bool {
        let value = self.amount.as_str();
        let (whole, fraction) = value.split_once('.').unwrap_or((value, ""));
        is_exact_decimal(value)
            && (whole == "0" || !whole.starts_with('0'))
            && fraction.len() <= 2
            && !decimal_is_less_than_or_equal(value, "0")
            && decimal_is_less_than_or_equal(value, "1000000")
    }

    fn matches(&self, money: &Money) -> bool {
        money_is_valid(money, "SEK")
            && decimal_is_less_than_or_equal(&self.amount, &money.amount)
            && decimal_is_less_than_or_equal(&money.amount, &self.amount)
    }
}

#[derive(Debug, Deserialize)]
pub struct BidQuote {
    pub quote_id: String,
    pub listing_id: String,
    #[serde(deserialize_with = "Option::deserialize")]
    pub live_session_id: Option<String>,
    pub round_ref: String,
    pub bid: Money,
    #[serde(deserialize_with = "Option::deserialize")]
    pub current_bid: Option<Money>,
    pub minimum_bid: Money,
    pub increment: Money,
    pub buyer_protection_fee: Money,
    pub buyer_protection_mandatory: bool,
    pub unpaid_fee: Money,
    pub payment_due_hours: u32,
    pub payment_grace_hours: u32,
    pub delivery_at_checkout: bool,
    pub ends_at: i64,
    pub expires_at: i64,
}

#[derive(Debug, Deserialize)]
pub struct BidSubmissionReceipt {
    pub action_id: String,
    pub status: AccountActionState,
    pub listing_id: String,
    pub bid: Money,
}

impl AccountApiClient {
    pub async fn prepare_bid(
        &self,
        access: &DelegatedAccess,
        listing: &str,
        bid: &BidAmount,
    ) -> Result<BidQuote, ClientError> {
        const OP: &str = "prepare_my_bid";
        self.require_scope(access, BIDS_READ_SCOPE, OP)?;
        if !self.write_permissions().bids || !is_listing_id(listing) || !bid.valid() {
            return Err(invalid_input(
                OP,
                "Invalid bid, listing or unavailable bidding",
            ));
        }
        let request = self
            .http
            .post(self.endpoint(OP, &format!("mcp/account/listings/{listing}/bid-quote"))?)
            .header(AUTHORIZATION, access.credential.header(OP)?)
            .json(&serde_json::json!({"bid":bid}));
        let result: ApiEnvelope<BidQuote> = self.send(OP, request, 8192).await?;
        result
            .data
            .filter(|q| {
                result.success
                    && is_action_id(&q.quote_id)
                    && q.listing_id == listing
                    && q.live_session_id.as_deref().is_none_or(is_live_stream_id)
                    && q.round_ref.strip_prefix("round_").is_some_and(|s| {
                        s.len() == 43
                            && s.bytes()
                                .all(|b| b.is_ascii_alphanumeric() || matches!(b, b'-' | b'_'))
                    })
                    && bid.matches(&q.bid)
                    && q.current_bid
                        .as_ref()
                        .is_none_or(|m| money_is_valid(m, "SEK"))
                    && [
                        &q.minimum_bid,
                        &q.increment,
                        &q.buyer_protection_fee,
                        &q.unpaid_fee,
                    ]
                    .into_iter()
                    .all(|m| money_is_valid(m, "SEK"))
                    && q.payment_due_hours > 0
                    && q.payment_grace_hours == 0
                    && q.delivery_at_checkout
                    && q.ends_at > 0
                    && q.expires_at > 0
            })
            .ok_or_else(|| invalid_response(OP))
    }

    pub async fn submit_bid(
        &self,
        access: &DelegatedAccess,
        listing: &str,
        quote_id: &str,
        bid: &BidAmount,
    ) -> Result<BidSubmissionReceipt, ClientError> {
        const OP: &str = "submit_my_quoted_bid";
        if !is_listing_id(listing) || !is_action_id(quote_id) || !bid.valid() {
            return Err(invalid_input(OP, "Invalid listing, quote or bid"));
        }
        // A lost response must reuse the same action even across a fresh MCP connection.
        let request = self
            .action_request(
                OP,
                BIDS_WRITE_SCOPE,
                access,
                Method::POST,
                &format!("mcp/account/listings/{listing}/bids"),
                quote_id,
            )?
            .json(&serde_json::json!({"quote_id":quote_id,"bid":bid}));
        self.direct_receipt(OP, request, |r: &BidSubmissionReceipt| {
            is_action_id(&r.action_id) && r.listing_id == listing && bid.matches(&r.bid)
        })
        .await
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn exact_bids_observe_the_product_ceiling_and_fixed_scale() {
        for (amount, valid) in [
            ("0.01", true),
            ("10.10", true),
            ("1000000.00", true),
            ("0.00", false),
            ("1000000.01", false),
            ("2.001", false),
            ("05", false),
            ("-1", false),
            ("1e2", false),
        ] {
            assert_eq!(
                BidAmount {
                    amount: amount.into(),
                    currency_code: BidCurrency::Sek
                }
                .valid(),
                valid,
                "{amount}"
            );
        }
    }
}
