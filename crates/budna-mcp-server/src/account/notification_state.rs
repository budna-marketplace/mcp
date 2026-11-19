use super::*;
use crate::{
    output::{NotificationReadBatchOutput, NotificationReadReceiptOutput},
    params::NotificationReadParams,
};

#[tool_router(router = notification_state_router, vis = "pub(super)")]
impl AccountMcpServer {
    #[tool(
        title = "Preview ordinary notifications to mark read",
        description = "Read up to 50 currently eligible, unread ordinary notification IDs with notifications:read. has_more indicates this is a partial batch. The preview itself changes no read state.",
        annotations(
            read_only_hint = true,
            destructive_hint = false,
            idempotent_hint = true,
            open_world_hint = true
        )
    )]
    pub async fn get_my_notification_read_batch(
        &self,
        Parameters(input): Parameters<SafeToolParams<EmptyAccountParams>>,
        context: RequestContext<RoleServer>,
    ) -> Result<Json<NotificationReadBatchOutput>, CallToolResult> {
        input
            .parse()
            .map_err(|e| input_error(NOTIFICATION_BATCH_TOOL, &e))?;
        let access = request_access(&context, NOTIFICATION_BATCH_TOOL)?;
        self.execute(
            NOTIFICATION_BATCH_TOOL,
            self.client.notification_read_batch(&access),
        )
        .await
    }

    #[tool(
        title = "Mark selected ordinary notifications read",
        description = "On the customer's explicit request, mark the listed ordinary notification IDs read, up to 50 at once. Requires notificationstate:write. Security and connected-app notices are protected. An ineligible ID rejects the whole batch. Review another batch only on a new request.",
        annotations(
            read_only_hint = false,
            destructive_hint = true,
            idempotent_hint = false,
            open_world_hint = true
        )
    )]
    pub async fn mark_my_notifications_read(
        &self,
        Parameters(input): Parameters<SafeToolParams<NotificationReadParams>>,
        context: RequestContext<RoleServer>,
    ) -> Result<Json<NotificationReadReceiptOutput>, CallToolResult> {
        let args = input
            .parse()
            .map_err(|e| input_error(NOTIFICATION_READ_TOOL, &e))?;
        let access = request_access(&context, NOTIFICATION_READ_TOOL)?;
        self.execute(
            NOTIFICATION_READ_TOOL,
            self.client.mark_notifications_read(
                &access,
                &args.notification_ids,
                &uuid::Uuid::new_v4().to_string(),
            ),
        )
        .await
    }
}
