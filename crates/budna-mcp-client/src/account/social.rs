use super::{
    AccountApiClient, AccountWatchlistActionStatus, ClientError, DelegatedAccess,
    FOLLOWING_READ_SCOPE, FOLLOWING_WRITE_SCOPE, PROFILE_READ_SCOPE, PROFILE_WRITE_SCOPE,
    actions::valid_action_text, invalid_input, invalid_response, is_action_id, is_user_id,
};
use reqwest::Method;
use serde::{Deserialize, Serialize};

#[derive(Debug, Deserialize)]
pub struct ProfileAuthoring {
    pub display_name: String,
    pub bio: Option<String>,
    pub public_profile: bool,
    pub revision: i64,
}
#[derive(Debug, Deserialize)]
pub struct ProfileEditReceipt {
    pub action_id: String,
    pub status: AccountWatchlistActionStatus,
}
#[derive(Debug, Deserialize)]
pub struct FollowState {
    pub user_id: String,
    pub username: String,
    pub following: bool,
}
#[derive(Debug, Deserialize)]
pub struct FollowReceipt {
    pub action_id: String,
    pub status: AccountWatchlistActionStatus,
    pub following: bool,
}

pub(super) fn is_account_username(s: &str) -> bool {
    s.len() <= 64
        && s.chars().next().is_some_and(char::is_alphabetic)
        && s.chars().all(|c| c.is_alphanumeric() || c == '_')
}

impl AccountApiClient {
    pub async fn profile_authoring(
        &self,
        access: &DelegatedAccess,
    ) -> Result<ProfileAuthoring, ClientError> {
        const OP: &str = "get_my_profile_authoring";
        self.require_scope(access, PROFILE_READ_SCOPE, OP)?;
        if !self.write_permissions().profile {
            return Err(invalid_input(OP, "Profile editing is unavailable"));
        }
        let result: ProfileAuthoring = self
            .get(OP, "mcp/account/profile/authoring", access, &[])
            .await?;
        if result.revision < 1
            || result.display_name.len() > 4096
            || result.bio.as_ref().is_some_and(|s| s.len() > 8192)
        {
            return Err(invalid_response(OP));
        }
        Ok(result)
    }

    pub async fn edit_public_profile(
        &self,
        access: &DelegatedAccess,
        revision: i64,
        display_name: Option<&str>,
        bio: Option<&str>,
        key: &str,
    ) -> Result<ProfileEditReceipt, ClientError> {
        const OP: &str = "update_my_public_profile";
        if revision < 1
            || (display_name.is_none() && bio.is_none())
            || display_name.is_some_and(|s| s.len() > 255 || !valid_action_text(s, 100))
            || bio.is_some_and(|s| s.len() > 1000 || (!s.is_empty() && !valid_action_text(s, 1000)))
        {
            return Err(invalid_input(OP, "Invalid profile revision, name or bio"));
        }
        #[derive(Serialize)]
        struct Edit<'a> {
            expected_revision: i64,
            #[serde(skip_serializing_if = "Option::is_none")]
            display_name: Option<&'a str>,
            #[serde(skip_serializing_if = "Option::is_none")]
            bio: Option<&'a str>,
        }
        let request = self
            .action_request(
                OP,
                PROFILE_WRITE_SCOPE,
                access,
                Method::PATCH,
                "mcp/account/profile",
                key,
            )?
            .json(&Edit {
                expected_revision: revision,
                display_name,
                bio,
            });
        self.direct_receipt(OP, request, |r: &ProfileEditReceipt| {
            is_action_id(&r.action_id)
        })
        .await
    }

    pub async fn following_state(
        &self,
        access: &DelegatedAccess,
        username: &str,
    ) -> Result<FollowState, ClientError> {
        const OP: &str = "get_my_following_state";
        self.require_scope(access, FOLLOWING_READ_SCOPE, OP)?;
        if !self.write_permissions().following || !is_account_username(username) {
            return Err(invalid_input(
                OP,
                "Following is unavailable or the handle is invalid",
            ));
        }
        let result: FollowState = self
            .get(
                OP,
                &format!("mcp/account/following/{username}"),
                access,
                &[],
            )
            .await?;
        if result.username != username || !is_user_id(&result.user_id) {
            return Err(invalid_response(OP));
        }
        Ok(result)
    }

    pub async fn set_following(
        &self,
        access: &DelegatedAccess,
        username: &str,
        user_id: &str,
        following: bool,
        key: &str,
    ) -> Result<FollowReceipt, ClientError> {
        const OP: &str = "set_my_following";
        if !is_account_username(username) || !is_user_id(user_id) {
            return Err(invalid_input(OP, "Invalid public target reference"));
        }
        let request = self
            .action_request(
                OP,
                FOLLOWING_WRITE_SCOPE,
                access,
                Method::PUT,
                &format!("mcp/account/following/{username}"),
                key,
            )?
            .json(&serde_json::json!({"expected_user_id":user_id,"following":following}));
        self.direct_receipt(OP, request, |r: &FollowReceipt| {
            is_action_id(&r.action_id) && r.following == following
        })
        .await
    }
}
