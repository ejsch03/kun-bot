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

pub enum EmbedItem {
    Track(Box<Song>),
    Playlist(String),
    Queue(Vec<TrackHandle>),
}

pub fn embed(
    ctx: PrefixContext<'_>,
    author: impl AsRef<str>,
    song: Option<EmbedItem>,
    brief: bool,
    queue_length: Option<usize>,
) -> CreateReply {
    let embed = CreateEmbed::new();

    // author
    let embed = embed
        .author({
            let embed = CreateEmbedAuthor::new(author.as_ref());

            // this is just bro
            if let Some(url) = song.as_ref().and_then(|s| {
                if let EmbedItem::Track(song) = s {
                    song.cover_url.clone()
                } else {
                    None
                }
            }) {
                embed.icon_url(url)
            } else if let Some(url) = ctx.guild().and_then(|g| g.icon_url()) {
                embed.icon_url(url)
            } else {
                embed
            }
        })
        .color(Colour::BLURPLE);

    // title
    let embed = if let Some(msg) = song {
        match msg {
            EmbedItem::Queue(q) => {
                const MAX_QUEUE_LENGTH: usize = 8;

                let mut tracks_iter = q.iter();
                let Some(top) = tracks_iter.next() else {
                    return CreateReply::default()
                        .embed(embed.description("<empty>."))
                        .reply(true);
                };
                let mut msg = format!("Current: **{}**\n", top.data::<TrackInfo>().title);

                let list_str = tracks_iter
                    .take(MAX_QUEUE_LENGTH)
                    .enumerate()
                    .map(|(i, t)| {
                        let mut s = String::new();
                        // 1-based so the numbers match `remove <position>`
                        s.push_str(&format!("{}. ", i + 1));
                        let song = t.data::<TrackInfo>();
                        s.push_str(&song.title);
                        if let Some(artist) = song.artist.as_ref() {
                            s.push_str(&format!("\u{00A0}\u{00A0}◦\u{00A0}\u{00A0}{artist}"));
                        }
                        s
                    })
                    .collect::<Vec<String>>()
                    .join("\n");

                msg.push_str(list_str.as_str());

                if q.len() > MAX_QUEUE_LENGTH + 1 {
                    msg.push_str("\n...");
                }

                embed.description(msg)
            }
            EmbedItem::Track(song) => {
                if !brief {
                    let dur = Duration::from_secs(song.duration);

                    let mut s = song.title.clone();
                    if let Some(artist) = song.artist.as_ref() {
                        s.push_str(&format!("\u{00A0}\u{00A0}◦\u{00A0}\u{00A0}{artist}"));
                    }
                    let dur = format!(" ・ {}", humantime::format_duration(dur));
                    s.push_str(&dur);
                    embed.title(s).url(song.track_url.as_str())
                } else {
                    embed
                }
            }
            EmbedItem::Playlist(title) => embed.title(title),
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

pub fn note(ctx: PrefixContext<'_>, song: Option<EmbedItem>, msg: &str) -> CreateReply {
    embed(ctx, msg, song, true, None)
}

pub async fn get_loc(ctx: PrefixContext<'_>) -> Result<(GuildId, ChannelId)> {
    let guild = ctx.guild().ok_or_else(|| anyhow!("not from a guild."))?;
    let channel_id = guild
        .voice_states
        .get(&ctx.author().id)
        .and_then(|vs| vs.channel_id)
        .ok_or_else(|| anyhow!("you're not in a voice_channel."))?;
    Ok((guild.id, channel_id))
}

pub async fn get_call(ctx: PrefixContext<'_>) -> Result<Arc<Mutex<Call>>> {
    let (guild_id, ..) = get_loc(ctx).await?;
    let manager = songbird::get(ctx.serenity_context())
        .await
        .ok_or_else(|| anyhow!("[Internal Error] songbird not registered"))?;
    manager
        .get(guild_id)
        .ok_or_else(|| anyhow!("not in a voice channel"))
}

pub async fn join_helper(ctx: PrefixContext<'_>) -> Result<Arc<Mutex<Call>>> {
    tracing::debug!("attempting to join channel");
    let (guild_id, channel_id) = get_loc(ctx).await?;
    if let Ok(call) = get_call(ctx).await {
        let mut guard = call.lock().await;
        if let Some(joined) = guard.current_channel()
            && channel_id.get() == joined.0.get()
        {
            // redeafen if not already deafened
            if !guard.is_deaf() {
                guard.deafen(true).await?;
            }
            drop(guard);
            return Ok(call);
        }
    }
    let manager = songbird::get(ctx.serenity_context())
        .await
        .ok_or_else(|| anyhow!("songbird not registered"))?;
    let call = manager.join(guild_id, channel_id).await?;
    call.lock().await.deafen(true).await?;

    tracing::debug!("joined voice channel");

    Ok(call)
}

pub async fn play_helper(ctx: PrefixContext<'_>, query: Vec<String>, is_next: bool) -> Result<()> {
    ctx.channel_id().broadcast_typing(ctx.http()).await?;
    _whitelist(ctx)?;

    let query = query
        .into_iter()
        .map(|s| s.trim().to_string())
        .collect::<Vec<String>>()
        .join(" ");

    tracing::debug!("searching for song");
    let search_results = ctx.data.stf.search(&query).await?;

    let call = join_helper(ctx).await?;
    let mut call = call.lock().await;

    // process the search results
    let item = match search_results {
        FullSearchResult::Track(song) => {
            tracing::debug!("found song {:?}", song.title);

            let input = Input::Lazy(Box::new(ctx.data.stf.source(song.uri.clone())));
            let track = Track::new_with_data(input, Arc::new(TrackInfo::new(*song.clone())));
            call.enqueue(track).await;

            if is_next {
                // move the new track just behind the currently playing one
                call.queue().modify_queue(|q| {
                    if q.len() > 2 {
                        q.make_contiguous()[1..].rotate_right(1);
                    }
                });
            }
            EmbedItem::Track(song)
        }
        FullSearchResult::Playlist { title, songs } => {
            if songs.is_empty() {
                bail!("the playlist contains no playable songs.")
            }

            let n = songs.len();
            tracing::debug!("found {n} songs in playlist {title:?}");

            for song in songs {
                let input = Input::Lazy(Box::new(ctx.data.stf.source(song.uri.clone())));
                let track = Track::new_with_data(input, Arc::new(TrackInfo::new(song)));
                call.enqueue(track).await;
            }

            if is_next {
                // move the playlist ahead of any previously queued tracks,
                // keeping the currently playing track at the front
                call.queue().modify_queue(|q| {
                    if q.len() > n + 1 {
                        q.make_contiguous()[1..].rotate_right(n);
                    }
                });
            }

            EmbedItem::Playlist(title)
        }
    };

    // determine the new queue length
    let len = call.queue().len();
    drop(call);

    // finish the embed, then send it
    ctx.send(embed(
        ctx,
        if is_next {
            "Playing next."
        } else {
            "Added to Queue."
        },
        Some(item),
        false,
        Some(len),
    ))
    .await?;
    Ok(())
}

// TODO - this is temp
pub fn _whitelist(ctx: PrefixContext<'_>) -> Result<()> {
    if [
        GuildId::new(684429201398562855),
        GuildId::new(1090358332440711209),
    ]
    .contains(
        &ctx.guild_id()
            .ok_or_else(|| anyhow!("allowed in guilds only."))?,
    ) {
        Ok(())
    } else {
        bail!("this guild isn't allowed to use music features... yet.")
    }
}
