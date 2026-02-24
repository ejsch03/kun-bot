use crate::prelude::*;

pub async fn get_images(title: &str, paths: Vec<PathBuf>) -> Result<Vec<CreateMessage>> {
    let mut id = 0;

    let check_de = |de: DirEntry| -> Option<(PathBuf, String)> {
        let path = de.path();
        // check if it's an image
        ImageFormat::from_extension(path.extension()?)?;
        Some((path.clone(), path.file_name()?.to_str()?.to_string()))
    };

    let mut images = Vec::new();

    for (p, file_name) in paths
        .clone()
        .into_iter()
        .filter_map(|p| Some(read_dir(p).ok()?.filter_map(Result::ok)))
        .flatten()
        .filter_map(check_de)
    {
        id += 1; // increment

        let cm = CreateMessage::default()
            .embed(
                CreateEmbed::default()
                    .title(title)
                    .attachment(file_name)
                    .color(Color::from_rgb(0, 0, 0))
                    .footer(CreateEmbedFooter::new(id.to_string())),
            )
            .add_file(CreateAttachment::path(p).await?);
        images.push(cm)
    }

    (!images.is_empty())
        .then_some(images)
        .ok_or_else(|| anyhow!("The provided path(s) contains no valid images."))
}

pub enum EmbedMessage {
    Queue(Vec<TrackHandle>),
    Song(Box<Song>),
}

pub fn embed(
    author: impl AsRef<str>,
    song: Option<EmbedMessage>,
    queue_length: Option<usize>,
) -> CreateReply {
    let embed = CreateEmbed::new();

    // author
    let embed = embed
        .author(CreateEmbedAuthor::new(author.as_ref()))
        .color(Colour::BLURPLE);

    // title
    let embed = if let Some(msg) = song {
        match msg {
            EmbedMessage::Queue(q) => embed.title(
                q.into_iter()
                    .enumerate()
                    .map(|(i, t)| {
                        let mut s = String::new();
                        if i == 0 {
                            s.push_str("`~`) ");
                        } else {
                            s.push_str(&format!("`{i}`) "));
                        }
                        let song = t.data::<TrackInfo>();
                        s.push_str(&song.title);
                        if let Some(artist) = song.artist.as_ref() {
                            s.push_str(&format!(" ◦ {artist}"));
                        }
                        s
                    })
                    .collect::<Vec<String>>()
                    .join("\n"),
            ),
            EmbedMessage::Song(song) => {
                let dur = Duration::from_secs(song.duration);
                let title = format!("{} - [{}]", song.title, humantime::format_duration(dur));
                embed.title(title).url(song.url)
            }
        }
    } else {
        embed
    };

    // footer
    let embed = if let Some(len) = queue_length {
        embed.footer(CreateEmbedFooter::new(format!("Queue Length: {len}")))
    } else {
        embed
    };

    CreateReply::default().embed(embed).reply(true)
}

pub fn note(msg: &str) -> CreateReply {
    embed(msg, None, None)
}

pub async fn get_loc(ctx: PrefixContext<'_, Data, anyhow::Error>) -> Result<(GuildId, ChannelId)> {
    let guild = ctx.guild().ok_or_else(|| anyhow!("not from a guild."))?;
    let channel_id = guild
        .voice_states
        .get(&ctx.author().id)
        .and_then(|vs| vs.channel_id)
        .ok_or_else(|| anyhow!("you're not in a voice_channel."))?;
    Ok((guild.id, channel_id))
}

pub async fn get_call(ctx: PrefixContext<'_, Data, anyhow::Error>) -> Result<Arc<Mutex<Call>>> {
    let (guild_id, ..) = get_loc(ctx).await?;
    let manager = songbird::get(ctx.serenity_context())
        .await
        .ok_or_else(|| anyhow!("[Internal Error] songbird not registered"))?;
    manager
        .get(guild_id)
        .ok_or_else(|| anyhow!("not in a voice channel"))
}

pub async fn join_helper(ctx: PrefixContext<'_, Data, anyhow::Error>) -> Result<Arc<Mutex<Call>>> {
    let (guild_id, channel_id) = get_loc(ctx).await?;
    if let Ok(call) = get_call(ctx).await
        && let Some(joined) = { call.lock().await.current_channel() }
        && channel_id.get() == joined.0.get()
    {
        call.lock().await.deafen(true).await?;
        return Ok(call);
    }
    let manager = songbird::get(ctx.serenity_context())
        .await
        .ok_or_else(|| anyhow!("songbird not registered"))?;
    let call = manager.join(guild_id, channel_id).await?;
    call.lock().await.deafen(true).await?;
    Ok(call)
}
