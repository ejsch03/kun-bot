// std
pub use std::collections::HashMap;
pub use std::io::{self, Read, Seek};
pub use std::sync::Arc;
pub use std::sync::atomic::{AtomicBool, Ordering};

// error-handling
pub use crate::prelude::{Result, anyhow, bail};

// librespot
pub use librespot::core::authentication::Credentials as LibreCreds;
pub use librespot::core::{Session, SpotifyId, SpotifyUri, cache::Cache};
pub use librespot::discovery::Credentials as LSpotCreds;
pub use librespot::playback::{
    audio_backend::{Sink, SinkResult},
    config::AudioFormat,
    convert::Converter,
    decoder::AudioPacket,
    mixer::NoOpVolume,
    player::{Player, PlayerEvent},
};

// rspotify
pub use rspotify::ClientCredsSpotify as RSpotify;
pub use rspotify::model::{
    AlbumId, FullTrack, Id, PlayableItem, PlaylistId, SearchResult, SearchType, TrackId,
};
pub use rspotify::prelude::BaseClient;

// songbird
pub use songbird::input::core::io::MediaSource;
pub use songbird::input::{AudioStream, AudioStreamError, Compose};

// misc
pub use ringbuf::traits::{Producer, Split};
pub use ringbuf::{HeapCons, HeapProd, HeapRb};
pub use serde::Serialize;
pub use tokio::sync::Mutex;
pub use waitx::{Waiter, Waker};
pub use zerocopy::IntoBytes;

// local
pub use super::auth::*;
pub use super::cfg::*;
pub use super::consts::*;
pub use super::json::*;
pub use super::recv::*;
pub use super::sink::*;
pub use super::source::*;
