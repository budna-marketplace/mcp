use super::{
    AccountApiClient, AccountWatchlistActionStatus, ClientError, DelegatedAccess,
    SHIPPING_DEFAULTS_SCOPE, SHIPPING_RULES_READ_SCOPE, SHIPPING_RULES_WRITE_SCOPE,
    actions::valid_action_text, invalid_input, invalid_response, is_action_id,
};
use crate::is_listing_id;
use reqwest::Method;
use serde::{Deserialize, Serialize};
use std::collections::HashSet;

#[derive(Clone, Copy, Debug, Deserialize, Serialize, schemars::JsonSchema, Eq, PartialEq, Hash)]
#[serde(rename_all = "snake_case")]
pub enum ShippingCarrier {
    Dsv,
    Postnord,
    Dhl,
}

#[derive(Clone, Copy, Debug, Deserialize, Serialize, schemars::JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum PackageSize {
    Small,
    Medium,
    Large,
    ExtraLarge,
}

#[derive(Debug, Deserialize, Serialize, schemars::JsonSchema)]
#[serde(tag = "operation", rename_all = "snake_case", deny_unknown_fields)]
pub enum ShippingDefaultEdit {
    SetCarriers {
        #[schemars(length(min = 1, max = 3))]
        provider_codes: Vec<ShippingCarrier>,
    },
    ResetCarriers {},
    SetCombined {
        enabled: bool,
    },
}
#[derive(Debug, Deserialize, Serialize, schemars::JsonSchema)]
#[serde(tag = "operation", rename_all = "snake_case", deny_unknown_fields)]
pub enum ShippingRuleEdit {
    CreateKind {
        name: String,
        listing_id: String,
    },
    RenameKind {
        kind_id: i64,
        name: String,
    },
    AssignListing {
        kind_id: i64,
        listing_id: String,
    },
    RemoveListing {
        kind_id: i64,
        listing_id: String,
    },
    DeleteKind {
        kind_id: i64,
    },
    CreateParcel {
        name: String,
        package_size: PackageSize,
        kind_id: i64,
        quantity: u16,
    },
    RenameParcel {
        parcel_id: i64,
        name: String,
    },
    SetParcelSize {
        parcel_id: i64,
        package_size: PackageSize,
    },
    SetParcelKind {
        parcel_id: i64,
        kind_id: i64,
        quantity: u16,
    },
    RemoveParcelKind {
        parcel_id: i64,
        kind_id: i64,
    },
    DeleteParcel {
        parcel_id: i64,
    },
}

pub(super) fn valid_revision(s: &str) -> bool {
    s.len() == 43
        && s.bytes()
            .all(|b| b.is_ascii_alphanumeric() || matches!(b, b'_' | b'-'))
}
fn valid_parcel_name(s: &str) -> bool {
    s.is_empty() || valid_action_text(s, 80)
}

impl ShippingDefaultEdit {
    fn operation(&self) -> Option<&'static str> {
        match self {
            Self::SetCarriers { provider_codes }
                if (1..=3).contains(&provider_codes.len())
                    && provider_codes.iter().collect::<HashSet<_>>().len()
                        == provider_codes.len() =>
            {
                Some("set_carriers")
            }
            Self::SetCarriers { .. } => None,
            Self::ResetCarriers {} => Some("reset_carriers"),
            Self::SetCombined { .. } => Some("set_combined"),
        }
    }
}

impl ShippingRuleEdit {
    fn matches_receipt(&self, r: &ShippingReceipt) -> bool {
        use ShippingRuleEdit::*;
        match self {
            CreateKind { .. } => r.parcel_id.is_none(),
            CreateParcel { .. } => r.kind_id.is_none(),
            RenameKind { kind_id, .. }
            | AssignListing { kind_id, .. }
            | RemoveListing { kind_id, .. }
            | DeleteKind { kind_id } => {
                r.parcel_id.is_none()
                    && r.kind_id.is_none_or(|id| id == *kind_id)
                    && (r.status != AccountWatchlistActionStatus::Executed || r.kind_id.is_some())
            }
            RenameParcel { parcel_id, .. }
            | SetParcelSize { parcel_id, .. }
            | SetParcelKind { parcel_id, .. }
            | RemoveParcelKind { parcel_id, .. }
            | DeleteParcel { parcel_id } => {
                r.kind_id.is_none()
                    && r.parcel_id.is_none_or(|id| id == *parcel_id)
                    && (r.status != AccountWatchlistActionStatus::Executed || r.parcel_id.is_some())
            }
        }
    }
    fn operation(&self) -> Option<&'static str> {
        match self {
            Self::CreateKind { name, listing_id }
                if valid_action_text(name, 80) && is_listing_id(listing_id) =>
            {
                Some("create_kind")
            }
            Self::RenameKind { kind_id, name } if *kind_id > 0 && valid_action_text(name, 80) => {
                Some("rename_kind")
            }
            Self::AssignListing {
                kind_id,
                listing_id,
            } if *kind_id > 0 && is_listing_id(listing_id) => Some("assign_listing"),
            Self::RemoveListing {
                kind_id,
                listing_id,
            } if *kind_id > 0 && is_listing_id(listing_id) => Some("remove_listing"),
            Self::DeleteKind { kind_id } if *kind_id > 0 => Some("delete_kind"),
            Self::CreateParcel {
                name,
                kind_id,
                quantity,
                ..
            } if valid_parcel_name(name) && *kind_id > 0 && (1..=999).contains(quantity) => {
                Some("create_parcel")
            }
            Self::RenameParcel { parcel_id, name } if *parcel_id > 0 && valid_parcel_name(name) => {
                Some("rename_parcel")
            }
            Self::SetParcelSize { parcel_id, .. } if *parcel_id > 0 => Some("set_parcel_size"),
            Self::SetParcelKind {
                parcel_id,
                kind_id,
                quantity,
            } if *parcel_id > 0 && *kind_id > 0 && (1..=999).contains(quantity) => {
                Some("set_parcel_kind")
            }
            Self::RemoveParcelKind { parcel_id, kind_id } if *parcel_id > 0 && *kind_id > 0 => {
                Some("remove_parcel_kind")
            }
            Self::DeleteParcel { parcel_id } if *parcel_id > 0 => Some("delete_parcel"),
            _ => None,
        }
    }
}

#[derive(Debug, Deserialize)]
pub struct ShippingRuleCatalog {
    pub revision: String,
    pub kinds: Vec<ShippingKind>,
    pub parcels: Vec<ShippingParcel>,
}
#[derive(Debug, Deserialize)]
pub struct ShippingKind {
    pub id: i64,
    pub name: String,
    pub listing_count: u16,
}
#[derive(Debug, Deserialize)]
pub struct ShippingParcel {
    pub id: i64,
    pub name: String,
    pub package_size: PackageSize,
    pub kinds: Vec<ShippingCapacity>,
}
#[derive(Debug, Deserialize)]
pub struct ShippingCapacity {
    pub kind_id: i64,
    pub quantity: u16,
}
#[derive(Debug, Deserialize)]
pub struct ShippingListingPage {
    pub revision: String,
    pub listings: Vec<ShippingListing>,
    #[serde(deserialize_with = "Option::deserialize")]
    pub next_after: Option<String>,
}
#[derive(Debug, Deserialize)]
pub struct ShippingListing {
    pub listing_id: String,
    #[serde(deserialize_with = "Option::deserialize")]
    pub title: Option<String>,
    #[serde(deserialize_with = "Option::deserialize")]
    pub kind_id: Option<i64>,
    #[serde(deserialize_with = "Option::deserialize")]
    pub package_size: Option<PackageSize>,
    #[serde(deserialize_with = "Option::deserialize")]
    pub weight_grams: Option<i64>,
    pub can_assign: bool,
}
#[derive(Debug, Deserialize)]
pub struct ShippingReceipt {
    pub action_id: String,
    pub operation: String,
    pub status: AccountWatchlistActionStatus,
    #[serde(deserialize_with = "Option::deserialize")]
    pub kind_id: Option<i64>,
    #[serde(deserialize_with = "Option::deserialize")]
    pub parcel_id: Option<i64>,
}

impl ShippingRuleCatalog {
    fn valid(&self) -> bool {
        if !valid_revision(&self.revision) || self.kinds.len() > 50 || self.parcels.len() > 50 {
            return false;
        }
        let mut kinds = HashSet::new();
        let mut parcels = HashSet::new();
        self.kinds.iter().all(|k| {
            k.id > 0 && kinds.insert(k.id) && k.listing_count <= 200 && k.name.chars().count() <= 80
        }) && self.parcels.iter().all(|p| {
            let mut capacities = HashSet::new();
            p.id > 0
                && parcels.insert(p.id)
                && p.name.chars().count() <= 80
                && (1..=50).contains(&p.kinds.len())
                && p.kinds.iter().all(|c| {
                    kinds.contains(&c.kind_id)
                        && capacities.insert(c.kind_id)
                        && (1..=999).contains(&c.quantity)
                })
        })
    }
}
impl ShippingReceipt {
    fn matches(&self, operation: &str) -> bool {
        is_action_id(&self.action_id)
            && self.operation == operation
            && self.kind_id.is_none_or(|id| id > 0)
            && self.parcel_id.is_none_or(|id| id > 0)
            && (self.status != AccountWatchlistActionStatus::Executed
                || match operation {
                    "create_kind" => self.kind_id.is_some(),
                    "create_parcel" => self.parcel_id.is_some(),
                    _ => true,
                })
    }
}

#[derive(Serialize)]
struct RevisionChange<'a, T> {
    expected_revision: &'a str,
    #[serde(flatten)]
    change: &'a T,
}

impl AccountApiClient {
    pub async fn shipping_rules(
        &self,
        access: &DelegatedAccess,
    ) -> Result<ShippingRuleCatalog, ClientError> {
        const OP: &str = "get_my_shipping_rules";
        self.require_scope(access, SHIPPING_RULES_READ_SCOPE, OP)?;
        let result: ShippingRuleCatalog = self
            .get(OP, "mcp/account/selling/shipping-rules", access, &[])
            .await?;
        if !result.valid() {
            return Err(invalid_response(OP));
        }
        Ok(result)
    }

    pub async fn shipping_rule_listings(
        &self,
        access: &DelegatedAccess,
        kind: Option<i64>,
        after: Option<&str>,
        limit: u32,
    ) -> Result<ShippingListingPage, ClientError> {
        const OP: &str = "list_my_shipping_rule_listings";
        self.require_scope(access, SHIPPING_RULES_READ_SCOPE, OP)?;
        if !(1..=50).contains(&limit)
            || kind.is_some_and(|k| k < 1)
            || after.is_some_and(|id| !is_listing_id(id))
        {
            return Err(invalid_input(
                OP,
                "Invalid shipping kind, listing cursor or limit",
            ));
        }
        let mut query = vec![("limit", limit.to_string())];
        if let Some(k) = kind {
            query.push(("kind_id", k.to_string()));
        }
        if let Some(id) = after {
            query.push(("after", id.to_owned()));
        }
        let result: ShippingListingPage = self
            .get(
                OP,
                "mcp/account/selling/shipping-rules/listings",
                access,
                &query,
            )
            .await?;
        let mut listings = HashSet::new();
        if !valid_revision(&result.revision)
            || result.listings.len() > limit as usize
            || result.next_after.as_deref().is_some_and(|id| {
                !is_listing_id(id)
                    || result.listings.last().is_none_or(|l| l.listing_id != id)
                    || Some(id) == after
            })
            || !result.listings.iter().all(|l| {
                is_listing_id(&l.listing_id)
                    && listings.insert(&l.listing_id)
                    && l.kind_id.is_none_or(|k| k > 0)
                    && kind.is_none_or(|k| l.kind_id == Some(k))
                    && l.title.as_ref().is_none_or(|s| s.chars().count() <= 512)
                    && (!l.can_assign
                        || (l.package_size.is_some() && l.weight_grams.is_some_and(|w| w > 0)))
            })
        {
            return Err(invalid_response(OP));
        }
        Ok(result)
    }

    pub async fn change_shipping_defaults(
        &self,
        access: &DelegatedAccess,
        revision: &str,
        change: &ShippingDefaultEdit,
        key: &str,
    ) -> Result<ShippingReceipt, ClientError> {
        const OP: &str = "change_my_shipping_defaults";
        let operation = change
            .operation()
            .filter(|_| valid_revision(revision))
            .ok_or_else(|| invalid_input(OP, "Invalid revision or carrier selection"))?;
        let request = self
            .action_request(
                OP,
                SHIPPING_DEFAULTS_SCOPE,
                access,
                Method::POST,
                "mcp/account/selling/shipping-settings",
                key,
            )?
            .json(&RevisionChange {
                expected_revision: revision,
                change,
            });
        self.direct_receipt(OP, request, |r: &ShippingReceipt| {
            r.matches(operation) && r.kind_id.is_none() && r.parcel_id.is_none()
        })
        .await
    }
    pub async fn change_shipping_rule(
        &self,
        access: &DelegatedAccess,
        revision: &str,
        change: &ShippingRuleEdit,
        key: &str,
    ) -> Result<ShippingReceipt, ClientError> {
        const OP: &str = "change_my_shipping_rule";
        let operation = change
            .operation()
            .filter(|_| valid_revision(revision))
            .ok_or_else(|| invalid_input(OP, "Invalid revision or shipping rule edit"))?;
        let request = self
            .action_request(
                OP,
                SHIPPING_RULES_WRITE_SCOPE,
                access,
                Method::POST,
                "mcp/account/selling/shipping-rules",
                key,
            )?
            .json(&RevisionChange {
                expected_revision: revision,
                change,
            });
        self.direct_receipt(OP, request, |r: &ShippingReceipt| {
            r.matches(operation) && change.matches_receipt(r)
        })
        .await
    }
}
