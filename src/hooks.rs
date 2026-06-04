use crate::prelude::*;

pub async fn on_error(error: FrameworkError<'_>) {
    match error {
        poise::FrameworkError::Command { error, ctx, .. } => {
            let error = format!("{:#}", error);

            tracing::error!(
                command = ctx.command().name,
                user = ctx.author().name,
                error = error,
                "command error"
            );
            let embed = CreateEmbed::new()
                .title("An error has ocurred.")
                .description(error);
            let reply = CreateReply::default().embed(embed).reply(true);

            if let Err(e) = ctx.send(reply).await {
                tracing::error!(error = %e, "failed to send error reply");
            }
        }
        other => {
            tracing::error!(error = %other, "framework error");
        }
    }
}
