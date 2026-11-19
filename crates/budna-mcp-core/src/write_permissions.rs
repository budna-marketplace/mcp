//! Deployment-side write families. An enabled permission still needs a current customer grant.

use crate::ConfigError;

pub const INBOX_WRITE_SCOPE: &str = "inbox:write";
pub const PROFILE_WRITE_SCOPE: &str = "profile:write";
pub const FOLLOWING_WRITE_SCOPE: &str = "following:write";
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
    pub inbox: bool,
    pub profile: bool,
    pub following: bool,
    pub live_reminders: bool,
    pub notification_state: bool,
    pub saved_searches: bool,
}

impl AccountWritePermissions {
    pub const NONE: Self = Self {
        watchlist_membership: false,
        watchlist_management: false,
        listing_drafts: false,
        inbox: false,
        profile: false,
        following: false,
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
                INBOX_WRITE_SCOPE => &mut permissions.inbox,
                PROFILE_WRITE_SCOPE => &mut permissions.profile,
                FOLLOWING_WRITE_SCOPE => &mut permissions.following,
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

    pub fn scopes(self) -> impl Iterator<Item = &'static str> {
        [
            self.watchlist_membership.then_some(WATCHLIST_WRITE_SCOPE),
            self.watchlist_management.then_some(WATCHLIST_MANAGE_SCOPE),
            self.listing_drafts.then_some(DRAFT_CREATE_SCOPE),
            self.inbox.then_some(INBOX_WRITE_SCOPE),
            self.profile.then_some(PROFILE_WRITE_SCOPE),
            self.following.then_some(FOLLOWING_WRITE_SCOPE),
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
            inbox: self.inbox && other.inbox,
            profile: self.profile && other.profile,
            following: self.following && other.following,
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
