//! Deployment-side write families. An enabled permission still needs a current customer grant.

use crate::ConfigError;

pub const MESSAGES_WRITE_SCOPE: &str = "messages:write";
pub const MESSAGE_REACTIONS_WRITE_SCOPE: &str = "messagereactions:write";
pub const PUBLIC_CHAT_WRITE_SCOPE: &str = "publicchat:write";
pub const RATINGS_WRITE_SCOPE: &str = "ratings:write";
pub const CHAT_ROOMS_READ_SCOPE: &str = "chatrooms:read";
pub const ROOM_MODERATION_READ_SCOPE: &str = "roommoderation:read";
pub const ROOM_REACTIONS_WRITE_SCOPE: &str = "roomreactions:write";
pub const LIVE_REPORTS_WRITE_SCOPE: &str = "livereports:write";
pub const ROOM_MODERATION_WRITE_SCOPE: &str = "roommoderation:write";
pub const BIDS_WRITE_SCOPE: &str = "bids:write";
pub const DRAFT_AUTHORING_SCOPE: &str = "draftauthoring:write";
pub const SHIPPING_RULES_WRITE_SCOPE: &str = "shippingrules:write";
pub const SHIPPING_RULES_READ_SCOPE: &str = "shippingrules:read";
pub const SHIPPING_DEFAULTS_SCOPE: &str = "shippingdefaults:write";
pub const INBOX_WRITE_SCOPE: &str = "inbox:write";
pub const PROFILE_WRITE_SCOPE: &str = "profile:write";
pub const FOLLOWING_WRITE_SCOPE: &str = "following:write";
pub const PERSONAL_BLOCKS_READ_SCOPE: &str = "personalblocks:read";
pub const PERSONAL_BLOCKS_WRITE_SCOPE: &str = "personalblocks:write";
pub const WATCHLIST_WRITE_SCOPE: &str = "watchlist:write";
pub const WATCHLIST_MANAGE_SCOPE: &str = "watchlists:write";
pub const DRAFT_CREATE_SCOPE: &str = "listingdrafts:write";
pub const SAVED_SEARCH_WRITE_SCOPE: &str = "savedsearches:write";
pub const NOTIFICATION_STATE_SCOPE: &str = "notificationstate:write";
pub const LIVE_REMINDER_WRITE_SCOPE: &str = "livereminders:write";

#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub struct AccountWritePermissions {
    pub watchlist_membership: bool,
    pub watchlist_management: bool,
    pub listing_drafts: bool,
    pub messages: bool,
    pub message_reactions: bool,
    pub public_chat: bool,
    pub ratings: bool,
    pub room_targets: bool,
    pub room_safety: bool,
    pub room_reactions: bool,
    pub live_reports: bool,
    pub room_moderation: bool,
    pub bids: bool,
    pub draft_authoring: bool,
    pub shipping_rules: bool,
    pub shipping_rules_read: bool,
    pub shipping_defaults: bool,
    pub inbox: bool,
    pub profile: bool,
    pub following: bool,
    pub personal_blocks_read: bool,
    pub personal_blocks: bool,
    pub live_reminders: bool,
    pub notification_state: bool,
    pub saved_searches: bool,
}

impl AccountWritePermissions {
    pub const NONE: Self = Self {
        watchlist_membership: false,
        watchlist_management: false,
        listing_drafts: false,
        messages: false,
        message_reactions: false,
        public_chat: false,
        ratings: false,
        room_targets: false,
        room_safety: false,
        room_reactions: false,
        live_reports: false,
        room_moderation: false,
        bids: false,
        draft_authoring: false,
        shipping_rules: false,
        shipping_rules_read: false,
        shipping_defaults: false,
        inbox: false,
        profile: false,
        following: false,
        personal_blocks_read: false,
        personal_blocks: false,
        live_reminders: false,
        notification_state: false,
        saved_searches: false,
    };

    pub fn from_scopes(value: &str) -> Result<Self, ConfigError> {
        let mut permissions = Self::NONE;
        for scope in value.split_ascii_whitespace() {
            let enabled = match scope {
                WATCHLIST_WRITE_SCOPE => &mut permissions.watchlist_membership,
                WATCHLIST_MANAGE_SCOPE => &mut permissions.watchlist_management,
                DRAFT_CREATE_SCOPE => &mut permissions.listing_drafts,
                SAVED_SEARCH_WRITE_SCOPE => &mut permissions.saved_searches,
                NOTIFICATION_STATE_SCOPE => &mut permissions.notification_state,
                MESSAGES_WRITE_SCOPE => &mut permissions.messages,
                MESSAGE_REACTIONS_WRITE_SCOPE => &mut permissions.message_reactions,
                PUBLIC_CHAT_WRITE_SCOPE => &mut permissions.public_chat,
                RATINGS_WRITE_SCOPE => &mut permissions.ratings,
                CHAT_ROOMS_READ_SCOPE => &mut permissions.room_targets,
                ROOM_MODERATION_READ_SCOPE => &mut permissions.room_safety,
                ROOM_REACTIONS_WRITE_SCOPE => &mut permissions.room_reactions,
                LIVE_REPORTS_WRITE_SCOPE => &mut permissions.live_reports,
                ROOM_MODERATION_WRITE_SCOPE => &mut permissions.room_moderation,
                BIDS_WRITE_SCOPE => &mut permissions.bids,
                DRAFT_AUTHORING_SCOPE => &mut permissions.draft_authoring,
                SHIPPING_RULES_WRITE_SCOPE => &mut permissions.shipping_rules,
                SHIPPING_RULES_READ_SCOPE => &mut permissions.shipping_rules_read,
                SHIPPING_DEFAULTS_SCOPE => &mut permissions.shipping_defaults,
                INBOX_WRITE_SCOPE => &mut permissions.inbox,
                PROFILE_WRITE_SCOPE => &mut permissions.profile,
                FOLLOWING_WRITE_SCOPE => &mut permissions.following,
                PERSONAL_BLOCKS_READ_SCOPE => &mut permissions.personal_blocks_read,
                PERSONAL_BLOCKS_WRITE_SCOPE => &mut permissions.personal_blocks,
                LIVE_REMINDER_WRITE_SCOPE => &mut permissions.live_reminders,
                _ => return Err(ConfigError::InvalidAccountSettings),
            };
            if *enabled {
                return Err(ConfigError::InvalidAccountSettings);
            }
            *enabled = true;
        }
        if permissions == Self::NONE {
            return Err(ConfigError::InvalidAccountSettings);
        }
        Ok(permissions)
    }

    pub fn additional_read_scopes(self) -> impl Iterator<Item = &'static str> {
        [
            self.personal_blocks_read
                .then_some(PERSONAL_BLOCKS_READ_SCOPE),
            self.shipping_rules_read
                .then_some(SHIPPING_RULES_READ_SCOPE),
            self.room_targets.then_some(CHAT_ROOMS_READ_SCOPE),
            self.room_safety.then_some(ROOM_MODERATION_READ_SCOPE),
        ]
        .into_iter()
        .flatten()
    }

    /// Write powers only. Supplemental reads cannot authorize action receipts.
    pub fn scopes(self) -> impl Iterator<Item = &'static str> {
        [
            self.watchlist_membership.then_some(WATCHLIST_WRITE_SCOPE),
            self.watchlist_management.then_some(WATCHLIST_MANAGE_SCOPE),
            self.listing_drafts.then_some(DRAFT_CREATE_SCOPE),
            self.messages.then_some(MESSAGES_WRITE_SCOPE),
            self.message_reactions
                .then_some(MESSAGE_REACTIONS_WRITE_SCOPE),
            self.public_chat.then_some(PUBLIC_CHAT_WRITE_SCOPE),
            self.ratings.then_some(RATINGS_WRITE_SCOPE),
            self.room_reactions.then_some(ROOM_REACTIONS_WRITE_SCOPE),
            self.live_reports.then_some(LIVE_REPORTS_WRITE_SCOPE),
            self.room_moderation.then_some(ROOM_MODERATION_WRITE_SCOPE),
            self.bids.then_some(BIDS_WRITE_SCOPE),
            self.draft_authoring.then_some(DRAFT_AUTHORING_SCOPE),
            self.shipping_rules.then_some(SHIPPING_RULES_WRITE_SCOPE),
            self.shipping_defaults.then_some(SHIPPING_DEFAULTS_SCOPE),
            self.inbox.then_some(INBOX_WRITE_SCOPE),
            self.profile.then_some(PROFILE_WRITE_SCOPE),
            self.following.then_some(FOLLOWING_WRITE_SCOPE),
            self.personal_blocks.then_some(PERSONAL_BLOCKS_WRITE_SCOPE),
            self.live_reminders.then_some(LIVE_REMINDER_WRITE_SCOPE),
            self.notification_state.then_some(NOTIFICATION_STATE_SCOPE),
            self.saved_searches.then_some(SAVED_SEARCH_WRITE_SCOPE),
        ]
        .into_iter()
        .flatten()
    }

    pub const fn intersection(self, other: Self) -> Self {
        Self {
            watchlist_membership: self.watchlist_membership && other.watchlist_membership,
            watchlist_management: self.watchlist_management && other.watchlist_management,
            listing_drafts: self.listing_drafts && other.listing_drafts,
            messages: self.messages && other.messages,
            message_reactions: self.message_reactions && other.message_reactions,
            public_chat: self.public_chat && other.public_chat,
            ratings: self.ratings && other.ratings,
            room_targets: self.room_targets && other.room_targets,
            room_safety: self.room_safety && other.room_safety,
            room_reactions: self.room_reactions && other.room_reactions,
            live_reports: self.live_reports && other.live_reports,
            room_moderation: self.room_moderation && other.room_moderation,
            bids: self.bids && other.bids,
            draft_authoring: self.draft_authoring && other.draft_authoring,
            shipping_rules: self.shipping_rules && other.shipping_rules,
            shipping_rules_read: self.shipping_rules_read && other.shipping_rules_read,
            shipping_defaults: self.shipping_defaults && other.shipping_defaults,
            inbox: self.inbox && other.inbox,
            profile: self.profile && other.profile,
            following: self.following && other.following,
            personal_blocks_read: self.personal_blocks_read && other.personal_blocks_read,
            personal_blocks: self.personal_blocks && other.personal_blocks,
            live_reminders: self.live_reminders && other.live_reminders,
            notification_state: self.notification_state && other.notification_state,
            saved_searches: self.saved_searches && other.saved_searches,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn scope_configuration_is_explicit_and_only_narrows() {
        for invalid in [
            "",
            " ",
            "watchlist:read",
            "unknown:write",
            "watchlist:write watchlist:write",
        ] {
            assert!(AccountWritePermissions::from_scopes(invalid).is_err());
        }
        let both = AccountWritePermissions::from_scopes("livereminders:write watchlist:write")
            .unwrap_or_else(|e| panic!("{e}"));
        let one = AccountWritePermissions::from_scopes("livereminders:write")
            .unwrap_or_else(|e| panic!("{e}"));
        assert_eq!(both.intersection(one), one);
        assert_eq!(one.scopes().collect::<Vec<_>>(), ["livereminders:write"]);
    }
}
