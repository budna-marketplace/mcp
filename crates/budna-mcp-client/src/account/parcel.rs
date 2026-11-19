use super::{
    AccountActionState, AccountApiClient, ApiEnvelope, BidCurrency, ClientError, DelegatedAccess,
    PackageSize, invalid_input, invalid_response, is_action_id, shipping::valid_revision,
};
use crate::{Money, is_exact_decimal, is_listing_id, is_user_id, money_is_valid};
use budna_mcp_core::write_permissions::{
    PARCEL_MANAGEMENT_SCOPE, PARCEL_REQUESTS_SCOPE, PARCEL_REVIEW_SCOPE,
};
use reqwest::{Method, header::AUTHORIZATION};
use serde::{Deserialize, Serialize};
const ROOT: &str = "mcp/account/parcels";
#[derive(Debug, Deserialize, Serialize, schemars::JsonSchema)]
#[serde(tag = "kind", rename_all = "snake_case", deny_unknown_fields)]
pub enum ParcelSelection {
    ForOrder {
        #[schemars(range(min = 1))]
        order_id: i64,
    },
    Orders {
        #[schemars(length(min = 2, max = 25))]
        order_ids: Vec<i64>,
    },
    Parcel {
        #[schemars(range(min = 1))]
        group_id: i64,
    },
}
fn valid_ids(ids: &[i64]) -> bool {
    (2..=25).contains(&ids.len())
        && ids.iter().all(|id| *id > 0)
        && ids.windows(2).all(|w| w[0] < w[1])
}
#[derive(Debug, Deserialize, Serialize, schemars::JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct ParcelPrice {
    #[schemars(
        length(min = 1, max = 14),
        regex(pattern = r"^(0|[1-9][0-9]*)(\.[0-9]{1,2})?$")
    )]
    pub amount: String,
    pub currency_code: BidCurrency,
}
#[derive(Debug, Deserialize, Serialize, schemars::JsonSchema)]
#[serde(tag = "operation", rename_all = "snake_case", deny_unknown_fields)]
pub enum ParcelManagement {
    Quote {
        #[schemars(range(min = 1))]
        group_id: i64,
        package_size: PackageSize,
        #[schemars(range(min = 1, max = 200000))]
        package_weight_grams: i32,
        custom_shipping_price: Option<ParcelPrice>,
    },
    Decline {
        #[schemars(range(min = 1))]
        group_id: i64,
    },
    Dissolve {
        #[schemars(range(min = 1))]
        group_id: i64,
    },
}
#[derive(Debug, PartialEq, Deserialize, Serialize, schemars::JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum ParcelRole {
    Buyer,
    Seller,
}
#[derive(Debug, PartialEq, Deserialize, Serialize, schemars::JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum ParcelStatus {
    AwaitingSeller,
    Quoted,
    Funded,
    Dissolved,
}
#[derive(Debug, Deserialize)]
pub struct CombinedParcelTerms {
    pub revision: String,
    pub reviewed_at: Option<i64>,
    pub expires_at: Option<i64>,
    pub group_id: Option<i64>,
    pub role: ParcelRole,
    pub seller_id: String,
    pub status: ParcelStatus,
    pub members: Vec<CombinedParcelMember>,
    pub payment_deadline: i64,
    pub seller_answer_deadline: i64,
    pub package_size: Option<PackageSize>,
    pub package_weight_grams: Option<i32>,
    pub custom_shipping_price: Option<Money>,
    pub shipping_cost: Option<Money>,
    pub matched_seller_rule: bool,
    pub can_quote: bool,
    pub can_decline: bool,
    pub can_dissolve: bool,
    pub payment_available_on_budna: bool,
}
#[derive(Debug, Deserialize)]
pub struct CombinedParcelMember {
    pub order_id: i64,
    pub listing_id: String,
    pub quantity: i32,
    pub item_price: Money,
    pub buyer_protection_fee: Option<Money>,
    pub high_value_fee: Money,
    pub current_payment_deadline: Option<i64>,
    pub resulting_payment_deadline: Option<i64>,
}
#[derive(Debug, Deserialize)]
pub struct CombinedParcelReceipt {
    pub action_id: String,
    pub action: String,
    pub status: AccountActionState,
    pub group_id: Option<i64>,
}
impl AccountApiClient {
    pub async fn parcel_terms(
        &self,
        access: &DelegatedAccess,
        selection: &ParcelSelection,
    ) -> Result<CombinedParcelTerms, ClientError> {
        const OP: &str = "get_my_combined_parcel_terms";
        self.require_scope(access, PARCEL_REVIEW_SCOPE, OP)?;
        let valid = match selection {
            ParcelSelection::ForOrder { order_id } => *order_id > 0,
            ParcelSelection::Orders { order_ids } => valid_ids(order_ids),
            ParcelSelection::Parcel { group_id } => *group_id > 0,
        };
        if !valid {
            return Err(invalid_input(
                OP,
                "Select a parcel or 2 to 25 distinct orders in ascending order",
            ));
        }
        let request = self
            .http
            .post(self.endpoint(OP, &format!("{ROOT}/review"))?)
            .header(AUTHORIZATION, access.credential.header(OP)?)
            .json(selection);
        let envelope: ApiEnvelope<CombinedParcelTerms> = self.send(OP, request, 49152).await?;
        let q = envelope
            .data
            .filter(|_| envelope.success)
            .ok_or_else(|| invalid_response(OP))?;
        let ids: Vec<_> = q.members.iter().map(|m| m.order_id).collect();
        let ok = match selection {
            ParcelSelection::ForOrder { order_id } => {
                q.group_id.is_some_and(|g| g > 0)
                    && q.reviewed_at.is_none()
                    && q.expires_at.is_none()
                    && ids.contains(order_id)
            }
            ParcelSelection::Orders { order_ids } => {
                q.group_id.is_none()
                    && q.role == ParcelRole::Buyer
                    && q.reviewed_at
                        .zip(q.expires_at)
                        .is_some_and(|(a, b)| a > 0 && b > a)
                    && &ids == order_ids
            }
            ParcelSelection::Parcel { group_id } => {
                q.group_id == Some(*group_id) && q.reviewed_at.is_none() && q.expires_at.is_none()
            }
        };
        if !ok
            || !valid_revision(&q.revision)
            || !is_user_id(&q.seller_id)
            || q.payment_deadline <= 0
            || q.seller_answer_deadline <= 0
            || q.members.len() > 25
            || !ids.windows(2).all(|i| i[0] < i[1])
            || q.package_size.is_some() != q.package_weight_grams.is_some()
            || q.package_weight_grams.is_some_and(|w| w <= 0)
            || q.members.iter().any(|m| {
                m.order_id <= 0
                    || !is_listing_id(&m.listing_id)
                    || m.quantity <= 0
                    || !money_is_valid(&m.item_price, "SEK")
                    || !money_is_valid(&m.high_value_fee, "SEK")
                    || m.buyer_protection_fee
                        .as_ref()
                        .is_some_and(|s| !money_is_valid(s, "SEK"))
                    || m.current_payment_deadline.is_some_and(|t| t <= 0)
                    || m.resulting_payment_deadline.is_some_and(|t| t <= 0)
            })
            || q.custom_shipping_price
                .iter()
                .chain(q.shipping_cost.iter())
                .any(|s| !money_is_valid(s, "SEK"))
            || q.role == ParcelRole::Buyer && (q.can_quote || q.can_decline)
            || q.role == ParcelRole::Seller && (q.can_dissolve || q.payment_available_on_budna)
        {
            return Err(invalid_response(OP));
        }
        Ok(q)
    }
    pub async fn request_combined_parcel(
        &self,
        access: &DelegatedAccess,
        revision: &str,
        reviewed_at: i64,
        ids: &[i64],
        key: &str,
    ) -> Result<CombinedParcelReceipt, ClientError> {
        const OP: &str = "request_my_combined_parcel";
        if !valid_revision(revision) || !valid_ids(ids) || reviewed_at <= 0 {
            return Err(invalid_input(
                OP,
                "Invalid parcel review or order selection",
            ));
        }
        let request=self.action_request(OP,PARCEL_REQUESTS_SCOPE,access,Method::POST,&format!("{ROOT}/request"),key)?.json(&serde_json::json!({"operation":"request","expected_revision":revision,"reviewed_at":reviewed_at,"order_ids":ids}));
        self.direct_receipt(OP, request, |r: &CombinedParcelReceipt| {
            is_action_id(&r.action_id)
                && r.action == "request_combined_parcel"
                && r.group_id.is_none_or(|g| g > 0)
                && (r.status != AccountActionState::Executed || r.group_id.is_some())
        })
        .await
    }
    pub async fn manage_combined_parcel(
        &self,
        access: &DelegatedAccess,
        revision: &str,
        change: &ParcelManagement,
        key: &str,
    ) -> Result<CombinedParcelReceipt, ClientError> {
        const OP: &str = "change_my_combined_parcel";
        let (id, name) = match change {
            ParcelManagement::Quote {
                group_id,
                package_weight_grams,
                custom_shipping_price,
                ..
            } if (1..=200000).contains(package_weight_grams)
                && custom_shipping_price.as_ref().is_none_or(|p| {
                    p.amount.len() <= 14
                        && is_exact_decimal(&p.amount)
                        && !p.amount.starts_with('-')
                        && p.amount.split_once('.').is_none_or(|(_, f)| f.len() <= 2)
                        && (!p.amount.starts_with('0')
                            || p.amount == "0"
                            || p.amount.starts_with("0."))
                }) =>
            {
                (*group_id, "quote_combined_parcel")
            }
            ParcelManagement::Decline { group_id } => (*group_id, "decline_combined_parcel"),
            ParcelManagement::Dissolve { group_id } => (*group_id, "dissolve_combined_parcel"),
            _ => return Err(invalid_input(OP, "Invalid parcel quote")),
        };
        if id <= 0 || !valid_revision(revision) {
            return Err(invalid_input(OP, "Invalid parcel review"));
        }
        #[derive(Serialize)]
        struct Body<'a> {
            expected_revision: &'a str,
            #[serde(flatten)]
            change: &'a ParcelManagement,
        }
        let req = self
            .action_request(
                OP,
                PARCEL_MANAGEMENT_SCOPE,
                access,
                Method::POST,
                &format!("{ROOT}/manage"),
                key,
            )?
            .json(&Body {
                expected_revision: revision,
                change,
            });
        self.direct_receipt(OP, req, |r: &CombinedParcelReceipt| {
            is_action_id(&r.action_id)
                && r.action == name
                && r.group_id == Some(id)
                && matches!(
                    r.status,
                    AccountActionState::Executed
                        | AccountActionState::Executing
                        | AccountActionState::Unknown
                )
        })
        .await
    }
}
