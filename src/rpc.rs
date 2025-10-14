use std::time::SystemTime;

use discord_presence::{
    Client,
    models::{ActivityAssets, ActivityType, DisplayType},
};

use crate::songs;

// Discord application ID
const CLIENT_ID: u64 = 1350909681681436692;

// My own assets
const ASSET_MINECRAFT: &str = "mclogo";

// Soundtrack assets
const ASSET_ALBUM_VOL_ALPHA: &str = "vol_alpha";
const ASSET_ALBUM_VOL_BETA: &str = "vol_beta";
const ASSET_ALBUM_VOL_NETHER: &str = "vol_nether";
const ASSET_ALBUM_VOL_CAVES: &str = "vol_caves";
const ASSET_ALBUM_VOL_WILD: &str = "vol_wild";
const ASSET_ALBUM_VOL_TRAILS: &str = "vol_trails";
const ASSET_ALBUM_VOL_TRICKY: &str = "vol_tricky";
const ASSET_ALBUM_VOL_CHASE: &str = "vol_chase";

/// Map album to its Discord Rich Presence asset names.
/// WARNING: `small_image` can be "" (empty) if no small image is desired,
/// CHECK FOR THIS BEFORE USING OR DISCORD PRESENCE WILL HANG!
/// Returns (large_image, small_image)
fn album_to_asset(album: &String) -> (&'static str, &'static str) {
    let large_image = match album.as_str() {
        "Minecraft - Volume Alpha" => ASSET_ALBUM_VOL_ALPHA,
        "Minecraft - Volume Beta" => ASSET_ALBUM_VOL_BETA,
        "Minecraft: Nether Update (Original Game Soundtrack)" => ASSET_ALBUM_VOL_NETHER,
        "Minecraft: Caves & Cliffs (Original Game Soundtrack)" => ASSET_ALBUM_VOL_CAVES,
        "Minecraft: The Wild Update (Original Game Soundtrack)" => ASSET_ALBUM_VOL_WILD,
        "Minecraft: Trails & Tales (Original Game Soundtrack)" => ASSET_ALBUM_VOL_TRAILS,
        "Minecraft: Tricky Trials (Original Game Soundtrack)" => ASSET_ALBUM_VOL_TRICKY,
        "Minecraft: Chase the Skies (Original Game Soundtrack)" => ASSET_ALBUM_VOL_CHASE,
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

fn dimension_to_string(dimension: &songs::Dimension) -> &'static str {
    match dimension {
        songs::Dimension::Overworld => "Overworld Music",
        songs::Dimension::Nether => "Nether Music",
        songs::Dimension::End => "End Music",
        songs::Dimension::Disc => "Minecraft Music Disc",
        songs::Dimension::Minecraft => "Minecraft Music",
    }
}

pub fn epoch_secs() -> u64 {
    SystemTime::now()
        .duration_since(SystemTime::UNIX_EPOCH)
        .unwrap()
        .as_secs()
}

pub struct RpcClient {
    active_media: String,
    last_start_time: u64,
    last_end_time: u64,
    client: Client,
}

impl RpcClient {
    pub fn blocking_start(&mut self) {
        println!("Starting RPC client");

        self.client.start();
        self.client
            .block_until_event(discord_presence::Event::Ready)
            .ok();

        println!("Started RPC client");
    }

    pub fn new() -> Self {
        RpcClient {
            active_media: String::new(),
            last_start_time: 0,
            last_end_time: 0,
            client: Client::new(CLIENT_ID),
        }
    }

    pub fn update_rpc(&mut self, state: &crate::vlc_http::VlcState) {
        // Extract metadata
        let meta = state
            .information
            .as_ref()
            .and_then(|info| info.category.as_ref())
            .and_then(|cat| cat.meta.as_ref());

        let title = meta
            .and_then(|m| m.title.clone())
            .unwrap_or_else(|| "Unknown Title".to_string());
        let artist = meta
            .and_then(|m| m.artist.clone())
            .unwrap_or_else(|| "Artist".to_string());
        let album = meta
            .and_then(|m| m.album.clone())
            .unwrap_or_else(|| "Album".to_string());

        let filename = meta.and_then(|m| m.filename.clone()).unwrap_or_default();
        // VLC provides a `time` field, but it's only updated every second,
        // so we use the position field which has many decimals for better accuracy.
        let seek = state.position.unwrap_or(0.0) * state.length.unwrap_or(0) as f64;
        let playing = state.state.as_deref() == Some("playing");

        if self.active_media != filename {
            // Media changed
            self.last_start_time = epoch_secs();
            self.last_end_time = self.last_start_time + state.length.unwrap_or(0) as u64;

            // client.clear_activity().ok(); // I don't believe this is needed

            self.active_media = filename.clone();
        }

        // Seek delta is [actual seek] - [expected seek (calculated from [now] - [last_start_time])]
        let seek_delta = seek - (epoch_secs() as i64 - self.last_start_time as i64) as f64;

        if seek_delta.abs() > 1.5 {
            // Significant seek detected, update start and end time

            // Logic here is based on: since the times are absolute,
            // if the user seeks forward, it'd be like the song started
            // earlier, so both the start and end time should be moved
            // backwards in time (by subtracting seek_delta). Similarly,
            // if the user seeks backwards, it'd be like the song started
            // later, so both times should be moved forwards (by subtracting
            // seek_delta, which is negative in this case, effectively adding it).

            self.last_start_time = self
                .last_start_time
                .checked_sub_signed(seek_delta.round() as i64)
                .unwrap_or_default();
            self.last_end_time = self
                .last_end_time
                .checked_sub_signed(seek_delta.round() as i64)
                .unwrap_or_default();
        }

        let dimension = songs::song_to_dimension(
            std::path::Path::new(&filename)
                .file_name()
                .and_then(|s| s.to_str())
                .unwrap_or_default(),
        );

        let (large_image, small_image) = album_to_asset(&album);

        // println!(
        //     "Updating Discord activity: {} - {} ({})",
        //     artist, title, album
        // );

        let fn_add_assets = |assets: ActivityAssets| -> ActivityAssets {
            let dimension_annotation = dimension_to_string(&dimension);

            // Note that in status display for music, the large image text is not
            // shown as a tooltip on the large image, rather, under the details
            // text instead. Small text seems to only show sometimes.

            let assets = assets.large_image(large_image).large_text(match playing {
                true => dimension_annotation,
                false => "Paused",
            });

            if small_image.is_empty() {
                assets
            } else {
                assets.small_image(small_image).small_text("From Minecraft")
            }
        };

        if playing {
            // println!("Setting playing activity");
            self.client
                .set_activity(|activity| {
                    activity
                        .activity_type(ActivityType::Listening)
                        .status_display(DisplayType::State)
                        .details(&format!("{} - {}", artist, title))
                        .state(album)
                        .timestamps(|f| f.start(self.last_start_time).end(self.last_end_time))
                        .assets(fn_add_assets)
                })
                .unwrap();
            // println!("Finished setting activity");
        } else {
            self.client
                .set_activity(|activity| {
                    activity
                        .activity_type(ActivityType::Listening)
                        .status_display(DisplayType::State)
                        .details(&format!("{} - {}", artist, title))
                        .state(album)
                        .assets(fn_add_assets)
                })
                .unwrap();
        }
    }
}
