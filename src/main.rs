mod rpc;
mod songs;
mod vlc_http;

use std::process::Command;
use std::thread::sleep;
use std::time::Duration;

use vlc_http::VlcHttpClient;

const VLC_PATH: &str = r"C:\Program Files\VideoLAN\VLC\vlc.exe";
const VLC_HOST: &str = "localhost";
const VLC_PORT: u16 = 3103;
const VLC_PASSWORD: &str = "melominoxide";

const PLAYLIST_PATH: &str = "Minecraft OST.xspf";

fn spawn_vlc() {
    // Start VLC with HTTP interface enabled. Non-blocking spawn.
    let args = [
        "--extraintf",
        "http",
        "--http-host",
        VLC_HOST,
        "--http-port",
        &VLC_PORT.to_string(),
        "--http-password",
        VLC_PASSWORD,
        "--intf",
        "qt",
        "--random",
        "--loop",
        PLAYLIST_PATH,
    ];

    match Command::new(VLC_PATH).args(&args).spawn() {
        Ok(child) => println!("Spawned VLC (pid={})", child.id()),
        Err(e) => eprintln!("Failed to spawn VLC: {}", e),
    }
}

fn main() {
    println!("Starting Discord RPC and VLC HTTP poller");

    let mut discord_rpc = rpc::start();
    let mut rpc_state = rpc::RPCState::new();
    println!("Started RPC client");

    spawn_vlc();

    let vlc_client = VlcHttpClient::new(VLC_HOST, VLC_PORT, VLC_PASSWORD);

    let ready = vlc_client.wait_until_ready(Duration::from_secs(10));
    if !ready {
        eprintln!("VLC HTTP interface not reachable after timeout");

        return;
    } else {
        println!("VLC HTTP interface reachable")
    }

    loop {
        match vlc_client.query_status() {
            Ok(state) => {
                rpc_state.update_rpc(&mut discord_rpc, &state);
            }
            Err(e) => {
                eprintln!("Error querying VLC HTTP: {}", e);
            }
        }

        sleep(Duration::from_secs(1));
    }
}
