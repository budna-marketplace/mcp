use super::{
    AccountApiClient, AccountWatchlistActionStatus, ClientError, DelegatedAccess,
    PERSONAL_BLOCKS_READ_SCOPE, PERSONAL_BLOCKS_WRITE_SCOPE, invalid_input, invalid_response,
    is_action_id, is_user_id,
};
use crate::is_public_username;
use reqwest::Method;
use serde::Deserialize;
use std::collections::BTreeSet;

#[derive(Debug, Deserialize)]
pub struct OutgoingBlock {
    pub user_id: String,
    pub username: String,
}
#[derive(Debug, Deserialize)]
pub struct OutgoingBlocks {
    pub items: Vec<OutgoingBlock>,
    pub next_after: Option<String>,
}
#[derive(Debug, Deserialize)]
pub struct PersonalBlockReceipt {
    pub action_id: String,
    pub user_id: String,
    pub blocked: bool,
    pub status: AccountWatchlistActionStatus,
}

impl AccountApiClient {
    pub async fn outgoing_blocks(
        &self,
        access: &DelegatedAccess,
        limit: u32,
        after: Option<&str>,
    ) -> Result<OutgoingBlocks, ClientError> {
        const OP: &str = "list_my_personal_blocks";
        self.require_scope(access, PERSONAL_BLOCKS_READ_SCOPE, OP)?;
        if !(1..=50).contains(&limit) || after.is_some_and(|s| !is_user_id(s)) {
            return Err(invalid_input(OP, "Invalid block page or cursor"));
        }
        let mut query = vec![("limit", limit.to_string())];
        if let Some(after) = after {
            query.push(("after", after.to_owned()));
        }
        let result: OutgoingBlocks = self
            .get(OP, "mcp/account/personal-blocks", access, &query)
            .await?;
        let mut ids = BTreeSet::new();
        if result.items.len() > limit as usize
            || result.items.iter().any(|item| {
                !is_user_id(&item.user_id)
                    || item.user_id == access.subject
                    || Some(item.user_id.as_str()) == after
                    || !ids.insert(&item.user_id)
                    || !is_public_username(&item.username)
            })
            || result.next_after.as_ref().is_some_and(|cursor| {
                result
                    .items
                    .last()
                    .is_none_or(|item| &item.user_id != cursor)
            })
        {
            return Err(invalid_response(OP));
        }
        Ok(result)
    }

    pub async fn set_personal_block(
        &self,
        access: &DelegatedAccess,
        id: &str,
        blocked: bool,
        key: &str,
    ) -> Result<PersonalBlockReceipt, ClientError> {
        const OP: &str = "set_my_personal_block";
        if !is_user_id(id) || id == access.subject {
            return Err(invalid_input(OP, "Select one other public account"));
        }
        let request = self
            .action_request(
                OP,
                PERSONAL_BLOCKS_WRITE_SCOPE,
                access,
                Method::PUT,
                &format!("mcp/account/personal-blocks/{id}"),
                key,
            )?
            .json(&serde_json::json!({"blocked":blocked}));
        self.direct_receipt(OP, request, |r: &PersonalBlockReceipt| {
            is_action_id(&r.action_id) && r.user_id == id && r.blocked == blocked
        })
        .await
    }
}
