use crate::{songs, vlc_http};

use discord_rich_presence::{
    DiscordIpc, DiscordIpcClient,
    activity::{self, Activity, Assets, StatusDisplayType},
};
use std::time::SystemTime;

// Discord application ID
const CLIENT_ID: &str = "1350909681681436692";

// My own assets
const ASSET_MINECRAFT: &str = "mclogo";

// Soundtrack assets
const ASSET_ALBUM_ALPHA: &str = "vol_alpha";
const ASSET_ALBUM_BETA: &str = "vol_beta";
const ASSET_ALBUM_AQUATIC: &str = "vol_c418_aquatic";
const ASSET_ALBUM_NETHER: &str = "vol_nether";
const ASSET_ALBUM_CAVES: &str = "vol_caves";
const ASSET_ALBUM_WILD: &str = "vol_wild";
const ASSET_ALBUM_TRAILS: &str = "vol_trails";
const ASSET_ALBUM_TRICKY: &str = "vol_tricky";
const ASSET_ALBUM_CHASE: &str = "vol_chase";

/// Map album to its Discord Rich Presence asset names.
/// WARNING: `small_image` can be "" (empty) if no small image is desired,
/// CHECK FOR THIS BEFORE USING OR DISCORD PRESENCE WILL HANG!
/// Returns (large_image, small_image)
fn album_to_asset(album: &String) -> (&'static str, &'static str) {
    if is_aquatic_album(album) {
        return (ASSET_ALBUM_AQUATIC, ASSET_MINECRAFT);
    }

    let large_image = match album.as_str() {
        "Minecraft - Volume Alpha" => ASSET_ALBUM_ALPHA,
        "Minecraft - Volume Beta" => ASSET_ALBUM_BETA,
        "Minecraft: Nether Update (Original Game Soundtrack)" => ASSET_ALBUM_NETHER,
        "Minecraft: Caves & Cliffs (Original Game Soundtrack)" => ASSET_ALBUM_CAVES,
        "Minecraft: The Wild Update (Original Game Soundtrack)" => ASSET_ALBUM_WILD,
        "Minecraft: Trails & Tales (Original Game Soundtrack)" => ASSET_ALBUM_TRAILS,
        "Minecraft: Tricky Trials (Original Game Soundtrack)" => ASSET_ALBUM_TRICKY,
        "Minecraft: Chase the Skies (Original Game Soundtrack)" => ASSET_ALBUM_CHASE,
        _ => ASSET_MINECRAFT,
    };

    (
        large_image,
        match large_image {
            ASSET_MINECRAFT => "",
            _ => ASSET_MINECRAFT,
        },
    )
}

fn is_aquatic_album(album: &String) -> bool {
    match album.as_str() {
        "Axolotl" | "Dragon Fish" | "Shuniji" => true,
        _ => false,
    }
}

fn special_case_aquatic(f: &String) -> String {
    match is_aquatic_album(f) {
        true => "Minecraft: Update Aquatic *(Not a real album)*".to_string(),
        false => f.clone(),
    }
}

fn dimension_to_string(dimension: &songs::Dimension) -> &'static str {
    match dimension {
        songs::Dimension::Overworld => "Overworld music",
        songs::Dimension::Nether => "Nether music",
        songs::Dimension::End => "End music",
        songs::Dimension::Disc => "Music disc",
        songs::Dimension::Minecraft => "Minecraft music",
    }
}

pub fn epoch_ms() -> i64 {
    SystemTime::now()
        .duration_since(SystemTime::UNIX_EPOCH)
        .unwrap()
        .as_millis() as i64
}

struct VlcCache {
    active_media: String,
    last_playing: bool,
    last_start_time: i64,
    last_end_time: i64,
}

pub struct RpcClient {
    vlc_cache: VlcCache,
    client: DiscordIpcClient,
    connected: bool,
}

impl RpcClient {
    pub fn blocking_start(&mut self) {
        println!("Starting RPC client");

        match self.client.connect().ok() {
            Some(()) => {
                self.connected = true;
                println!("Started RPC client");
            }
            None => {
                self.connected = false;
                println!("Failed to connect to Discord, will continuously retry");
            }
        }
    }

    pub fn new() -> Self {
        RpcClient {
            vlc_cache: VlcCache {
                active_media: String::new(),
                last_playing: false,
                last_start_time: 0,
                last_end_time: 0,
            },
            client: DiscordIpcClient::new(CLIENT_ID),
            connected: false,
        }
    }

    pub fn update_connected(&mut self, attempt_reconnect: bool) {
        match self.client.send(serde_json::Value::Null, 1) {
            Ok(_) => {
                self.connected = true;
            }
            Err(_) => {
                if self.connected {
                    println!("Lost connection to Discord, will continuously retry");
                }

                self.connected = false;
                if attempt_reconnect {
                    self.connected = match self.client.connect().ok() {
                        Some(()) => true,
                        None => false,
                    };

                    if self.connected {
                        println!("Reconnected to Discord");
                    }
                }
            }
        }
    }

    pub fn disconnect(&mut self) {
        if self.connected {
            self.client.close().ok();
        }
        self.connected = false;
    }

    pub fn update_rpc(&mut self, state: &vlc_http::VlcState) {
        self.update_connected(true);

        // Extract metadata
        let meta = state
            .information
            .as_ref()
            .and_then(|info| info.category.as_ref())
            .and_then(|cat| cat.meta.as_ref());

        let title = meta
            .and_then(|m| m.title.clone())
            .unwrap_or("Unknown Title".to_string());
        let artist = meta
            .and_then(|m| m.artist.clone())
            .unwrap_or("Artist".to_string());
        let album_raw = meta
            .and_then(|m| m.album.clone())
            .unwrap_or("Album".to_string());
        let album = special_case_aquatic(&album_raw);

        let filename = meta.and_then(|m| m.filename.clone()).unwrap_or_default();
        let song_changed = self.vlc_cache.active_media != filename;
        // VLC provides a `time` field, but it's only accurate to the second, so
        // we use the position field % which has many decimals for better accuracy.
        let seek = state.position.unwrap_or(0.0) * state.length.unwrap_or(0) as f64 * 1000.0;
        // Seek delta is [actual seek] - [expected seek (calculated from [now] - [last_start_time])]
        let seek_delta = seek - (epoch_ms() - self.vlc_cache.last_start_time) as f64;
        let playing = state.state.as_deref() == Some("playing");

        if song_changed {
            self.vlc_cache.last_start_time = epoch_ms();
            self.vlc_cache.last_end_time =
                self.vlc_cache.last_start_time + state.length.unwrap_or(0) as i64 * 1000;

            // client.clear_activity().ok(); // I don't believe this is needed

            self.vlc_cache.active_media = filename.clone();
        }

        if playing && seek_delta.abs() > 670.0 {
            // Significant seek detected, update start and end time

            // Logic here is based on: since the times are absolute,
            // if the user seeks forward, it'd be like the song started
            // earlier, so both the start and end time should be moved
            // backwards in time (by subtracting seek_delta). Similarly,
            // if the user seeks backwards, it'd be like the song started
            // later, so both times should be moved forwards (by subtracting
            // seek_delta, which is negative in this case, effectively adding it).

            self.vlc_cache.last_start_time -= seek_delta.round() as i64;
            self.vlc_cache.last_end_time -= seek_delta.round() as i64;
        } else {
            if !song_changed && (self.vlc_cache.last_playing == playing) {
                // Optimization to avoid updating Discord RPC if nothing changed
                return;
            }
        }
        self.vlc_cache.last_playing = playing;

        if !self.connected {
            // Do not proceed to activity related calls
            return;
        }

        let dimension = songs::song_to_dimension(
            std::path::Path::new(&filename)
                .file_name()
                .and_then(|s| s.to_str())
                .unwrap_or_default(),
        );

        let (large_image, small_image) = album_to_asset(&album_raw);

        // println!(
        //     "Updating Discord activity: {} - {} ({})",
        //     artist, title, album
        // );

        let dimension_annotation = dimension_to_string(&dimension);

        // Note that in status display for music, the large image text is
        // also shown under the details line

        let large_url = format!("https://minecraft.wiki/w/{}", album.replace(" ", "_"));
        let get_assets = || -> Assets {
            let mut assets: Assets<'_> =
                Assets::new()
                    .large_image(large_image)
                    .large_text(match playing {
                        true => dimension_annotation,
                        false => "Paused",
                    });

            // The aquatic update did not have an official album, as
            // C418 was only commissioned for some tracks, so we avoid
            // linking an article for it.
            if large_image != ASSET_MINECRAFT && large_image != ASSET_ALBUM_AQUATIC {
                assets = assets.large_url(&large_url);
            }

            if !small_image.is_empty() {
                assets = assets.small_image(small_image).small_text("from Minecraft");
            }

            assets
        };

        let name = format!("{} - {}", artist, title);

        let mut activity = Activity::new()
            .activity_type(activity::ActivityType::Listening)
            .status_display_type(StatusDisplayType::Details)
            .details(&name)
            .state(&album)
            .assets(get_assets());

        activity = match playing {
            true => activity.timestamps(
                activity::Timestamps::new()
                    .start(self.vlc_cache.last_start_time)
                    .end(self.vlc_cache.last_end_time),
            ),
            false => activity,
        };

        self.client.set_activity(activity).ok();
    }
}
