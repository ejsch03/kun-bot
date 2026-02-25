pub use std::collections::{HashMap, VecDeque};
pub use std::sync::Arc;

// error-handling
pub use crate::prelude::{Result, anyhow, bail};

// librespot
pub use librespot::core::{Session, SpotifyId, SpotifyUri, cache::Cache};
pub use librespot::discovery::Credentials as LSpotCreds;
pub use librespot::playback::{
    audio_backend::{Sink, SinkResult},
    config::AudioFormat,
    convert::Converter,
    decoder::AudioPacket,
    mixer::NoOpVolume,
    player::Player,
};

// rspotify
pub use rspotify::ClientCredsSpotify as RSpotify;
pub use rspotify::model::{AlbumId, FullTrack, Id, SearchResult, SearchType};
pub use rspotify::prelude::BaseClient;

// songbird
pub use songbird::input::{Input, LiveInput};

// misc
pub use parking_lot::Mutex;
pub use serde::Serialize;
pub use tokio::sync::{Mutex as AsyncMutex, MutexGuard as AsyncMutexGuard};
pub use zerocopy::IntoBytes;

// local
pub use super::cfg::*;
pub use super::consts::*;
pub use super::json::*;
pub use super::recv::*;
pub use super::sink::*;
