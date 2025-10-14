mod rpc;
mod songs;
mod vlc_http;

use std::thread::sleep;
use std::time::Duration;

use vlc_http::VlcHttpClient;

const PLAYLIST_PATH: &str = "Minecraft OST.xspf";

const VLC_PATH: &str = r"C:\Program Files\VideoLAN\VLC\vlc.exe";
const VLC_HOST: &str = "localhost";
const VLC_PORT: u16 = 3103;
const VLC_PASSWORD: &str = "melominoxide";

fn main() {
    let mut discord_rpc = rpc::RpcClient::new();
    discord_rpc.blocking_start();

    let vlc_client = VlcHttpClient::new(VLC_HOST, VLC_PORT, VLC_PASSWORD);
    vlc_client
        .spawn_vlc_if_needed(VLC_PATH, PLAYLIST_PATH)
        .unwrap();

    let ready = vlc_client.wait_until_ready(Duration::from_secs(10));
    if !ready {
        eprintln!("VLC HTTP interface not reachable after timeout");

        return;
    } else {
        println!("VLC HTTP interface reachable");
    }

    loop {
        match vlc_client.query_status() {
            Ok(state) => {
                discord_rpc.update_rpc(&state);
            }
            Err(e) => {
                eprintln!("Error querying VLC HTTP: {}", e);
            }
        }

        sleep(Duration::from_secs(1));
    }
}
