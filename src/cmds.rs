use crate::prelude::*;

#[poise::command(prefix_command)]
pub async fn a(ctx: PrefixContext<'_, Data, anyhow::Error>) -> Result<()> {
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
pub async fn w(ctx: PrefixContext<'_, Data, anyhow::Error>) -> Result<()> {
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
pub async fn join(ctx: PrefixContext<'_, Data, anyhow::Error>) -> Result<()> {
    if ctx
        .guild_id()
        .ok_or_else(|| anyhow!("allowed in guilds only."))?
        == GuildId::new(684429201398562855)
    {
        bail!("this guild isn't whitelisted.")
    }

    join_helper(ctx).await?;
    Ok(())
}

#[poise::command(prefix_command, guild_only, aliases("quit", "dip"))]
pub async fn leave(ctx: PrefixContext<'_, Data, anyhow::Error>) -> Result<()> {
    {
        let call = get_call(ctx).await?;
        call.lock().await.queue().stop();
    }
    let guild_id = ctx.msg.guild_id.ok_or_else(|| anyhow!("not in a guild"))?;
    let manager = songbird::get(ctx.serenity_context())
        .await
        .ok_or_else(|| anyhow!("songbird not registered"))?;
    manager.remove(guild_id).await?;
    ctx.send(note("left.")).await?;
    Ok(())
}

#[poise::command(prefix_command, guild_only, aliases("piss"))]
pub async fn play(ctx: PrefixContext<'_, Data, anyhow::Error>, query: Vec<String>) -> Result<()> {
    if ctx
        .guild_id()
        .ok_or_else(|| anyhow!("allowed in guilds only."))?
        == GuildId::new(684429201398562855)
    {
        bail!("this guild isn't whitelisted.")
    }

    let query = query
        .into_iter()
        .map(|s| s.trim().to_string())
        .collect::<Vec<String>>()
        .join(" ");

    let song = ctx.data.stf.search(&query).await?;
    let input = ctx.data().stf.stream(song.uri.clone()).await?;
    let len = {
        let track = Track::new_with_data(input, Arc::new(TrackInfo::new(song.clone())));
        let call = join_helper(ctx).await?;
        let mut call = call.lock().await;
        call.enqueue(track).await;
        call.queue().len()
    };
    ctx.send(embed(
        "Added to Queue.",
        Some(EmbedMessage::Song(Box::new(song))),
        Some(len),
    ))
    .await?;

    Ok(())
}

#[poise::command(prefix_command, guild_only, aliases("playtop"))]
pub async fn playnext(
    ctx: PrefixContext<'_, Data, anyhow::Error>,
    query: Vec<String>,
) -> Result<()> {
    if ctx
        .guild_id()
        .ok_or_else(|| anyhow!("allowed in guilds only."))?
        == GuildId::new(684429201398562855)
    {
        bail!("this guild isn't whitelisted.")
    }

    let query = query
        .into_iter()
        .map(|s| s.trim().to_string())
        .collect::<Vec<String>>()
        .join(" ");

    let song = ctx.data.stf.search(&query).await?;
    let input = ctx.data().stf.stream(song.uri.clone()).await?;
    let len = {
        let track = Track::new_with_data(input, Arc::new(TrackInfo::new(song.clone())));
        let call = join_helper(ctx).await?;
        let mut call = call.lock().await;
        call.enqueue(track).await;
        call.queue().modify_queue(|q| {
            if q.len() > 1
                && let Some(last) = q.pop_back()
            {
                q.insert(1, last);
            }
        });
        call.queue().len()
    };
    ctx.send(embed(
        "Playing next.",
        Some(EmbedMessage::Song(Box::new(song))),
        Some(len),
    ))
    .await?;
    Ok(())
}

#[poise::command(prefix_command, guild_only, aliases("nah"))]
pub async fn skip(ctx: PrefixContext<'_, Data, anyhow::Error>) -> Result<()> {
    let call = get_call(ctx).await?;
    let call = call.lock().await;
    let queue = call.queue();

    if let Some(track) = queue.current() {
        let new_len = queue.len().saturating_sub(1);
        queue.skip()?;
        ctx.send(embed(
            format!("Skipped: {}", track.data::<TrackInfo>().title),
            None,
            Some(new_len),
        ))
        .await?;
    } else {
        ctx.send(note("Nothing is playing.")).await?;
    }
    Ok(())
}

#[poise::command(prefix_command, guild_only, aliases("stop"))]
pub async fn pause(ctx: PrefixContext<'_, Data, anyhow::Error>) -> Result<()> {
    let call = get_call(ctx).await?;
    let call = call.lock().await;
    let queue = call.queue();

    if queue.is_empty() {
        ctx.send(note("Nothing is playing.")).await?;
    } else {
        queue.pause()?;
        ctx.send(note("Paused.")).await?;
    }
    Ok(())
}

#[poise::command(prefix_command, guild_only, aliases("continue"))]
pub async fn resume(ctx: PrefixContext<'_, Data, anyhow::Error>) -> Result<()> {
    let call = get_call(ctx).await?;
    let call = call.lock().await;
    let queue = call.queue();

    if queue.is_empty() {
        ctx.send(note("Nothing is playing.")).await?;
    } else {
        queue.resume()?;
        ctx.send(note("Resumed.")).await?;
    }
    Ok(())
}

#[poise::command(prefix_command, guild_only, aliases("clean"))]
pub async fn clear(ctx: PrefixContext<'_, Data, anyhow::Error>) -> Result<()> {
    let call = get_call(ctx).await?;
    let call = call.lock().await;
    let queue = call.queue();

    if queue.is_empty() {
        ctx.send(note("There is no queue.")).await?;
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
    // clear the queue
    queue.modify_queue(|q| {
        if is_playing {
            q.drain(1..);
        } else {
            q.clear();
        }
    });
    ctx.send(note("Queue has been cleared.")).await?;
    Ok(())
}

#[poise::command(prefix_command, guild_only, aliases("q"))]
pub async fn queue(ctx: PrefixContext<'_, Data, anyhow::Error>) -> Result<()> {
    let call = get_call(ctx).await?;
    let call = call.lock().await;
    let queue = call.queue();

    ctx.send(embed(
        "The Queue.",
        Some(EmbedMessage::Queue(queue.current_queue())),
        Some(queue.len()),
    ))
    .await?;

    Ok(())
}

#[poise::command(prefix_command, guild_only, aliases("rm"))]
pub async fn remove(
    ctx: PrefixContext<'_, Data, anyhow::Error>,
    track_index: Option<usize>,
) -> Result<()> {
    let call = get_call(ctx).await?;
    let call = call.lock().await;
    let queue = call.queue();

    if let Some(index) = track_index {
        if let Some(track) = queue.current() {
            if index == 0 {
                let new_len = queue.len().saturating_sub(1);
                queue.skip()?;
                ctx.send(embed(
                    format!("Skipped: {}", track.data::<TrackInfo>().title),
                    None,
                    Some(new_len),
                ))
                .await?;
            } else if let Some(t) = queue.dequeue(index) {
                ctx.send(note(&format!("Removed: {}", t.data::<TrackInfo>().title)))
                    .await?;
            } else {
                bail!("No track at that position.")
            }
        } else {
            ctx.send(note("Nothing is playing.")).await?;
        }
    } else {
        bail!("Please provide the track position.")
    }
    Ok(())
}
