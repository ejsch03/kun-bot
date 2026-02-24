// std
pub use std::collections::HashMap;
pub use std::fs::{DirEntry, read_dir};
pub use std::path::{Path, PathBuf};
pub use std::sync::Arc;
pub use std::time::Duration;

// error-handling
pub use anyhow::{Result, anyhow, bail};

// serenity
pub use serenity::all::{
    ChannelId, Color, Colour, Context as SerenityContext, CreateAttachment, CreateEmbed,
    CreateEmbedAuthor, CreateEmbedFooter, CreateMessage, EventHandler, GatewayIntents, GuildId,
    Message, MessageId, UserId,
};
pub use serenity::prelude::TypeMapKey;

// poise
pub use poise::{CreateReply, Framework, FrameworkOptions, PrefixContext, PrefixFrameworkOptions};

// songbird
pub use songbird::Call;
pub use songbird::tracks::{PlayMode, Track, TrackHandle};

// misc
pub use clap::Parser;
pub use image::ImageFormat;
pub use rand::seq::IndexedRandom;
pub use tokio::fs::File;
pub use tokio::io::{AsyncReadExt, AsyncWriteExt};
pub use tokio::sync::Mutex;

// crate
pub use crate::cfg::*;
pub use crate::cmds::*;
pub use crate::handlers::*;
pub use crate::keys::*;
pub use crate::link::*;
pub use crate::stf::*;
pub use crate::util::*;
