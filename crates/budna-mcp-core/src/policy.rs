use crate::AccountWritePermissions;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
#[non_exhaustive]
pub enum ToolCapability {
    PublicExplore,
    AccountRead,
    PrivateWatchlistWrite,
    PrivateWatchlistManagement,
    ListingDraftCreate,
    InboxWrite,
    ProfileWrite,
    FollowingWrite,
    LiveReminderWrite,
    NotificationStateWrite,
    SavedSearchWrite,
}

/// The concrete capability profile exposed by a server instance.
///
/// Additional profiles must use their own router and authorization boundary;
/// they must not disable individual tools behind an already-advertised static
/// router.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
#[non_exhaustive]
pub enum ToolPolicy {
    PublicExplore,
    AccountRead,
    AccountWatchlistPilot,
    AccountWatchlistManagement,
    AccountListingDraftPilot,
    AccountActions(AccountWritePermissions),
}

impl ToolPolicy {
    pub const fn public_explore() -> Self {
        Self::PublicExplore
    }

    pub const fn write_permissions(self) -> AccountWritePermissions {
        let none = AccountWritePermissions::NONE;
        match self {
            Self::PublicExplore | Self::AccountRead => none,
            Self::AccountWatchlistPilot => AccountWritePermissions {
                watchlist_membership: true,
                ..none
            },
            Self::AccountWatchlistManagement => AccountWritePermissions {
                watchlist_membership: true,
                watchlist_management: true,
                ..none
            },
            Self::AccountListingDraftPilot => AccountWritePermissions {
                listing_drafts: true,
                ..none
            },
            Self::AccountActions(permissions) => permissions,
        }
    }

    pub const fn allows(&self, capability: ToolCapability) -> bool {
        let writes = self.write_permissions();
        match capability {
            ToolCapability::PublicExplore => true,
            ToolCapability::AccountRead => !matches!(self, Self::PublicExplore),
            ToolCapability::PrivateWatchlistWrite => writes.watchlist_membership,
            ToolCapability::PrivateWatchlistManagement => writes.watchlist_management,
            ToolCapability::ListingDraftCreate => writes.listing_drafts,
            ToolCapability::SavedSearchWrite => writes.saved_searches,
            ToolCapability::NotificationStateWrite => writes.notification_state,
            ToolCapability::InboxWrite => writes.inbox,
            ToolCapability::ProfileWrite => writes.profile,
            ToolCapability::FollowingWrite => writes.following,
            ToolCapability::LiveReminderWrite => writes.live_reminders,
        }
    }
}

impl Default for ToolPolicy {
    fn default() -> Self {
        Self::public_explore()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn current_policy_enables_public_exploration() {
        let policy = ToolPolicy::public_explore();

        assert!(policy.allows(ToolCapability::PublicExplore));
    }
}
