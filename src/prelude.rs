// std
pub use std::collections::HashMap;
pub use std::fs::{DirEntry, read_dir};
pub use std::path::{Path, PathBuf};
pub use std::sync::Arc;
pub use std::time::{Duration, SystemTime};

// error-handling
pub use anyhow::{Context, Result, anyhow, bail};

// async
pub use tokio::fs::File;
pub use tokio::io::{AsyncReadExt, AsyncWriteExt};
pub use tokio::sync::Mutex;
pub use tokio::task::{JoinHandle, spawn};
pub use tokio::time::sleep;

// serenity
pub use serenity::all::{
    ChannelId, Color, Colour, Context as SerenityContext, CreateAttachment, CreateEmbed,
    CreateEmbedAuthor, CreateEmbedFooter, CreateMessage, EventHandler, GatewayIntents, GuildId,
    Message, MessageId, UserId,
};
pub use serenity::prelude::TypeMapKey;

// poise
pub use poise::{CreateReply, Framework, FrameworkOptions, PrefixFrameworkOptions};
pub type PrefixContext<'a> = poise::PrefixContext<'a, Data, anyhow::Error>;
pub type FrameworkError<'a> = poise::FrameworkError<'a, Data, anyhow::Error>;

// songbird
pub use songbird::Call;
pub use songbird::input::Input;
pub use songbird::tracks::{PlayMode, Track, TrackHandle};

// logging
pub use tracing_appender::non_blocking::WorkerGuard;
pub use tracing_subscriber::EnvFilter;
pub use tracing_subscriber::layer::SubscriberExt;
pub use tracing_subscriber::util::SubscriberInitExt;

// misc
pub use clap::Parser;
pub use image::ImageFormat;
pub use rand::seq::{IndexedRandom, SliceRandom};

// crate
pub use crate::cfg::*;
pub use crate::cmds::*;
pub use crate::handlers::*;
pub use crate::hooks::*;
pub use crate::keys::*;
pub use crate::link::*;
pub use crate::stf::*;
pub use crate::util::*;
