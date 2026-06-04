use crate::prelude::*;

pub struct Handler;

#[serenity::async_trait]
impl EventHandler for Handler {
    async fn message_delete(
        &self,
        ctx: SerenityContext,
        channel_id: ChannelId,
        deleted_message_id: MessageId,
        _: Option<GuildId>,
    ) {
        //  don't really care about this
        _ = delete_if_linked(&ctx, channel_id, &deleted_message_id).await;
    }
}
