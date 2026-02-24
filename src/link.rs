use crate::prelude::*;

pub async fn delete_if_linked(
    ctx: &SerenityContext,
    channel_id: ChannelId,
    msg: &MessageId,
) -> Result<()> {
    let link_id = {
        //  delete the message if the message is linked
        let data = ctx.data.read().await;
        let links = data
            .get::<MessageLink>()
            .ok_or_else(|| anyhow!("Message link map hasn't been instantiated."))?;
        *links
            .lock()
            .await
            .get(msg)
            .ok_or_else(|| anyhow!("Message did not have a link to embed."))?
    };
    ctx.http.delete_message(channel_id, link_id, None).await?;
    {
        //  remove the message from links if it was able to be deleted
        let data = ctx.data.write().await;
        let mut links = data
            .get::<MessageLink>()
            .ok_or_else(|| anyhow!("Message link map hasn't been instantiated."))?
            .lock()
            .await;
        links.remove(msg);
        links.remove(&link_id);
    }
    Ok(())
}

pub async fn link_messages(
    links: &Mutex<HashMap<MessageId, MessageId>>,
    from: MessageId,
    to: MessageId,
) -> Result<()> {
    let mut map = links.lock().await;
    map.insert(from, to);
    map.insert(to, from);
    Ok(())
}

pub fn try_into_guild_id(s: &str) -> Result<GuildId> {
    s.parse::<u64>().map(GuildId::from).map_err(Into::into)
}

pub async fn update_wl_file(whitelist: &Whitelist) -> Result<()> {
    let mut f = File::create(whitelist.path()).await?;

    f.write_all(
        whitelist
            .data()
            .iter()
            .map(|g| g.to_string())
            .collect::<Vec<String>>()
            .join(" ")
            .as_bytes(),
    )
    .await?;
    f.flush().await.map_err(Into::into)
}

pub async fn try_whitelist_add(
    whitelist: &Mutex<Whitelist>,
    msg: &Message,
    args: &str,
) -> Result<()> {
    let id = if args.is_empty() {
        msg.guild_id
            .ok_or_else(|| anyhow!("Message not received over the gateway."))?
    } else {
        try_into_guild_id(args)?
    };

    let mut whitelist = whitelist.lock().await;
    if whitelist.data().contains(&id) {
        bail!("Server is already whitelisted")
    } else {
        whitelist.data_mut().push(id);
        update_wl_file(&whitelist).await
    }
}
