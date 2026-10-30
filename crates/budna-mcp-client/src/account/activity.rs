//! Allowlists for the connected account's own marketplace activity.
use std::collections::BTreeMap;

use serde::Deserialize;

use crate::{Money, is_currency_code, is_exact_decimal, is_listing_id};

#[derive(Clone, Debug, Deserialize)]
pub struct AccountBid {
    pub listing_id: String,
    pub listing_title: Option<String>,
    pub amount: Money,
    pub placed_at: i64,
    pub status: String,
}

#[derive(Clone, Debug, Deserialize)]
pub struct AccountOffer {
    pub id: i64,
    pub listing_id: String,
    pub listing_title: Option<String>,
    pub role: String,
    pub amount: Money,
    pub quantity: i32,
    pub counter_amount: Option<Money>,
    pub status: String,
    pub last_action_by: Option<String>,
    pub amount_withdrawn_at: Option<i64>,
    pub expires_at: i64,
    pub created_at: i64,
    pub updated_at: i64,
}

#[derive(Clone, Debug, Deserialize)]
pub struct AccountWatchlist {
    pub id: i64,
    pub name: String,
    pub is_default: bool,
    pub item_count: i64,
    pub created_at: i64,
    pub updated_at: i64,
}

#[derive(Clone, Debug, Deserialize)]
pub struct AccountWatchedListing {
    pub listing_id: String,
    pub title: Option<String>,
}

#[derive(Clone, Debug, Deserialize)]
pub struct AccountSavedSearch {
    pub id: i64,
    pub name: String,
    pub query: AccountSavedSearchQuery,
    pub notification_frequency: String,
    pub is_active: bool,
    pub unseen_count: i64,
    pub total_matches: i64,
    pub last_viewed_at: Option<i64>,
    pub created_at: i64,
    pub updated_at: i64,
}

#[derive(Clone, Debug, Deserialize)]
pub struct AccountSavedSearchQuery {
    pub text: Option<String>,
    pub category_id: Option<i32>,
    pub market: Option<String>,
    pub min_price: Option<Money>,
    pub max_price: Option<Money>,
    pub condition: Option<String>,
    pub listing_type: Option<String>,
    pub location_region: Option<String>,
    pub location_municipality: Option<String>,
    pub allow_pickup: Option<bool>,
    pub filters: BTreeMap<String, String>,
}

#[derive(Clone, Debug, Deserialize)]
pub struct AccountOrder {
    pub id: i64,
    pub role: String,
    pub listing_id: String,
    pub listing_title: Option<String>,
    pub amount: Money,
    pub quantity: i32,
    pub status: String,
    pub payment_status: String,
    pub fulfillment_mode: String,
    pub fulfillment_state: String,
    pub is_auction_order: bool,
    pub tracking: AccountOrderTracking,
    pub withdrawal_status: Option<String>,
    pub payout: Option<Money>,
    pub payment_due_at: Option<i64>,
    pub paid_at: Option<i64>,
    pub ship_by: Option<i64>,
    pub created_at: i64,
    pub updated_at: i64,
}

#[derive(Clone, Debug, Deserialize)]
pub struct AccountOrderTracking {
    pub available: bool,
    pub carrier: Option<String>,
    pub tracking_number: Option<String>,
}

#[derive(Clone, Debug, Deserialize)]
pub struct AccountNotification {
    pub id: i64,
    pub notification_type: String,
    pub priority: String,
    pub listing_id: Option<String>,
    pub listing_title: Option<String>,
    pub order_id: Option<i64>,
    pub offer_id: Option<i64>,
    pub saved_search_id: Option<i64>,
    pub read: bool,
    pub created_at: i64,
}

#[derive(Clone, Debug, Deserialize)]
pub struct AccountRating {
    pub listing_id: String,
    pub listing_title: Option<String>,
    pub value: i32,
    pub comment: Option<String>,
    pub created_at: i64,
}

#[derive(Clone, Debug, Deserialize)]
pub struct AccountOrderDetail {
    pub order: AccountOrder,
    pub tracking_events: Vec<AccountTrackingEvent>,
    pub order_return: Option<AccountOrderReturn>,
    pub refund: Option<AccountRefund>,
    pub dispute: Option<AccountDispute>,
    pub invoices: Vec<AccountInvoice>,
}

#[derive(Clone, Debug, Deserialize)]
pub struct AccountTrackingEvent {
    pub status: String,
    pub description: Option<String>,
    pub location: Option<String>,
    pub occurred_at: i64,
}

#[derive(Clone, Debug, Deserialize)]
pub struct AccountOrderReturn {
    pub status: Option<String>,
    pub kind: Option<String>,
    pub requested_at: Option<i64>,
    pub deadline: Option<i64>,
    pub carrier: Option<String>,
    pub tracking_number: Option<String>,
    pub received_at: Option<i64>,
    pub delivered_at: Option<i64>,
}

#[derive(Clone, Debug, Deserialize)]
pub struct AccountRefund {
    pub status: String,
    pub amount: Money,
    pub created_at: i64,
    pub updated_at: i64,
}

#[derive(Clone, Debug, Deserialize)]
pub struct AccountDispute {
    pub status: String,
    pub reason: Option<String>,
    pub opened_at: i64,
    pub resolved_at: Option<i64>,
}

#[derive(Clone, Debug, Deserialize)]
pub struct AccountInvoice {
    pub kind: String,
    pub number: String,
    pub issued_at: i64,
    pub total: Money,
}

#[derive(Clone, Debug, Deserialize)]
pub struct AccountAutoBid {
    pub listing_id: String,
    pub listing_title: Option<String>,
    pub max_amount: Money,
    pub increment: Option<Money>,
    pub current_bid: Option<Money>,
    pub status: String,
    pub expires_at: Option<i64>,
    pub created_at: i64,
    pub updated_at: i64,
}

#[derive(Clone, Debug, Deserialize)]
pub struct AccountSecondChanceOffer {
    pub offer_id: i64,
    pub listing_id: String,
    pub listing_title: Option<String>,
    pub price: Money,
    pub your_bid: Money,
    pub status: String,
    pub offer_status: String,
    pub window_ends_at: Option<i64>,
    pub order_id: Option<i64>,
    pub created_at: i64,
}

#[derive(Clone, Debug, Deserialize)]
pub struct AccountMoneySummary {
    pub currencies: Vec<AccountCurrencyTotals>,
}

#[derive(Clone, Debug, Deserialize)]
pub struct AccountCurrencyTotals {
    pub currency: String,
    pub incoming_pending: Money,
    pub incoming_on_the_way: Money,
    pub incoming_paid_out: Money,
    pub incoming_on_hold: Money,
    pub incoming_in_balance: Money,
    pub outgoing_paid: Money,
    pub outgoing_refunded: Money,
}

#[derive(Clone, Debug, Deserialize)]
pub struct AccountPaymentActivity {
    pub items: Vec<AccountPaymentActivityEntry>,
    pub next_cursor: Option<String>,
}

#[derive(Clone, Debug, Deserialize)]
pub struct AccountPaymentActivityEntry {
    pub id: String,
    pub direction: String,
    pub kind: String,
    pub status: String,
    pub amount: Money,
    pub title: Option<String>,
    pub order_id: Option<i64>,
    pub listing_id: Option<String>,
    pub release_eligible_at: Option<i64>,
    pub created_at: i64,
}

#[derive(Clone, Debug, Deserialize)]
pub struct AccountPayouts {
    pub items: Vec<AccountPayout>,
}

#[derive(Clone, Debug, Deserialize)]
pub struct AccountPayout {
    pub id: String,
    pub status: String,
    pub amount: Money,
    pub requested_at: i64,
    pub settled_at: Option<i64>,
    pub paid_out_at: Option<i64>,
    pub failed_at: Option<i64>,
}

#[derive(Clone, Debug, Deserialize)]
pub struct AccountDebt {
    pub id: i64,
    pub amount: Money,
    pub status: String,
    pub reason: String,
    pub order_id: i64,
    pub created_at: i64,
    pub resolved_at: Option<i64>,
    pub waived_at: Option<i64>,
}

/// Status values are backend vocabulary that may grow, so they are checked by shape.
fn code(value: &str) -> bool {
    let mut bytes = value.bytes();
    value.len() <= 64
        && bytes.next().is_some_and(|byte| byte.is_ascii_lowercase())
        && bytes.all(|byte| byte.is_ascii_lowercase() || byte.is_ascii_digit() || byte == b'_')
}

fn optional_text(value: Option<&str>, max: usize) -> bool {
    value.is_none_or(|value| value.len() <= max)
}

fn optional_time(value: Option<i64>) -> bool {
    value.is_none_or(|value| value >= 0)
}

fn optional_id(value: Option<i64>) -> bool {
    value.is_none_or(|value| value > 0)
}

fn money(value: &Money) -> bool {
    is_currency_code(&value.currency_code) && is_exact_decimal(&value.amount)
}

impl AccountBid {
    pub(super) fn is_valid(&self) -> bool {
        is_listing_id(&self.listing_id)
            && optional_text(self.listing_title.as_deref(), 4096)
            && money(&self.amount)
            && self.placed_at >= 0
            && code(&self.status)
    }
}

impl AccountOffer {
    pub(super) fn is_valid(&self, role: &str) -> bool {
        self.id > 0
            && self.role == role
            && is_listing_id(&self.listing_id)
            && optional_text(self.listing_title.as_deref(), 4096)
            && money(&self.amount)
            && self.quantity >= 0
            && self.counter_amount.as_ref().is_none_or(|counter| {
                money(counter) && counter.currency_code == self.amount.currency_code
            })
            && code(&self.status)
            && self
                .last_action_by
                .as_deref()
                .is_none_or(|actor| matches!(actor, "me" | "other_party"))
            && optional_time(self.amount_withdrawn_at)
            && self.expires_at >= 0
            && self.created_at >= 0
            && self.updated_at >= 0
    }
}

impl AccountWatchlist {
    pub(super) fn is_valid(&self) -> bool {
        self.id > 0
            && self.name.len() <= 4096
            && self.item_count >= 0
            && self.created_at >= 0
            && self.updated_at >= 0
    }
}

impl AccountWatchedListing {
    pub(super) fn is_valid(&self) -> bool {
        is_listing_id(&self.listing_id) && optional_text(self.title.as_deref(), 4096)
    }
}

impl AccountSavedSearch {
    pub(super) fn is_valid(&self) -> bool {
        let query = &self.query;
        self.id > 0
            && self.name.len() <= 4096
            && self.notification_frequency.len() <= 64
            && self.unseen_count >= 0
            && self.total_matches >= 0
            && optional_time(self.last_viewed_at)
            && self.created_at >= 0
            && self.updated_at >= 0
            && optional_text(query.text.as_deref(), 4096)
            && query.category_id.is_none_or(|id| id > 0)
            && [&query.market, &query.condition, &query.listing_type]
                .into_iter()
                .all(|value| optional_text(value.as_deref(), 64))
            && [&query.min_price, &query.max_price]
                .into_iter()
                .all(|price| price.as_ref().is_none_or(money))
            && optional_text(query.location_region.as_deref(), 4096)
            && optional_text(query.location_municipality.as_deref(), 4096)
            && query.filters.len() <= 50
            && query
                .filters
                .iter()
                .all(|(name, value)| name.len() <= 256 && value.len() <= 4096)
    }
}

impl AccountOrder {
    pub(super) fn is_valid(&self, role: &str) -> bool {
        self.id > 0
            && self.role == role
            && is_listing_id(&self.listing_id)
            && optional_text(self.listing_title.as_deref(), 4096)
            && money(&self.amount)
            && self.quantity >= 0
            && [
                &self.status,
                &self.payment_status,
                &self.fulfillment_mode,
                &self.fulfillment_state,
            ]
            .into_iter()
            .all(|value| code(value))
            && self.withdrawal_status.as_deref().is_none_or(code)
            && self.payout.as_ref().is_none_or(money)
            && optional_text(self.tracking.carrier.as_deref(), 256)
            && optional_text(self.tracking.tracking_number.as_deref(), 256)
            && [self.payment_due_at, self.paid_at, self.ship_by]
                .into_iter()
                .all(optional_time)
            && self.created_at >= 0
            && self.updated_at >= 0
    }
}

impl AccountNotification {
    pub(super) fn is_valid(&self) -> bool {
        self.id > 0
            && code(&self.notification_type)
            && code(&self.priority)
            && self.listing_id.as_deref().is_none_or(is_listing_id)
            && optional_text(self.listing_title.as_deref(), 4096)
            && [self.order_id, self.offer_id, self.saved_search_id]
                .into_iter()
                .all(optional_id)
            && self.created_at >= 0
    }
}

impl AccountRating {
    pub(super) fn is_valid(&self) -> bool {
        is_listing_id(&self.listing_id)
            && optional_text(self.listing_title.as_deref(), 4096)
            && (1..=5).contains(&self.value)
            && optional_text(self.comment.as_deref(), 8192)
            && self.created_at >= 0
    }
}

impl AccountOrderDetail {
    pub(super) fn is_valid(&self) -> bool {
        matches!(self.order.role.as_str(), "buyer" | "seller")
            && self.order.is_valid(&self.order.role)
            && self.tracking_events.len() <= 200
            && self.tracking_events.iter().all(|event| {
                !event.status.is_empty()
                    && event.status.len() <= 128
                    && optional_text(event.description.as_deref(), 4096)
                    && optional_text(event.location.as_deref(), 4096)
                    && event.occurred_at >= 0
            })
            && self.order_return.as_ref().is_none_or(|order_return| {
                order_return.status.as_deref().is_none_or(code)
                    && order_return.kind.as_deref().is_none_or(code)
                    && optional_text(order_return.carrier.as_deref(), 256)
                    && optional_text(order_return.tracking_number.as_deref(), 256)
                    && [
                        order_return.requested_at,
                        order_return.deadline,
                        order_return.received_at,
                        order_return.delivered_at,
                    ]
                    .into_iter()
                    .all(optional_time)
            })
            && self.refund.as_ref().is_none_or(|refund| {
                code(&refund.status)
                    && money(&refund.amount)
                    && refund.created_at >= 0
                    && refund.updated_at >= 0
            })
            && self.dispute.as_ref().is_none_or(|dispute| {
                code(&dispute.status)
                    && dispute.reason.as_deref().is_none_or(code)
                    && dispute.opened_at >= 0
                    && optional_time(dispute.resolved_at)
            })
            && self.invoices.len() <= 100
            && self.invoices.iter().all(|invoice| {
                code(&invoice.kind)
                    && !invoice.number.is_empty()
                    && invoice.number.len() <= 128
                    && invoice.issued_at >= 0
                    && money(&invoice.total)
            })
    }
}

impl AccountAutoBid {
    pub(super) fn is_valid(&self) -> bool {
        is_listing_id(&self.listing_id)
            && optional_text(self.listing_title.as_deref(), 4096)
            && money(&self.max_amount)
            && [&self.increment, &self.current_bid]
                .into_iter()
                .all(|amount| amount.as_ref().is_none_or(money))
            && code(&self.status)
            && optional_time(self.expires_at)
            && self.created_at >= 0
            && self.updated_at >= 0
    }
}

impl AccountSecondChanceOffer {
    pub(super) fn is_valid(&self) -> bool {
        self.offer_id > 0
            && is_listing_id(&self.listing_id)
            && optional_text(self.listing_title.as_deref(), 4096)
            && money(&self.price)
            && money(&self.your_bid)
            && code(&self.status)
            && code(&self.offer_status)
            && optional_time(self.window_ends_at)
            && optional_id(self.order_id)
            && self.created_at >= 0
    }
}

fn opaque_id(value: &str) -> bool {
    !value.is_empty()
        && value.len() <= 128
        && value
            .bytes()
            .all(|byte| byte.is_ascii_alphanumeric() || b"-_".contains(&byte))
}

impl AccountMoneySummary {
    pub(super) fn is_valid(&self) -> bool {
        self.currencies.len() <= 10
            && self.currencies.iter().all(|totals| {
                is_currency_code(&totals.currency)
                    && [
                        &totals.incoming_pending,
                        &totals.incoming_on_the_way,
                        &totals.incoming_paid_out,
                        &totals.incoming_on_hold,
                        &totals.incoming_in_balance,
                        &totals.outgoing_paid,
                        &totals.outgoing_refunded,
                    ]
                    .into_iter()
                    .all(|amount| money(amount) && amount.currency_code == totals.currency)
            })
    }
}

impl AccountPaymentActivity {
    pub(super) fn is_valid(&self, limit: u32) -> bool {
        self.items.len() <= limit as usize
            && self
                .next_cursor
                .as_deref()
                .is_none_or(|cursor| !cursor.is_empty() && cursor.len() <= 128)
            && self.items.iter().all(|entry| {
                opaque_id(&entry.id)
                    && code(&entry.direction)
                    && code(&entry.kind)
                    && code(&entry.status)
                    && money(&entry.amount)
                    && optional_text(entry.title.as_deref(), 4096)
                    && optional_id(entry.order_id)
                    && entry.listing_id.as_deref().is_none_or(is_listing_id)
                    && optional_time(entry.release_eligible_at)
                    && entry.created_at >= 0
            })
    }
}

impl AccountPayouts {
    pub(super) fn is_valid(&self, limit: u32) -> bool {
        self.items.len() <= limit as usize
            && self.items.iter().all(|payout| {
                opaque_id(&payout.id)
                    && code(&payout.status)
                    && money(&payout.amount)
                    && payout.requested_at >= 0
                    && [payout.settled_at, payout.paid_out_at, payout.failed_at]
                        .into_iter()
                        .all(optional_time)
            })
    }
}

impl AccountDebt {
    pub(super) fn is_valid(&self) -> bool {
        self.id > 0
            && money(&self.amount)
            && code(&self.status)
            && code(&self.reason)
            && self.order_id > 0
            && self.created_at >= 0
            && optional_time(self.resolved_at)
            && optional_time(self.waived_at)
    }
}
