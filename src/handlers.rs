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

// struct DisconnectHandler {
//     manager: Arc<songbird::Songbird>,
//     guild_id: GuildId,
// }

// #[async_trait::async_trait]
// impl EventHandler for DisconnectHandler {
//     async fn act(&self, ctx: &EventContext<'_>) -> Option<Event> {
//         if let EventContext::DriverDisconnect(_) = ctx {
//             // clean up — remove the call entirely so state is fresh
//             let _ = self.manager.remove(self.guild_id).await;
//         }
//         None
//     }
// }
