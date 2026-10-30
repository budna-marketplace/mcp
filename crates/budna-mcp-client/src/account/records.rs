//! Allowlists for the account's disputes, combined-shipping groups, negotiations, alerts, chats,
//! settings and rewards.
use std::collections::BTreeMap;

use serde::Deserialize;

use super::activity::{code, money, optional_text, optional_time};
use crate::{Money, is_currency_code, is_listing_id, is_live_stream_id};

const MAX_TITLE_BYTES: usize = 4096;
const MAX_NAME_BYTES: usize = 256;

#[derive(Clone, Debug, Deserialize)]
pub struct AccountOwnDispute {
    pub id: i64,
    pub order_id: i64,
    pub listing_id: String,
    pub listing_title: Option<String>,
    pub role: String,
    pub reason: String,
    pub status: String,
    pub opened_at: i64,
    pub updated_at: i64,
    pub resolved_at: Option<i64>,
}

#[derive(Clone, Debug, Deserialize)]
pub struct AccountDisputeDetail {
    #[serde(flatten)]
    pub dispute: AccountOwnDispute,
    pub statements: Vec<AccountDisputeStatement>,
    pub settlement_offers: Vec<AccountSettlementOffer>,
}

#[derive(Clone, Debug, Deserialize)]
pub struct AccountDisputeStatement {
    pub from: String,
    pub text: String,
    pub created_at: i64,
}

#[derive(Clone, Debug, Deserialize)]
pub struct AccountSettlementOffer {
    pub from: String,
    pub amount: Money,
    pub status: String,
    pub expires_at: i64,
    pub created_at: i64,
    pub responded_at: Option<i64>,
}

/// A combined-shipping parcel: its member orders and sums, never an address or service point.
#[derive(Clone, Debug, Deserialize)]
pub struct AccountOrderGroup {
    pub id: i64,
    pub role: String,
    pub counterparty_username: Option<String>,
    pub status: String,
    pub order_ids: Vec<i64>,
    pub item_amount: Money,
    pub buyer_protection_amount: Money,
    pub high_value_fee_amount: Money,
    pub shipping_amount: Option<Money>,
    pub total_amount: Option<Money>,
    pub fulfillment_mode: Option<String>,
    pub carrier: Option<String>,
    pub payment_deadline: i64,
    pub seller_answer_deadline: i64,
    pub dissolved_reason: Option<String>,
    pub created_at: i64,
    pub updated_at: i64,
}

#[derive(Clone, Debug, Deserialize)]
pub struct AccountOfferHistory {
    pub quantity: i32,
    pub steps: Vec<AccountOfferStep>,
}

#[derive(Clone, Debug, Deserialize)]
pub struct AccountOfferStep {
    pub actor: String,
    pub action: String,
    pub amount: Money,
    pub at: i64,
}

#[derive(Clone, Debug, Deserialize)]
pub struct AccountSavedSearchMatch {
    pub listing_id: String,
    pub title: Option<String>,
    pub price: Money,
    pub listing_type: String,
    pub end_time: i64,
}

#[derive(Clone, Debug, Deserialize)]
pub struct AccountLiveReminder {
    pub session_id: String,
    pub title: Option<String>,
    pub status: String,
    pub scheduled_start_at: i64,
}

#[derive(Clone, Debug, Deserialize)]
pub struct AccountListingChat {
    pub listing_id: String,
    pub listing_title: Option<String>,
    pub role: String,
    pub last_message_from: String,
    pub counterparty_username: Option<String>,
    pub last_message_preview: String,
    pub unread_count: i64,
    pub last_activity_at: i64,
}

/// Search has no total, so a page says only whether another follows.
#[derive(Clone, Debug, Deserialize)]
pub struct AccountMessageSearch {
    pub items: Vec<AccountMessageMatch>,
    pub next_page: Option<u32>,
}

#[derive(Clone, Debug, Deserialize)]
pub struct AccountMessageMatch {
    pub conversation_id: i64,
    pub message_id: i64,
    pub from: String,
    pub snippet: String,
    pub sent_at: i64,
}

#[derive(Clone, Debug, Deserialize)]
pub struct AccountNotificationPreferences {
    pub push_enabled: bool,
    pub email_enabled: bool,
    pub in_app_enabled: bool,
    pub categories: BTreeMap<String, bool>,
    pub quiet_hours_start: Option<i32>,
    pub quiet_hours_end: Option<i32>,
    pub updated_at: i64,
}

#[derive(Clone, Debug, Deserialize)]
pub struct AccountCredits {
    pub balances: Vec<AccountCreditBalance>,
}

#[derive(Clone, Debug, Deserialize)]
pub struct AccountCreditBalance {
    pub kind: String,
    pub granted: i64,
    pub consumed: i64,
    pub available: i64,
    pub next_expiry_at: Option<i64>,
}

/// Verification and activation states only: no bank or account numbers and no provider links.
#[derive(Clone, Debug, Deserialize)]
pub struct AccountPayoutSetup {
    pub readiness: String,
    pub merchant_kind: String,
    pub verification_status: String,
    pub payout_status: String,
    pub agreement_status: String,
    pub settlement_account_status: String,
    pub methods: Vec<AccountPayoutMethod>,
}

#[derive(Clone, Debug, Deserialize)]
pub struct AccountPayoutMethod {
    pub method: String,
    pub currency: String,
    pub status: String,
}

#[derive(Clone, Debug, Deserialize)]
pub struct AccountAchievements {
    pub achievements: Vec<AccountAchievement>,
    pub hidden_remaining: i32,
}

#[derive(Clone, Debug, Deserialize)]
pub struct AccountAchievement {
    pub name: String,
    pub description: String,
    pub category: String,
    pub progress: i32,
    pub target: i32,
    pub unlocked: bool,
    pub unlocked_at: Option<i64>,
}

/// Counts and rewards only: never who was referred.
#[derive(Clone, Debug, Deserialize)]
pub struct AccountReferral {
    pub verified: bool,
    pub code: Option<String>,
    pub counted: i64,
    pub waiting: i64,
    pub rank: Option<i64>,
    pub rewards: AccountReferralRewards,
    pub closes_at: Option<i64>,
}

#[derive(Clone, Debug, Deserialize)]
pub struct AccountReferralRewards {
    pub premium_months: u32,
    pub boosts: u32,
    pub next: Option<AccountReferralStep>,
    pub premium_for_good: bool,
    pub lifetime_tier: Option<String>,
}

#[derive(Clone, Debug, Deserialize)]
pub struct AccountReferralStep {
    pub referrals: u32,
    pub premium_months: u32,
}

#[derive(Clone, Debug, Deserialize)]
pub struct AccountLegalDocuments {
    pub documents: Vec<AccountLegalDocument>,
}

#[derive(Clone, Debug, Deserialize)]
pub struct AccountLegalDocument {
    pub document: String,
    pub current_version: String,
    pub accepted_version: Option<String>,
    pub accepted_at: Option<i64>,
    pub pending: bool,
    pub blocking: bool,
}

fn side(value: &str) -> bool {
    matches!(value, "buyer" | "seller")
}

fn party(value: &str) -> bool {
    matches!(value, "me" | "other_party")
}

/// Document versions such as `1.0` or `2025-02`.
fn version(value: &str) -> bool {
    !value.is_empty() && value.len() <= 64 && value.bytes().all(|byte| byte.is_ascii_graphic())
}

fn referral_code(value: &str) -> bool {
    !value.is_empty()
        && value.len() <= 32
        && value
            .bytes()
            .all(|byte| byte.is_ascii_uppercase() || byte.is_ascii_digit() || byte == b'-')
}

impl AccountOwnDispute {
    pub(super) fn is_valid(&self) -> bool {
        self.id > 0
            && self.order_id > 0
            && is_listing_id(&self.listing_id)
            && optional_text(self.listing_title.as_deref(), MAX_TITLE_BYTES)
            && side(&self.role)
            && code(&self.reason)
            && code(&self.status)
            && self.opened_at >= 0
            && self.updated_at >= 0
            && optional_time(self.resolved_at)
    }
}

impl AccountDisputeDetail {
    pub(super) fn is_valid(&self) -> bool {
        self.dispute.is_valid()
            && self.statements.len() <= 100
            && self.statements.iter().all(|statement| {
                party(&statement.from)
                    && statement.text.len() <= 20_000
                    && statement.created_at >= 0
            })
            && self.settlement_offers.len() <= 100
            && self.settlement_offers.iter().all(|offer| {
                party(&offer.from)
                    && money(&offer.amount)
                    && code(&offer.status)
                    && offer.expires_at >= 0
                    && offer.created_at >= 0
                    && optional_time(offer.responded_at)
            })
    }
}

impl AccountOrderGroup {
    pub(super) fn is_valid(&self) -> bool {
        let currency = self.item_amount.currency_code.as_str();
        let in_currency = |amount: &Money| money(amount) && amount.currency_code == currency;
        self.id > 0
            && side(&self.role)
            && optional_text(self.counterparty_username.as_deref(), MAX_NAME_BYTES)
            && code(&self.status)
            && self.order_ids.len() <= 50
            && self.order_ids.iter().all(|id| *id > 0)
            && [
                &self.item_amount,
                &self.buyer_protection_amount,
                &self.high_value_fee_amount,
            ]
            .into_iter()
            .all(in_currency)
            && [&self.shipping_amount, &self.total_amount]
                .into_iter()
                .all(|amount| amount.as_ref().is_none_or(in_currency))
            && self.fulfillment_mode.as_deref().is_none_or(code)
            && self.carrier.as_deref().is_none_or(code)
            && self.payment_deadline >= 0
            && self.seller_answer_deadline >= 0
            && self.dissolved_reason.as_deref().is_none_or(code)
            && self.created_at >= 0
            && self.updated_at >= 0
    }
}

impl AccountOfferHistory {
    pub(super) fn is_valid(&self) -> bool {
        let currency = self
            .steps
            .first()
            .map(|step| step.amount.currency_code.as_str());
        self.quantity >= 0
            && self.steps.len() <= 100
            && self.steps.iter().all(|step| {
                party(&step.actor)
                    && code(&step.action)
                    && money(&step.amount)
                    && Some(step.amount.currency_code.as_str()) == currency
                    && step.at >= 0
            })
    }
}

impl AccountSavedSearchMatch {
    pub(super) fn is_valid(&self) -> bool {
        is_listing_id(&self.listing_id)
            && optional_text(self.title.as_deref(), MAX_TITLE_BYTES)
            && money(&self.price)
            && code(&self.listing_type)
            && self.end_time >= 0
    }
}

impl AccountLiveReminder {
    pub(super) fn is_valid(&self) -> bool {
        is_live_stream_id(&self.session_id)
            && optional_text(self.title.as_deref(), MAX_TITLE_BYTES)
            && code(&self.status)
            && self.scheduled_start_at >= 0
    }
}

impl AccountListingChat {
    /// Only someone else's newest message names its author.
    pub(super) fn is_valid(&self) -> bool {
        let named_by_other =
            self.counterparty_username.is_none() || self.last_message_from == "other_party";
        is_listing_id(&self.listing_id)
            && optional_text(self.listing_title.as_deref(), MAX_TITLE_BYTES)
            && side(&self.role)
            && party(&self.last_message_from)
            && named_by_other
            && optional_text(self.counterparty_username.as_deref(), MAX_NAME_BYTES)
            && self.last_message_preview.len() <= 8192
            && self.unread_count >= 0
            && self.last_activity_at >= 0
    }
}

impl AccountMessageSearch {
    pub(super) fn is_valid(&self, page: u32, limit: u32) -> bool {
        self.items.len() <= limit as usize
            && self.next_page.is_none_or(|next| next == page + 1)
            && self.items.iter().all(|found| {
                found.conversation_id > 0
                    && found.message_id > 0
                    && party(&found.from)
                    && found.snippet.len() <= 4096
                    && found.sent_at >= 0
            })
    }
}

impl AccountNotificationPreferences {
    pub(super) fn is_valid(&self) -> bool {
        self.categories.len() <= 32
            && self.categories.keys().all(|category| code(category))
            && [self.quiet_hours_start, self.quiet_hours_end]
                .into_iter()
                .all(|hour| hour.is_none_or(|hour| (0..24).contains(&hour)))
            && self.updated_at >= 0
    }
}

impl AccountCredits {
    pub(super) fn is_valid(&self) -> bool {
        self.balances.len() <= 16
            && self.balances.iter().all(|balance| {
                code(&balance.kind)
                    && balance.granted >= 0
                    && balance.consumed >= 0
                    && balance.available >= 0
                    && optional_time(balance.next_expiry_at)
            })
    }
}

impl AccountPayoutSetup {
    pub(super) fn is_valid(&self) -> bool {
        [
            &self.readiness,
            &self.merchant_kind,
            &self.verification_status,
            &self.payout_status,
            &self.agreement_status,
            &self.settlement_account_status,
        ]
        .into_iter()
        .all(|value| code(value))
            && self.methods.len() <= 16
            && self.methods.iter().all(|method| {
                code(&method.method) && is_currency_code(&method.currency) && code(&method.status)
            })
    }
}

impl AccountAchievements {
    pub(super) fn is_valid(&self) -> bool {
        self.hidden_remaining >= 0
            && self.achievements.len() <= 500
            && self.achievements.iter().all(|achievement| {
                achievement.name.len() <= MAX_NAME_BYTES
                    && achievement.description.len() <= MAX_TITLE_BYTES
                    && code(&achievement.category)
                    && achievement.progress >= 0
                    && achievement.target >= 0
                    && optional_time(achievement.unlocked_at)
            })
    }
}

impl AccountReferral {
    pub(super) fn is_valid(&self) -> bool {
        self.code.as_deref().is_none_or(referral_code)
            && self.counted >= 0
            && self.waiting >= 0
            && self.rank.is_none_or(|rank| rank > 0)
            && self.rewards.lifetime_tier.as_deref().is_none_or(code)
            && optional_time(self.closes_at)
    }
}

impl AccountLegalDocuments {
    pub(super) fn is_valid(&self) -> bool {
        self.documents.len() <= 16
            && self.documents.iter().all(|document| {
                code(&document.document)
                    && version(&document.current_version)
                    && document.accepted_version.as_deref().is_none_or(version)
                    && optional_time(document.accepted_at)
            })
    }
}
