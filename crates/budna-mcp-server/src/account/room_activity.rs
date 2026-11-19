use super::*;
use crate::{
    output::room_activity::{RoomModerationViewOutput, RoomReceiptOutput, RoomTargetsOutput},
    params::{messages::RoomMessageParams, room_activity::*},
};
use budna_mcp_client::RoomOperation;

#[tool_router(router=room_targets_router,vis="pub(super)")]
impl AccountMcpServer {
    #[tool(
        title = "Read public reaction targets and my selections",
        description = "Read a bounded page of eligible room message and bid IDs with aggregate counts and your choices under chatrooms:read. Bidder identities and private maximums are excluded. Choose an exact target before reacting.",
        annotations(
            read_only_hint = true,
            destructive_hint = false,
            idempotent_hint = true,
            open_world_hint = true
        )
    )]
    pub async fn list_my_room_reaction_targets(
        &self,
        Parameters(i): Parameters<SafeToolParams<RoomTargetsParams>>,
        context: RequestContext<RoleServer>,
    ) -> Result<Json<RoomTargetsOutput>, CallToolResult> {
        const OP: &str = "list_my_room_reaction_targets";
        let i = i.parse().map_err(|e| input_error(OP, &e))?;
        let access = request_access(&context, OP)?;
        self.execute(
            OP,
            self.client.room_targets(
                &access,
                &i.room_id,
                i.limit.unwrap_or(20),
                i.before_message_id,
                i.before_bid_id,
            ),
        )
        .await
    }
}

#[tool_router(router=room_safety_router,vis="pub(super)")]
impl AccountMcpServer {
    #[tool(
        title = "Read my room moderation state",
        description = "Read the public IDs and ban state for participants in one room you own with roommoderation:read. The ban scope identifies whether it applies across your live shop rooms.",
        annotations(
            read_only_hint = true,
            destructive_hint = false,
            idempotent_hint = true,
            open_world_hint = true
        )
    )]
    pub async fn list_my_room_moderation_participants(
        &self,
        Parameters(i): Parameters<SafeToolParams<RoomModerationParticipantsParams>>,
        context: RequestContext<RoleServer>,
    ) -> Result<Json<RoomModerationViewOutput>, CallToolResult> {
        const OP: &str = "list_my_room_moderation_participants";
        let i = i.parse().map_err(|e| input_error(OP, &e))?;
        let access = request_access(&context, OP)?;
        self.execute(
            OP,
            self.client.room_moderation_state(
                &access,
                &i.room_id,
                i.limit.unwrap_or(20),
                i.after_user_id.as_deref(),
            ),
        )
        .await
    }
}

#[tool_router(router=room_reaction_router,vis="pub(super)")]
impl AccountMcpServer {
    #[tool(
        title = "Set my reaction to a public message or bid",
        description = "With roomreactions:write, set or remove the chosen emoji on the exact public post or eligible bid. This changes a public tally. The desired state is explicit.",
        annotations(
            read_only_hint = false,
            destructive_hint = true,
            idempotent_hint = false,
            open_world_hint = true
        )
    )]
    pub async fn set_my_room_reaction(
        &self,
        Parameters(i): Parameters<SafeToolParams<RoomReactParams>>,
        context: RequestContext<RoleServer>,
    ) -> Result<Json<RoomReceiptOutput>, CallToolResult> {
        const OP: &str = "set_my_room_reaction";
        let i = i.parse().map_err(|e| input_error(OP, &e))?;
        let access = request_access(&context, OP)?;
        self.execute(
            OP,
            self.client.change_room_activity(
                &access,
                &i.room_id,
                RoomOperation::Reaction {
                    target_kind: i.target_kind,
                    target_id: i.target_id,
                    emoji: i.emoji,
                    reacted: i.reacted,
                },
                &uuid::Uuid::new_v4().to_string(),
            ),
        )
        .await
    }
}

#[tool_router(router=live_report_router,vis="pub(super)")]
impl AccountMcpServer {
    #[tool(
        title = "Report one post in a live shop",
        description = "Submit one real public shop-stream post for Budna staff review under livereports:write. The post identifies its author. A receipt acknowledges filing.",
        annotations(
            read_only_hint = false,
            destructive_hint = false,
            idempotent_hint = false,
            open_world_hint = true
        )
    )]
    pub async fn report_live_chat_post(
        &self,
        Parameters(i): Parameters<SafeToolParams<RoomMessageParams>>,
        context: RequestContext<RoleServer>,
    ) -> Result<Json<RoomReceiptOutput>, CallToolResult> {
        const OP: &str = "report_live_chat_post";
        let i = i.parse().map_err(|e| input_error(OP, &e))?;
        let access = request_access(&context, OP)?;
        self.execute(
            OP,
            self.client.change_room_activity(
                &access,
                &i.room_id,
                RoomOperation::ReportPost {
                    message_id: i.message_id,
                },
                &uuid::Uuid::new_v4().to_string(),
            ),
        )
        .await
    }
}

#[tool_router(router=room_moderation_router,vis="pub(super)")]
impl AccountMcpServer {
    #[tool(
        title = "Pin or unpin one public room message",
        description = "With roommoderation:write, set or remove a pin on the selected message in a room you moderate. Pinning can replace the current pinned message.",
        annotations(
            read_only_hint = false,
            destructive_hint = true,
            idempotent_hint = false,
            open_world_hint = true
        )
    )]
    pub async fn set_my_room_message_pinned(
        &self,
        Parameters(i): Parameters<SafeToolParams<RoomPinParams>>,
        context: RequestContext<RoleServer>,
    ) -> Result<Json<RoomReceiptOutput>, CallToolResult> {
        const OP: &str = "set_my_room_message_pinned";
        let i = i.parse().map_err(|e| input_error(OP, &e))?;
        let access = request_access(&context, OP)?;
        self.execute(
            OP,
            self.client.change_room_activity(
                &access,
                &i.room_id,
                RoomOperation::Pin {
                    message_id: i.message_id,
                    pinned: i.pinned,
                },
                &uuid::Uuid::new_v4().to_string(),
            ),
        )
        .await
    }

    #[tool(
        title = "Hide a visible post in my public room",
        description = "Hide the selected post in its public room under roommoderation:write. Explain that the ordinary room controls cannot undo hiding before submitting.",
        annotations(
            read_only_hint = false,
            destructive_hint = true,
            idempotent_hint = false,
            open_world_hint = true
        )
    )]
    pub async fn hide_my_room_message(
        &self,
        Parameters(i): Parameters<SafeToolParams<RoomMessageParams>>,
        context: RequestContext<RoleServer>,
    ) -> Result<Json<RoomReceiptOutput>, CallToolResult> {
        const OP: &str = "hide_my_room_message";
        let i = i.parse().map_err(|e| input_error(OP, &e))?;
        let access = request_access(&context, OP)?;
        self.execute(
            OP,
            self.client.change_room_activity(
                &access,
                &i.room_id,
                RoomOperation::Hide {
                    message_id: i.message_id,
                },
                &uuid::Uuid::new_v4().to_string(),
            ),
        )
        .await
    }

    #[tool(
        title = "Report a participant in my room",
        description = "Submit an eligible participant in the room you moderate for staff review under roommoderation:write. Select a post by that participant in the listing or showcase room. For a shop stream use report_live_chat_post.",
        annotations(
            read_only_hint = false,
            destructive_hint = false,
            idempotent_hint = false,
            open_world_hint = true
        )
    )]
    pub async fn report_my_room_participant(
        &self,
        Parameters(i): Parameters<SafeToolParams<RoomParticipantReportParams>>,
        context: RequestContext<RoleServer>,
    ) -> Result<Json<RoomReceiptOutput>, CallToolResult> {
        const OP: &str = "report_my_room_participant";
        let i = i.parse().map_err(|e| input_error(OP, &e))?;
        let access = request_access(&context, OP)?;
        self.execute(
            OP,
            self.client.change_room_activity(
                &access,
                &i.room_id,
                RoomOperation::ReportParticipant {
                    user_id: &i.user_id,
                    message_id: i.message_id,
                },
                &uuid::Uuid::new_v4().to_string(),
            ),
        )
        .await
    }

    #[tool(
        title = "Ban a participant from my room",
        description = "Ban the selected participant under roommoderation:write. Check the ban scope first. A live shop ban applies across that seller’s live chats.",
        annotations(
            read_only_hint = false,
            destructive_hint = true,
            idempotent_hint = false,
            open_world_hint = true
        )
    )]
    pub async fn ban_my_room_participant(
        &self,
        Parameters(i): Parameters<SafeToolParams<RoomMemberParams>>,
        context: RequestContext<RoleServer>,
    ) -> Result<Json<RoomReceiptOutput>, CallToolResult> {
        const OP: &str = "ban_my_room_participant";
        let i = i.parse().map_err(|e| input_error(OP, &e))?;
        let access = request_access(&context, OP)?;
        self.execute(
            OP,
            self.client.change_room_activity(
                &access,
                &i.room_id,
                RoomOperation::Ban {
                    user_id: &i.user_id,
                },
                &uuid::Uuid::new_v4().to_string(),
            ),
        )
        .await
    }

    #[tool(
        title = "Remove a ban from my room",
        description = "With roommoderation:write, remove the selected participant’s ban from your live shop chats. Check the room’s ban scope first.",
        annotations(
            read_only_hint = false,
            destructive_hint = true,
            idempotent_hint = false,
            open_world_hint = true
        )
    )]
    pub async fn unban_my_room_participant(
        &self,
        Parameters(i): Parameters<SafeToolParams<RoomMemberParams>>,
        context: RequestContext<RoleServer>,
    ) -> Result<Json<RoomReceiptOutput>, CallToolResult> {
        const OP: &str = "unban_my_room_participant";
        let i = i.parse().map_err(|e| input_error(OP, &e))?;
        let access = request_access(&context, OP)?;
        self.execute(
            OP,
            self.client.change_room_activity(
                &access,
                &i.room_id,
                RoomOperation::Unban {
                    user_id: &i.user_id,
                },
                &uuid::Uuid::new_v4().to_string(),
            ),
        )
        .await
    }
}
