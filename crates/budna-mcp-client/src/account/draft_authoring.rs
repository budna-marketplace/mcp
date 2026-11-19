use super::{
    AccountApiClient, AccountWatchlistActionStatus, ClientError, DRAFT_AUTHORING_SCOPE,
    DelegatedAccess, DraftReceipt, invalid_input, is_action_id,
};
use crate::is_listing_id;
use reqwest::Method;
impl AccountApiClient {
    pub async fn create_similar_draft(
        &self,
        access: &DelegatedAccess,
        source: &str,
        expected_revision: i64,
        request_id: &str,
    ) -> Result<DraftReceipt, ClientError> {
        const OP: &str = "create_my_similar_draft";
        if !is_listing_id(source) || expected_revision < 1 {
            return Err(invalid_input(OP, "Invalid source listing or revision"));
        }
        let request = self
            .action_request(
                OP,
                DRAFT_AUTHORING_SCOPE,
                access,
                Method::POST,
                &format!("mcp/account/listings/{source}/similar-draft"),
                request_id,
            )?
            .json(&serde_json::json!({"expected_revision":expected_revision}));
        self.direct_receipt(OP, request, |r: &DraftReceipt| {
            is_action_id(&r.action_id)
                && r.listing_id
                    .as_deref()
                    .is_none_or(|id| is_listing_id(id) && id != source)
                && (r.status != AccountWatchlistActionStatus::Executed || r.listing_id.is_some())
        })
        .await
    }
}

use super::{
    LISTINGS_READ_SCOPE,
    actions::valid_outgoing_text,
    draft_fields::{DraftFieldChanges, DraftFieldState, DraftVatChoice},
};
use serde::{Deserialize, Serialize};
use std::collections::HashSet;

#[derive(Debug, Deserialize, Serialize, schemars::JsonSchema)]
#[serde(
    tag = "kind",
    content = "value",
    rename_all = "snake_case",
    deny_unknown_fields
)]
pub enum DraftAttributeValue {
    Text(String),
    Options(Vec<String>),
    Number(String),
    Boolean(bool),
    Date(i64),
}
impl DraftAttributeValue {
    fn valid(&self, writing: bool) -> bool {
        let text = |s: &str| {
            if writing {
                valid_outgoing_text(s, 1000, true)
            } else {
                s.chars().count() <= 1000
            }
        };
        match self {
            Self::Text(s) => text(s),
            Self::Options(v) => {
                (1..=50).contains(&v.len())
                    && v.iter().all(|s| text(s))
                    && v.iter().collect::<HashSet<_>>().len() == v.len()
            }
            Self::Number(s) => {
                let d = s.strip_prefix('-').unwrap_or(s);
                let (whole, frac) = d.split_once('.').unwrap_or((d, ""));
                s.len() <= 30
                    && crate::is_exact_decimal(d)
                    && (whole == "0" || !whole.starts_with('0'))
                    && frac.len() <= 28
            }
            Self::Boolean(_) => true,
            Self::Date(v) => *v > 0,
        }
    }
}
#[derive(Debug, Deserialize, Serialize, schemars::JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct DraftAttribute {
    #[schemars(range(min = 1))]
    pub filter_definition_id: i32,
    pub value: DraftAttributeValue,
}
fn valid_attributes(attributes: &[DraftAttribute], writing: bool) -> bool {
    let mut unique = HashSet::new();
    attributes.len() <= 50
        && attributes.iter().all(|a| {
            a.filter_definition_id > 0
                && unique.insert(a.filter_definition_id)
                && a.value.valid(writing)
        })
}

#[derive(Debug, Deserialize)]
pub struct DraftVatState {
    pub vat_registered: bool,
    #[serde(deserialize_with = "Option::deserialize")]
    pub vat_treatment: Option<String>,
    #[serde(deserialize_with = "Option::deserialize")]
    pub default_vat_treatment: Option<String>,
}
#[derive(Debug, Deserialize)]
struct AttributeRecord {
    filter_definition_id: i32,
    value: AttributeRecordValue,
}
#[derive(Debug, Deserialize)]
#[serde(tag = "kind", content = "value", rename_all = "snake_case")]
enum AttributeRecordValue {
    Text(String),
    Options(Vec<String>),
    Number(String),
    Boolean(bool),
    Date(i64),
}
fn read_attributes<'de, D: serde::Deserializer<'de>>(
    d: D,
) -> Result<Vec<DraftAttribute>, D::Error> {
    Ok(Vec::<AttributeRecord>::deserialize(d)?
        .into_iter()
        .map(|a| DraftAttribute {
            filter_definition_id: a.filter_definition_id,
            value: match a.value {
                AttributeRecordValue::Text(v) => DraftAttributeValue::Text(v),
                AttributeRecordValue::Options(v) => DraftAttributeValue::Options(v),
                AttributeRecordValue::Number(v) => DraftAttributeValue::Number(v),
                AttributeRecordValue::Boolean(v) => DraftAttributeValue::Boolean(v),
                AttributeRecordValue::Date(v) => DraftAttributeValue::Date(v),
            },
        })
        .collect())
}
#[derive(Debug, Deserialize)]
pub struct DraftAuthoringState {
    pub listing_id: String,
    pub revision: i64,
    pub fields: DraftFieldState,
    #[serde(deserialize_with = "read_attributes")]
    pub attributes: Vec<DraftAttribute>,
    pub attributes_replaceable: bool,
    #[serde(deserialize_with = "Option::deserialize")]
    pub vat: Option<DraftVatState>,
    pub media_count: u32,
}
pub enum DraftEdit<'a> {
    Fields(&'a DraftFieldChanges),
    Attributes(&'a [DraftAttribute]),
    Vat(Option<DraftVatChoice>),
    Discard,
}
impl AccountApiClient {
    pub async fn draft_authoring(
        &self,
        access: &DelegatedAccess,
        id: &str,
    ) -> Result<DraftAuthoringState, ClientError> {
        const OP: &str = "get_my_draft_authoring";
        self.require_scope(access, LISTINGS_READ_SCOPE, OP)?;
        if !is_listing_id(id) || !self.write_permissions().draft_authoring {
            return Err(invalid_input(
                OP,
                "Invalid source or authoring is unavailable",
            ));
        }
        let result: DraftAuthoringState = self
            .get(
                OP,
                &format!("mcp/account/listings/{id}/draft-authoring"),
                access,
                &[],
            )
            .await?;
        let known_vat = |s: &str| {
            matches!(
                s,
                "vmb_used_goods" | "standard_25" | "reduced_12" | "reduced_6" | "exempt"
            )
        };
        if result.listing_id != id
            || result.revision < 1
            || !result.fields.valid()
            || !valid_attributes(&result.attributes, false)
            || result.media_count > 30
            || result.vat.as_ref().is_some_and(|v| {
                !v.vat_treatment.as_deref().is_none_or(known_vat)
                    || !v.default_vat_treatment.as_deref().is_none_or(known_vat)
            })
        {
            return Err(super::invalid_response(OP));
        }
        Ok(result)
    }
    pub async fn edit_draft(
        &self,
        access: &DelegatedAccess,
        id: &str,
        expected_revision: i64,
        edit: DraftEdit<'_>,
        key: &str,
    ) -> Result<DraftReceipt, ClientError> {
        let (op, method, suffix, body, valid) = match edit {
            DraftEdit::Fields(changes) => (
                "edit_my_listing_draft",
                Method::PATCH,
                "",
                serde_json::json!({"expected_revision":expected_revision,"changes":changes}),
                changes.valid(),
            ),
            DraftEdit::Attributes(attributes) => (
                "replace_my_draft_attributes",
                Method::PUT,
                "/attributes",
                serde_json::json!({"expected_revision":expected_revision,"attributes":attributes}),
                valid_attributes(attributes, true),
            ),
            DraftEdit::Vat(treatment) => (
                "set_my_draft_vat",
                Method::PUT,
                "/vat",
                serde_json::json!({"expected_revision":expected_revision,"vat_treatment":treatment}),
                true,
            ),
            DraftEdit::Discard => (
                "discard_my_listing_draft",
                Method::DELETE,
                "",
                serde_json::json!({"expected_revision":expected_revision}),
                true,
            ),
        };
        if !is_listing_id(id) || expected_revision < 1 || !valid {
            return Err(invalid_input(
                op,
                "Invalid draft fields, revision or listing",
            ));
        }
        let request = self
            .action_request(
                op,
                DRAFT_AUTHORING_SCOPE,
                access,
                method,
                &format!("mcp/account/listings/{id}/draft-authoring{suffix}"),
                key,
            )?
            .json(&body);
        self.direct_receipt(op, request, |r: &DraftReceipt| {
            is_action_id(&r.action_id)
                && r.listing_id.as_deref().is_none_or(|x| x == id)
                && (r.status != AccountWatchlistActionStatus::Executed || r.listing_id.is_some())
        })
        .await
    }
}
