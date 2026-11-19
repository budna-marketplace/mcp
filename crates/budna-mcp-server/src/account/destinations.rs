use super::*;
use rmcp::schemars;
use serde::{Deserialize, Serialize};

#[derive(Clone, Copy, Debug, Deserialize, Serialize, schemars::JsonSchema)]
#[serde(rename_all = "snake_case")]
enum FirstPartyTask {
    VerifyIdentity,
    VerifyCompany,
    ManageConnectedApps,
    ManageAddresses,
    ManageWallet,
    ManageCompany,
    ManageSubscription,
    ManageNotifications,
    ManagePrivacyCookies,
    ManageListingMedia,
    ManageShipping,
    ManageOrders,
    ManageDisputes,
    ManageStreams,
    ManageInventoryTags,
}

impl FirstPartyTask {
    const fn path(self) -> &'static str {
        match self {
            Self::VerifyIdentity => "/account/verification",
            Self::VerifyCompany => "/account/wallet?setup=payouts",
            Self::ManageConnectedApps => "/account/security",
            Self::ManageAddresses => "/account/addresses",
            Self::ManageWallet => "/account/wallet",
            Self::ManageCompany => "/account/company",
            Self::ManageSubscription => "/account/plan",
            Self::ManageNotifications => "/account/notifications",
            Self::ManagePrivacyCookies => "/account/cookies",
            Self::ManageListingMedia => "/account/my-listings",
            Self::ManageShipping => "/account/shipping",
            Self::ManageOrders => "/account/orders",
            Self::ManageDisputes => "/account/disputes",
            Self::ManageStreams => "/account/streams",
            Self::ManageInventoryTags => "/account/tags",
        }
    }
}

#[derive(Debug, Deserialize, schemars::JsonSchema)]
#[serde(deny_unknown_fields)]
struct DestinationParams {
    task: FirstPartyTask,
}

#[derive(Debug, Serialize, schemars::JsonSchema)]
pub(super) struct FirstPartyDestination {
    task: FirstPartyTask,
    web_url: String,
}

#[tool_router(router=destinations_router,vis="pub(super)")]
impl AccountMcpServer {
    #[tool(
        title = "Continue an account task on Budna web",
        description = "Return the fixed first-party Budna web page for a customer task such as identity verification, addresses, payment, media, security, notifications or connected apps. The person completes the task there. Opening a page never submits or approves an action.",
        annotations(
            read_only_hint = true,
            destructive_hint = false,
            idempotent_hint = true,
            open_world_hint = false
        )
    )]
    async fn get_budna_account_destination(
        &self,
        Parameters(input): Parameters<SafeToolParams<DestinationParams>>,
        context: RequestContext<RoleServer>,
    ) -> Result<Json<FirstPartyDestination>, CallToolResult> {
        const OP: &str = "get_budna_account_destination";
        let input = input.parse().map_err(|e| input_error(OP, &e))?;
        request_access(&context, OP)?;
        Ok(Json(FirstPartyDestination {
            task: input.task,
            web_url: format!("{}{}", self.public_urls.listing_origin(), input.task.path()),
        }))
    }
}
