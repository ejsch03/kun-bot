pub use std::collections::HashMap;
pub use std::io::{self, Read, Seek};
pub use std::sync::Arc;

// error-handling
pub use crate::prelude::{Result, anyhow, bail};

// librespot
pub use librespot::core::{Session, SessionConfig, SpotifyId, SpotifyUri, cache::Cache};
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
pub use rspotify::model::{AlbumId, FullTrack, Id, SearchResult, SearchType, TrackId};
pub use rspotify::prelude::BaseClient;

// songbird
pub use songbird::input::core::io::MediaSource;
pub use songbird::input::{Input, LiveInput, Parsed, core::probe::Hint};

// symphonia
pub use symphonia::core::{codecs::DecoderOptions, formats::FormatOptions, io::MediaSourceStream};

// misc
pub use ringbuf::traits::{Producer, Split};
pub use ringbuf::{HeapCons, HeapProd, HeapRb};
pub use serde::Serialize;
pub use tokio::sync::Mutex;
pub use waitx::{Waiter, Waker};
pub use zerocopy::IntoBytes;

// local
pub use super::cfg::*;
pub use super::consts::*;
pub use super::json::*;
pub use super::recv::*;
pub use super::sink::*;
