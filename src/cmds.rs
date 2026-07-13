use crate::prelude::*;

#[poise::command(prefix_command)]
pub async fn a(ctx: PrefixContext<'_>) -> Result<()> {
    let msg = ctx.msg;
    let args = ctx.args;

    let is_admin = {
        let admins = &ctx.data().admins;
        admins.contains(&msg.author.id)
    };
    if is_admin
        && try_whitelist_add(&ctx.data().whitelist, msg, args)
            .await
            .is_ok()
    {
        msg.react(&ctx.http(), '\u{2705}').await.map(|_| ())
    } else {
        msg.delete(&ctx.http()).await
    }
    .map_err(Into::into)
}

#[poise::command(prefix_command)]
pub async fn w(ctx: PrefixContext<'_>) -> Result<()> {
    let msg = ctx.msg;
    let args = ctx.args;

    //  determine if the server is whitelisted
    let whitelisted = {
        if let Some(id) = msg.guild_id {
            ctx.data().whitelist.lock().await.data().contains(&id)
        } else {
            false
        }
    };
    if whitelisted {
        if !args.is_empty() {
            bail!("Arguments were provided")
        }
        let image = ctx
            .data()
            .images
            .choose(&mut rand::rng())
            .ok_or_else(|| anyhow!("no available images."))?
            .clone();
        let response = msg.channel_id.send_message(&ctx.http(), image).await?;
        link_messages(&ctx.data().links, msg.id, response.id).await
    } else {
        msg.delete(&ctx.http()).await.map_err(Into::into)
    }
}

#[poise::command(prefix_command, guild_only, aliases("j", "hop-in"))]
pub async fn join(ctx: PrefixContext<'_>) -> Result<()> {
    _whitelist(ctx)?;
    join_helper(ctx).await?;
    Ok(())
}

#[poise::command(prefix_command, guild_only, aliases("quit", "dip"))]
pub async fn leave(ctx: PrefixContext<'_>) -> Result<()> {
    {
        let call = get_call(ctx).await?;
        call.lock().await.queue().stop();
    }
    let guild_id = ctx.msg.guild_id.ok_or_else(|| anyhow!("not in a guild"))?;
    let manager = songbird::get(ctx.serenity_context())
        .await
        .ok_or_else(|| anyhow!("songbird not registered"))?;
    manager.remove(guild_id).await?;
    Ok(())
}

#[poise::command(prefix_command, guild_only, aliases("piss"))]
pub async fn play(ctx: PrefixContext<'_>, query: Vec<String>) -> Result<()> {
    play_helper(ctx, query, false).await
}

#[poise::command(prefix_command, guild_only, aliases("playtop"))]
pub async fn playnext(ctx: PrefixContext<'_>, query: Vec<String>) -> Result<()> {
    play_helper(ctx, query, true).await
}

#[poise::command(prefix_command, guild_only, aliases("next", "nah"))]
pub async fn skip(ctx: PrefixContext<'_>) -> Result<()> {
    let call = get_call(ctx).await?;
    let call = call.lock().await;
    let queue = call.queue();

    if let Some(track) = queue.current() {
        let new_len = queue.len().saturating_sub(1);
        queue.skip()?;
        let song = track.data::<TrackInfo>().as_ref().clone().into_inner();
        ctx.send(embed(
            ctx,
            format!("Skipped: {}", song.title),
            Some(EmbedItem::Track(Box::new(song))),
            true,
            Some(new_len),
        ))
        .await?;
    } else {
        ctx.send(note(ctx, None, "Nothing is playing.")).await?;
    }
    Ok(())
}

#[poise::command(prefix_command, guild_only, aliases("stop"))]
pub async fn pause(ctx: PrefixContext<'_>) -> Result<()> {
    let call = get_call(ctx).await?;
    let call = call.lock().await;
    let queue = call.queue();

    if queue.is_empty() {
        ctx.send(note(ctx, None, "Nothing is playing.")).await?;
    } else {
        queue.pause()?;
        ctx.send(note(ctx, None, "Paused.")).await?;
    }
    Ok(())
}

#[poise::command(prefix_command, guild_only, aliases("continue"))]
pub async fn resume(ctx: PrefixContext<'_>) -> Result<()> {
    let call = get_call(ctx).await?;
    let call = call.lock().await;
    let queue = call.queue();

    if queue.is_empty() {
        ctx.send(note(ctx, None, "Nothing is playing.")).await?;
    } else {
        queue.resume()?;
        ctx.send(note(ctx, None, "Resumed.")).await?;
    }
    Ok(())
}

#[poise::command(prefix_command, guild_only, aliases("clean"))]
pub async fn clear(ctx: PrefixContext<'_>) -> Result<()> {
    let call = get_call(ctx).await?;
    let call = call.lock().await;
    let queue = call.queue();

    if queue.is_empty() {
        ctx.send(note(ctx, None, "There is no queue.")).await?;
        return Ok(());
    }

    // determine if there is currently a song playing
    let mut is_playing = false;
    if let Some(handle) = queue.current()
        && let Ok(info) = handle.get_info().await
        && let PlayMode::Play = info.playing
    {
        is_playing = true
    }
    // clear the queue, stopping removed tracks so they don't leak in the driver
    queue.modify_queue(|q| {
        let kept = usize::from(is_playing);
        for track in q.drain(kept..) {
            let _ = track.stop();
        }
    });
    ctx.send(note(ctx, None, "Queue has been cleared.")).await?;
    Ok(())
}

#[poise::command(prefix_command, guild_only, aliases("q"))]
pub async fn queue(ctx: PrefixContext<'_>) -> Result<()> {
    let call = get_call(ctx).await?;
    let call = call.lock().await;
    let queue = call.queue();

    ctx.send(embed(
        ctx,
        "The Queue.",
        Some(EmbedItem::Queue(queue.current_queue())),
        true,
        Some(queue.len()),
    ))
    .await?;

    Ok(())
}

// wow just wow
#[poise::command(prefix_command, guild_only, aliases("rm"))]
pub async fn remove(ctx: PrefixContext<'_>, track_index: Option<usize>) -> Result<()> {
    let call = get_call(ctx).await?;
    let call = call.lock().await;
    let queue = call.queue();

    if let Some(index) = track_index {
        if let Some(track) = queue.current() {
            if index == 0 {
                let new_len = queue.len().saturating_sub(1);
                queue.skip()?;
                let song = track.data::<TrackInfo>().as_ref().clone().into_inner();
                ctx.send(embed(
                    ctx,
                    format!("Skipped: {}", song.title),
                    Some(EmbedItem::Track(Box::new(song))),
                    true,
                    Some(new_len),
                ))
                .await?;
            } else if let Some(t) = queue.dequeue(index) {
                t.stop()?;
                let song = t.data::<TrackInfo>().as_ref().clone().into_inner();
                let msg = format!("Removed: {}", song.title);
                ctx.send(note(ctx, Some(EmbedItem::Track(Box::new(song))), &msg))
                    .await?;
            } else {
                bail!("No track at that position.")
            }
        } else {
            ctx.send(note(ctx, None, "Nothing is playing.")).await?;
        }
    } else {
        bail!("Please provide the track position.")
    }
    Ok(())
}

#[poise::command(prefix_command, guild_only, aliases("mix"))]
pub async fn shuffle(ctx: PrefixContext<'_>) -> Result<()> {
    let call = get_call(ctx).await?;
    let call = call.lock().await;
    let queue = call.queue();

    // randomly shuffle the queue
    queue.modify_queue(|q| {
        if q.len() > 1 {
            q.make_contiguous()[1..].shuffle(&mut rand::rng());
        }
    });

    ctx.say("Queue shuffled.").await?;
    Ok(())
}
