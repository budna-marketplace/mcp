use budna_mcp_client::{
    AccountAutoBid, AccountBid, AccountDebt, AccountMoneySummary, AccountNotification,
    AccountOffer, AccountOrder, AccountOrderDetail, AccountPage, AccountPaymentActivity,
    AccountPayouts, AccountRating, AccountSavedSearch, AccountSecondChanceOffer,
    AccountWatchedListing, AccountWatchlist,
};
use rmcp::schemars;
use serde::Serialize;

use super::{
    MAX_CODE_BYTES, MAX_DISPLAY_TEXT_BYTES, MAX_LONG_TEXT_BYTES, MAX_NAME_BYTES, ProjectionBudget,
    UNTRUSTED_CONTENT_NOTICE,
    common::{MoneyOutput, PaginationOutput},
};

const MAX_PAGE_ITEMS: usize = 50;
const MAX_SEARCH_FILTERS: usize = 20;

#[derive(Debug, Serialize, schemars::JsonSchema)]
pub struct AccountPageOutput<T> {
    #[schemars(length(max = 50))]
    pub items: Vec<T>,
    pub pagination: PaginationOutput,
    pub truncated: bool,
    pub untrusted_content_notice: &'static str,
}

fn project<T, O>(
    page: AccountPage<T>,
    mut item: impl FnMut(T, &mut ProjectionBudget) -> O,
) -> AccountPageOutput<O> {
    let mut budget = ProjectionBudget::default();
    let items = budget
        .objects(page.items, MAX_PAGE_ITEMS)
        .into_iter()
        .map(|value| item(value, &mut budget))
        .collect();
    AccountPageOutput {
        items,
        pagination: page.pagination.into(),
        truncated: budget.truncated(),
        untrusted_content_notice: UNTRUSTED_CONTENT_NOTICE,
    }
}

#[derive(Debug, Serialize, schemars::JsonSchema)]
pub struct AccountBidOutput {
    #[schemars(length(max = 64))]
    pub listing_id: String,
    #[schemars(length(max = 512))]
    pub listing_title: Option<String>,
    pub amount: MoneyOutput,
    pub placed_at: i64,
    #[schemars(length(max = 64))]
    pub status: String,
}

impl From<AccountPage<AccountBid>> for AccountPageOutput<AccountBidOutput> {
    fn from(page: AccountPage<AccountBid>) -> Self {
        project(page, |bid, budget| AccountBidOutput {
            listing_id: bid.listing_id,
            listing_title: budget.optional_text(bid.listing_title, MAX_DISPLAY_TEXT_BYTES),
            amount: bid.amount.into(),
            placed_at: bid.placed_at,
            status: bid.status,
        })
    }
}

#[derive(Debug, Serialize, schemars::JsonSchema)]
pub struct AccountOfferOutput {
    pub id: i64,
    #[schemars(length(max = 64))]
    pub listing_id: String,
    #[schemars(length(max = 512))]
    pub listing_title: Option<String>,
    #[schemars(length(max = 16))]
    pub role: String,
    pub amount: MoneyOutput,
    pub quantity: i32,
    pub counter_amount: Option<MoneyOutput>,
    #[schemars(length(max = 64))]
    pub status: String,
    #[schemars(length(max = 16))]
    pub last_action_by: Option<String>,
    pub amount_withdrawn_at: Option<i64>,
    pub expires_at: i64,
    pub created_at: i64,
    pub updated_at: i64,
}

impl From<AccountPage<AccountOffer>> for AccountPageOutput<AccountOfferOutput> {
    fn from(page: AccountPage<AccountOffer>) -> Self {
        project(page, |offer, budget| AccountOfferOutput {
            id: offer.id,
            listing_id: offer.listing_id,
            listing_title: budget.optional_text(offer.listing_title, MAX_DISPLAY_TEXT_BYTES),
            role: offer.role,
            amount: offer.amount.into(),
            quantity: offer.quantity,
            counter_amount: offer.counter_amount.map(Into::into),
            status: offer.status,
            last_action_by: offer.last_action_by,
            amount_withdrawn_at: offer.amount_withdrawn_at,
            expires_at: offer.expires_at,
            created_at: offer.created_at,
            updated_at: offer.updated_at,
        })
    }
}

#[derive(Debug, Serialize, schemars::JsonSchema)]
pub struct AccountWatchlistOutput {
    pub id: i64,
    #[schemars(length(max = 256))]
    pub name: String,
    pub is_default: bool,
    pub item_count: i64,
    pub created_at: i64,
    pub updated_at: i64,
}

impl From<AccountPage<AccountWatchlist>> for AccountPageOutput<AccountWatchlistOutput> {
    fn from(page: AccountPage<AccountWatchlist>) -> Self {
        project(page, |watchlist, budget| AccountWatchlistOutput {
            id: watchlist.id,
            name: budget.text(watchlist.name, MAX_NAME_BYTES),
            is_default: watchlist.is_default,
            item_count: watchlist.item_count,
            created_at: watchlist.created_at,
            updated_at: watchlist.updated_at,
        })
    }
}

#[derive(Debug, Serialize, schemars::JsonSchema)]
pub struct AccountWatchedListingOutput {
    #[schemars(length(max = 64))]
    pub listing_id: String,
    #[schemars(length(max = 512))]
    pub title: Option<String>,
}

impl From<AccountPage<AccountWatchedListing>> for AccountPageOutput<AccountWatchedListingOutput> {
    fn from(page: AccountPage<AccountWatchedListing>) -> Self {
        project(page, |listing, budget| AccountWatchedListingOutput {
            listing_id: listing.listing_id,
            title: budget.optional_text(listing.title, MAX_DISPLAY_TEXT_BYTES),
        })
    }
}

#[derive(Debug, Serialize, schemars::JsonSchema)]
pub struct AccountSavedSearchOutput {
    pub id: i64,
    #[schemars(length(max = 256))]
    pub name: String,
    pub query: AccountSavedSearchQueryOutput,
    #[schemars(length(max = 64))]
    pub notification_frequency: String,
    pub is_active: bool,
    pub unseen_count: i64,
    pub total_matches: i64,
    pub last_viewed_at: Option<i64>,
    pub created_at: i64,
    pub updated_at: i64,
}

#[derive(Debug, Serialize, schemars::JsonSchema)]
pub struct AccountSavedSearchQueryOutput {
    #[schemars(length(max = 512))]
    pub text: Option<String>,
    pub category_id: Option<i32>,
    #[schemars(length(max = 64))]
    pub market: Option<String>,
    pub min_price: Option<MoneyOutput>,
    pub max_price: Option<MoneyOutput>,
    #[schemars(length(max = 64))]
    pub condition: Option<String>,
    #[schemars(length(max = 64))]
    pub listing_type: Option<String>,
    #[schemars(length(max = 256))]
    pub location_region: Option<String>,
    #[schemars(length(max = 256))]
    pub location_municipality: Option<String>,
    pub allow_pickup: Option<bool>,
    #[schemars(length(max = 20))]
    pub filters: Vec<AccountSearchFilterOutput>,
}

#[derive(Debug, Serialize, schemars::JsonSchema)]
pub struct AccountSearchFilterOutput {
    #[schemars(length(max = 128))]
    pub name: String,
    #[schemars(length(max = 512))]
    pub value: String,
}

impl From<AccountPage<AccountSavedSearch>> for AccountPageOutput<AccountSavedSearchOutput> {
    fn from(page: AccountPage<AccountSavedSearch>) -> Self {
        project(page, |search, budget| {
            let query = search.query;
            let filters = budget
                .objects(query.filters.into_iter().collect(), MAX_SEARCH_FILTERS)
                .into_iter()
                .map(|(name, value)| AccountSearchFilterOutput {
                    name: budget.text(name, MAX_CODE_BYTES),
                    value: budget.text(value, MAX_DISPLAY_TEXT_BYTES),
                })
                .collect();
            AccountSavedSearchOutput {
                id: search.id,
                name: budget.text(search.name, MAX_NAME_BYTES),
                query: AccountSavedSearchQueryOutput {
                    text: budget.optional_text(query.text, MAX_DISPLAY_TEXT_BYTES),
                    category_id: query.category_id,
                    market: budget.optional_text(query.market, 64),
                    min_price: query.min_price.map(Into::into),
                    max_price: query.max_price.map(Into::into),
                    condition: budget.optional_text(query.condition, 64),
                    listing_type: budget.optional_text(query.listing_type, 64),
                    location_region: budget.optional_text(query.location_region, MAX_NAME_BYTES),
                    location_municipality: budget
                        .optional_text(query.location_municipality, MAX_NAME_BYTES),
                    allow_pickup: query.allow_pickup,
                    filters,
                },
                notification_frequency: budget.text(search.notification_frequency, 64),
                is_active: search.is_active,
                unseen_count: search.unseen_count,
                total_matches: search.total_matches,
                last_viewed_at: search.last_viewed_at,
                created_at: search.created_at,
                updated_at: search.updated_at,
            }
        })
    }
}

#[derive(Debug, Serialize, schemars::JsonSchema)]
pub struct AccountOrderOutput {
    pub id: i64,
    #[schemars(length(max = 16))]
    pub role: String,
    #[schemars(length(max = 64))]
    pub listing_id: String,
    #[schemars(length(max = 512))]
    pub listing_title: Option<String>,
    pub amount: MoneyOutput,
    pub quantity: i32,
    #[schemars(length(max = 64))]
    pub status: String,
    #[schemars(length(max = 64))]
    pub payment_status: String,
    #[schemars(length(max = 64))]
    pub fulfillment_mode: String,
    #[schemars(length(max = 64))]
    pub fulfillment_state: String,
    pub is_auction_order: bool,
    pub tracking: AccountOrderTrackingOutput,
    #[schemars(length(max = 64))]
    pub withdrawal_status: Option<String>,
    pub payout: Option<MoneyOutput>,
    pub payment_due_at: Option<i64>,
    pub paid_at: Option<i64>,
    pub ship_by: Option<i64>,
    pub created_at: i64,
    pub updated_at: i64,
}

#[derive(Debug, Serialize, schemars::JsonSchema)]
pub struct AccountOrderTrackingOutput {
    pub available: bool,
    #[schemars(length(max = 64))]
    pub carrier: Option<String>,
    #[schemars(length(max = 128))]
    pub tracking_number: Option<String>,
}

fn project_order(order: AccountOrder, budget: &mut ProjectionBudget) -> AccountOrderOutput {
    AccountOrderOutput {
        id: order.id,
        role: order.role,
        listing_id: order.listing_id,
        listing_title: budget.optional_text(order.listing_title, MAX_DISPLAY_TEXT_BYTES),
        amount: order.amount.into(),
        quantity: order.quantity,
        status: order.status,
        payment_status: order.payment_status,
        fulfillment_mode: order.fulfillment_mode,
        fulfillment_state: order.fulfillment_state,
        is_auction_order: order.is_auction_order,
        tracking: AccountOrderTrackingOutput {
            available: order.tracking.available,
            carrier: budget.optional_text(order.tracking.carrier, 64),
            tracking_number: budget.optional_text(order.tracking.tracking_number, MAX_CODE_BYTES),
        },
        withdrawal_status: order.withdrawal_status,
        payout: order.payout.map(Into::into),
        payment_due_at: order.payment_due_at,
        paid_at: order.paid_at,
        ship_by: order.ship_by,
        created_at: order.created_at,
        updated_at: order.updated_at,
    }
}

impl From<AccountPage<AccountOrder>> for AccountPageOutput<AccountOrderOutput> {
    fn from(page: AccountPage<AccountOrder>) -> Self {
        project(page, project_order)
    }
}

const MAX_TRACKING_EVENTS: usize = 100;
const MAX_INVOICES: usize = 50;

#[derive(Debug, Serialize, schemars::JsonSchema)]
pub struct AccountOrderDetailOutput {
    pub order: AccountOrderOutput,
    #[schemars(length(max = 100))]
    pub tracking_events: Vec<AccountTrackingEventOutput>,
    pub order_return: Option<AccountOrderReturnOutput>,
    pub refund: Option<AccountRefundOutput>,
    pub dispute: Option<AccountDisputeOutput>,
    #[schemars(length(max = 50))]
    pub invoices: Vec<AccountInvoiceOutput>,
    pub truncated: bool,
    pub untrusted_content_notice: &'static str,
}

#[derive(Debug, Serialize, schemars::JsonSchema)]
pub struct AccountTrackingEventOutput {
    #[schemars(length(max = 128))]
    pub status: String,
    #[schemars(length(max = 512))]
    pub description: Option<String>,
    #[schemars(length(max = 256))]
    pub location: Option<String>,
    pub occurred_at: i64,
}

#[derive(Debug, Serialize, schemars::JsonSchema)]
pub struct AccountOrderReturnOutput {
    #[schemars(length(max = 64))]
    pub status: Option<String>,
    #[schemars(length(max = 64))]
    pub kind: Option<String>,
    pub requested_at: Option<i64>,
    pub deadline: Option<i64>,
    #[schemars(length(max = 64))]
    pub carrier: Option<String>,
    #[schemars(length(max = 128))]
    pub tracking_number: Option<String>,
    pub received_at: Option<i64>,
    pub delivered_at: Option<i64>,
}

#[derive(Debug, Serialize, schemars::JsonSchema)]
pub struct AccountRefundOutput {
    #[schemars(length(max = 64))]
    pub status: String,
    pub amount: MoneyOutput,
    pub created_at: i64,
    pub updated_at: i64,
}

#[derive(Debug, Serialize, schemars::JsonSchema)]
pub struct AccountDisputeOutput {
    #[schemars(length(max = 64))]
    pub status: String,
    #[schemars(length(max = 64))]
    pub reason: Option<String>,
    pub opened_at: i64,
    pub resolved_at: Option<i64>,
}

#[derive(Debug, Serialize, schemars::JsonSchema)]
pub struct AccountInvoiceOutput {
    #[schemars(length(max = 64))]
    pub kind: String,
    #[schemars(length(max = 128))]
    pub number: String,
    pub issued_at: i64,
    pub total: MoneyOutput,
}

impl From<AccountOrderDetail> for AccountOrderDetailOutput {
    fn from(detail: AccountOrderDetail) -> Self {
        let mut budget = ProjectionBudget::default();
        let order = project_order(detail.order, &mut budget);
        let tracking_events = budget
            .objects(detail.tracking_events, MAX_TRACKING_EVENTS)
            .into_iter()
            .map(|event| AccountTrackingEventOutput {
                status: budget.text(event.status, MAX_CODE_BYTES),
                description: budget.optional_text(event.description, MAX_DISPLAY_TEXT_BYTES),
                location: budget.optional_text(event.location, MAX_NAME_BYTES),
                occurred_at: event.occurred_at,
            })
            .collect();
        let order_return = detail
            .order_return
            .map(|order_return| AccountOrderReturnOutput {
                status: order_return.status,
                kind: order_return.kind,
                requested_at: order_return.requested_at,
                deadline: order_return.deadline,
                carrier: budget.optional_text(order_return.carrier, 64),
                tracking_number: budget.optional_text(order_return.tracking_number, MAX_CODE_BYTES),
                received_at: order_return.received_at,
                delivered_at: order_return.delivered_at,
            });
        let invoices = budget
            .objects(detail.invoices, MAX_INVOICES)
            .into_iter()
            .map(|invoice| AccountInvoiceOutput {
                kind: invoice.kind,
                number: budget.text(invoice.number, MAX_CODE_BYTES),
                issued_at: invoice.issued_at,
                total: invoice.total.into(),
            })
            .collect();
        Self {
            order,
            tracking_events,
            order_return,
            refund: detail.refund.map(|refund| AccountRefundOutput {
                status: refund.status,
                amount: refund.amount.into(),
                created_at: refund.created_at,
                updated_at: refund.updated_at,
            }),
            dispute: detail.dispute.map(|dispute| AccountDisputeOutput {
                status: dispute.status,
                reason: dispute.reason,
                opened_at: dispute.opened_at,
                resolved_at: dispute.resolved_at,
            }),
            invoices,
            truncated: budget.truncated(),
            untrusted_content_notice: UNTRUSTED_CONTENT_NOTICE,
        }
    }
}

#[derive(Debug, Serialize, schemars::JsonSchema)]
pub struct AccountAutoBidOutput {
    #[schemars(length(max = 64))]
    pub listing_id: String,
    #[schemars(length(max = 512))]
    pub listing_title: Option<String>,
    pub max_amount: MoneyOutput,
    pub increment: Option<MoneyOutput>,
    pub current_bid: Option<MoneyOutput>,
    #[schemars(length(max = 64))]
    pub status: String,
    pub expires_at: Option<i64>,
    pub created_at: i64,
    pub updated_at: i64,
}

impl From<AccountPage<AccountAutoBid>> for AccountPageOutput<AccountAutoBidOutput> {
    fn from(page: AccountPage<AccountAutoBid>) -> Self {
        project(page, |auto_bid, budget| AccountAutoBidOutput {
            listing_id: auto_bid.listing_id,
            listing_title: budget.optional_text(auto_bid.listing_title, MAX_DISPLAY_TEXT_BYTES),
            max_amount: auto_bid.max_amount.into(),
            increment: auto_bid.increment.map(Into::into),
            current_bid: auto_bid.current_bid.map(Into::into),
            status: auto_bid.status,
            expires_at: auto_bid.expires_at,
            created_at: auto_bid.created_at,
            updated_at: auto_bid.updated_at,
        })
    }
}

#[derive(Debug, Serialize, schemars::JsonSchema)]
pub struct AccountSecondChanceOfferOutput {
    pub offer_id: i64,
    #[schemars(length(max = 64))]
    pub listing_id: String,
    #[schemars(length(max = 512))]
    pub listing_title: Option<String>,
    pub price: MoneyOutput,
    pub your_bid: MoneyOutput,
    #[schemars(length(max = 64))]
    pub status: String,
    #[schemars(length(max = 64))]
    pub offer_status: String,
    pub window_ends_at: Option<i64>,
    pub order_id: Option<i64>,
    pub created_at: i64,
}

impl From<AccountPage<AccountSecondChanceOffer>>
    for AccountPageOutput<AccountSecondChanceOfferOutput>
{
    fn from(page: AccountPage<AccountSecondChanceOffer>) -> Self {
        project(page, |offer, budget| AccountSecondChanceOfferOutput {
            offer_id: offer.offer_id,
            listing_id: offer.listing_id,
            listing_title: budget.optional_text(offer.listing_title, MAX_DISPLAY_TEXT_BYTES),
            price: offer.price.into(),
            your_bid: offer.your_bid.into(),
            status: offer.status,
            offer_status: offer.offer_status,
            window_ends_at: offer.window_ends_at,
            order_id: offer.order_id,
            created_at: offer.created_at,
        })
    }
}

#[derive(Debug, Serialize, schemars::JsonSchema)]
pub struct AccountNotificationOutput {
    pub id: i64,
    #[schemars(length(max = 64))]
    pub notification_type: String,
    #[schemars(length(max = 64))]
    pub priority: String,
    #[schemars(length(max = 64))]
    pub listing_id: Option<String>,
    #[schemars(length(max = 512))]
    pub listing_title: Option<String>,
    pub order_id: Option<i64>,
    pub offer_id: Option<i64>,
    pub saved_search_id: Option<i64>,
    pub read: bool,
    pub created_at: i64,
}

impl From<AccountPage<AccountNotification>> for AccountPageOutput<AccountNotificationOutput> {
    fn from(page: AccountPage<AccountNotification>) -> Self {
        project(page, |notification, budget| AccountNotificationOutput {
            id: notification.id,
            notification_type: notification.notification_type,
            priority: notification.priority,
            listing_id: notification.listing_id,
            listing_title: budget.optional_text(notification.listing_title, MAX_DISPLAY_TEXT_BYTES),
            order_id: notification.order_id,
            offer_id: notification.offer_id,
            saved_search_id: notification.saved_search_id,
            read: notification.read,
            created_at: notification.created_at,
        })
    }
}

#[derive(Debug, Serialize, schemars::JsonSchema)]
pub struct AccountRatingOutput {
    #[schemars(length(max = 64))]
    pub listing_id: String,
    #[schemars(length(max = 512))]
    pub listing_title: Option<String>,
    #[schemars(range(min = 1, max = 5))]
    pub value: i32,
    #[schemars(length(max = 4096))]
    pub comment: Option<String>,
    pub created_at: i64,
}

impl From<AccountPage<AccountRating>> for AccountPageOutput<AccountRatingOutput> {
    fn from(page: AccountPage<AccountRating>) -> Self {
        project(page, |rating, budget| AccountRatingOutput {
            listing_id: rating.listing_id,
            listing_title: budget.optional_text(rating.listing_title, MAX_DISPLAY_TEXT_BYTES),
            value: rating.value,
            comment: budget.optional_text(rating.comment, MAX_LONG_TEXT_BYTES),
            created_at: rating.created_at,
        })
    }
}

#[derive(Debug, Serialize, schemars::JsonSchema)]
pub struct AccountMoneySummaryOutput {
    #[schemars(length(max = 10))]
    pub currencies: Vec<AccountCurrencyTotalsOutput>,
}

#[derive(Debug, Serialize, schemars::JsonSchema)]
pub struct AccountCurrencyTotalsOutput {
    #[schemars(length(min = 3, max = 3))]
    pub currency: String,
    pub incoming_pending: MoneyOutput,
    pub incoming_on_the_way: MoneyOutput,
    pub incoming_paid_out: MoneyOutput,
    pub incoming_on_hold: MoneyOutput,
    pub incoming_in_balance: MoneyOutput,
    pub outgoing_paid: MoneyOutput,
    pub outgoing_refunded: MoneyOutput,
}

impl From<AccountMoneySummary> for AccountMoneySummaryOutput {
    fn from(summary: AccountMoneySummary) -> Self {
        Self {
            currencies: summary
                .currencies
                .into_iter()
                .map(|totals| AccountCurrencyTotalsOutput {
                    currency: totals.currency,
                    incoming_pending: totals.incoming_pending.into(),
                    incoming_on_the_way: totals.incoming_on_the_way.into(),
                    incoming_paid_out: totals.incoming_paid_out.into(),
                    incoming_on_hold: totals.incoming_on_hold.into(),
                    incoming_in_balance: totals.incoming_in_balance.into(),
                    outgoing_paid: totals.outgoing_paid.into(),
                    outgoing_refunded: totals.outgoing_refunded.into(),
                })
                .collect(),
        }
    }
}

#[derive(Debug, Serialize, schemars::JsonSchema)]
pub struct AccountPaymentActivityOutput {
    #[schemars(length(max = 50))]
    pub items: Vec<AccountPaymentActivityEntryOutput>,
    #[schemars(length(max = 128))]
    pub next_cursor: Option<String>,
    pub truncated: bool,
    pub untrusted_content_notice: &'static str,
}

#[derive(Debug, Serialize, schemars::JsonSchema)]
pub struct AccountPaymentActivityEntryOutput {
    #[schemars(length(max = 128))]
    pub id: String,
    #[schemars(length(max = 64))]
    pub direction: String,
    #[schemars(length(max = 64))]
    pub kind: String,
    #[schemars(length(max = 64))]
    pub status: String,
    pub amount: MoneyOutput,
    #[schemars(length(max = 512))]
    pub title: Option<String>,
    pub order_id: Option<i64>,
    #[schemars(length(max = 64))]
    pub listing_id: Option<String>,
    pub release_eligible_at: Option<i64>,
    pub created_at: i64,
}

impl From<AccountPaymentActivity> for AccountPaymentActivityOutput {
    fn from(activity: AccountPaymentActivity) -> Self {
        let mut budget = ProjectionBudget::default();
        let items = budget
            .objects(activity.items, MAX_PAGE_ITEMS)
            .into_iter()
            .map(|entry| AccountPaymentActivityEntryOutput {
                id: entry.id,
                direction: entry.direction,
                kind: entry.kind,
                status: entry.status,
                amount: entry.amount.into(),
                title: budget.optional_text(entry.title, MAX_DISPLAY_TEXT_BYTES),
                order_id: entry.order_id,
                listing_id: entry.listing_id,
                release_eligible_at: entry.release_eligible_at,
                created_at: entry.created_at,
            })
            .collect();
        Self {
            items,
            next_cursor: activity.next_cursor,
            truncated: budget.truncated(),
            untrusted_content_notice: UNTRUSTED_CONTENT_NOTICE,
        }
    }
}

#[derive(Debug, Serialize, schemars::JsonSchema)]
pub struct AccountPayoutsOutput {
    #[schemars(length(max = 50))]
    pub items: Vec<AccountPayoutOutput>,
}

#[derive(Debug, Serialize, schemars::JsonSchema)]
pub struct AccountPayoutOutput {
    #[schemars(length(max = 128))]
    pub id: String,
    #[schemars(length(max = 64))]
    pub status: String,
    pub amount: MoneyOutput,
    pub requested_at: i64,
    pub settled_at: Option<i64>,
    pub paid_out_at: Option<i64>,
    pub failed_at: Option<i64>,
}

impl From<AccountPayouts> for AccountPayoutsOutput {
    fn from(payouts: AccountPayouts) -> Self {
        Self {
            items: payouts
                .items
                .into_iter()
                .map(|payout| AccountPayoutOutput {
                    id: payout.id,
                    status: payout.status,
                    amount: payout.amount.into(),
                    requested_at: payout.requested_at,
                    settled_at: payout.settled_at,
                    paid_out_at: payout.paid_out_at,
                    failed_at: payout.failed_at,
                })
                .collect(),
        }
    }
}

#[derive(Debug, Serialize, schemars::JsonSchema)]
pub struct AccountDebtOutput {
    pub id: i64,
    pub amount: MoneyOutput,
    #[schemars(length(max = 64))]
    pub status: String,
    #[schemars(length(max = 64))]
    pub reason: String,
    pub order_id: i64,
    pub created_at: i64,
    pub resolved_at: Option<i64>,
    pub waived_at: Option<i64>,
}

impl From<AccountPage<AccountDebt>> for AccountPageOutput<AccountDebtOutput> {
    fn from(page: AccountPage<AccountDebt>) -> Self {
        project(page, |debt, _budget| AccountDebtOutput {
            id: debt.id,
            amount: debt.amount.into(),
            status: debt.status,
            reason: debt.reason,
            order_id: debt.order_id,
            created_at: debt.created_at,
            resolved_at: debt.resolved_at,
            waived_at: debt.waived_at,
        })
    }
}

#[cfg(test)]
mod tests;
