use super::{ProjectionBudget, UNTRUSTED_CONTENT_NOTICE, account::WatchlistActionStatusOutput};
use budna_mcp_client::{PackageSize, ShippingListingPage, ShippingReceipt, ShippingRuleCatalog};
use rmcp::schemars;
use serde::Serialize;

#[derive(Debug, Serialize, schemars::JsonSchema)]
pub struct ShippingRulesOutput {
    revision: String,
    kinds: Vec<KindOutput>,
    parcels: Vec<ParcelOutput>,
    untrusted_content_notice: &'static str,
}
#[derive(Debug, Serialize, schemars::JsonSchema)]
struct KindOutput {
    id: i64,
    name: String,
    listing_count: u16,
}
#[derive(Debug, Serialize, schemars::JsonSchema)]
struct ParcelOutput {
    id: i64,
    name: String,
    package_size: PackageSize,
    kinds: Vec<CapacityOutput>,
}
#[derive(Debug, Serialize, schemars::JsonSchema)]
struct CapacityOutput {
    kind_id: i64,
    quantity: u16,
}
impl From<ShippingRuleCatalog> for ShippingRulesOutput {
    fn from(s: ShippingRuleCatalog) -> Self {
        let mut budget = ProjectionBudget::default();
        Self {
            revision: s.revision,
            kinds: s
                .kinds
                .into_iter()
                .map(|k| KindOutput {
                    id: k.id,
                    name: budget.text(k.name, 320),
                    listing_count: k.listing_count,
                })
                .collect(),
            parcels: s
                .parcels
                .into_iter()
                .map(|p| ParcelOutput {
                    id: p.id,
                    name: budget.text(p.name, 320),
                    package_size: p.package_size,
                    kinds: p
                        .kinds
                        .into_iter()
                        .map(|c| CapacityOutput {
                            kind_id: c.kind_id,
                            quantity: c.quantity,
                        })
                        .collect(),
                })
                .collect(),
            untrusted_content_notice: UNTRUSTED_CONTENT_NOTICE,
        }
    }
}

#[derive(Debug, Serialize, schemars::JsonSchema)]
pub struct ShippingListingsOutput {
    revision: String,
    listings: Vec<ShippingListingOutput>,
    next_after: Option<String>,
    untrusted_content_notice: &'static str,
}
#[derive(Debug, Serialize, schemars::JsonSchema)]
struct ShippingListingOutput {
    listing_id: String,
    title: Option<String>,
    kind_id: Option<i64>,
    package_size: Option<PackageSize>,
    weight_grams: Option<i64>,
    can_assign: bool,
}
impl From<ShippingListingPage> for ShippingListingsOutput {
    fn from(p: ShippingListingPage) -> Self {
        let mut budget = ProjectionBudget::default();
        Self {
            revision: p.revision,
            listings: p
                .listings
                .into_iter()
                .map(|l| ShippingListingOutput {
                    listing_id: l.listing_id,
                    title: budget.optional_text(l.title, 512),
                    kind_id: l.kind_id,
                    package_size: l.package_size,
                    weight_grams: l.weight_grams,
                    can_assign: l.can_assign,
                })
                .collect(),
            next_after: p.next_after,
            untrusted_content_notice: UNTRUSTED_CONTENT_NOTICE,
        }
    }
}
#[derive(Debug, Serialize, schemars::JsonSchema)]
pub struct ShippingReceiptOutput {
    action_id: String,
    operation: String,
    status: WatchlistActionStatusOutput,
    kind_id: Option<i64>,
    parcel_id: Option<i64>,
}
impl From<ShippingReceipt> for ShippingReceiptOutput {
    fn from(r: ShippingReceipt) -> Self {
        Self {
            action_id: r.action_id,
            operation: r.operation,
            status: r.status.into(),
            kind_id: r.kind_id,
            parcel_id: r.parcel_id,
        }
    }
}
