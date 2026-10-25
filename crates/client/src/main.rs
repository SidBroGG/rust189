mod app;
mod auth;
mod config;
mod logging;
mod timer;

use app::App;
use assets::{check_fast_integrity, AssetManager, AssetProgress, IntegrityStatus};
use auth::OfflineProfile;
use config::ClientConfig;
use std::path::{Path, PathBuf};
use tracing::{error, info, warn};
use winit::event_loop::EventLoop;

#[cfg(windows)]
#[link(name = "winmm")]
extern "system" {
    fn timeBeginPeriod(uPeriod: u32) -> u32;
    fn timeEndPeriod(uPeriod: u32) -> u32;
}

fn main() -> anyhow::Result<()> {
    #[cfg(windows)]
    unsafe {
        timeBeginPeriod(1);
    }

    logging::init_logging()?;

    info!("===============================================================");
    info!(" Rust189 Client (Protocol 47) - High-Performance Client");
    info!("===============================================================");

    // Initialize multi-threaded Tokio asynchronous runtime in background
    let rt = tokio::runtime::Builder::new_multi_thread()
        .thread_stack_size(8 * 1024 * 1024)
        .enable_all()
        .build()?;
    let _rt_guard = rt.enter();

    let config_path = PathBuf::from("config.toml");
    let config = ClientConfig::load_or_create(&config_path)?;
    info!(
        nickname = %config.auth.nickname,
        render_distance = config.video.render_distance,
        max_fps = config.video.max_fps,
        vsync = config.video.vsync,
        gui_scale = config.video.gui_scale,
        "Configuration loaded successfully from {:?}",
        config_path
    );

    let profile = match OfflineProfile::new(&config.auth.nickname) {
        Ok(p) => p,
        Err(e) => {
            warn!(error = %e, "Invalid nickname in config.toml, falling back to default 'Steve'");
            OfflineProfile::default()
        }
    };
    info!(
        username = %profile.username,
        uuid = %profile.uuid,
        "Offline authentication profile active"
    );

    let assets_dir = Path::new("assets");
    let asset_manager = AssetManager::new(assets_dir);

    // If cache not complete, ensure assets are downloaded
    if !matches!(check_fast_integrity(assets_dir), IntegrityStatus::Complete) {
        info!("Asset cache incomplete, synchronizing vanilla 1.8.9 assets...");
        rt.block_on(async {
            asset_manager
                .ensure_assets(|progress| match progress {
                    AssetProgress::VerifyingCache => {
                        info!("Verifying local asset cache integrity...");
                    }
                    AssetProgress::FetchingVersionManifest => {
                        info!("Fetching official Mojang Version Manifest (v2)...");
                    }
                    AssetProgress::FetchingVersionDetails => {
                        info!("Fetching version 1.8.9 package metadata...");
                    }
                    AssetProgress::DownloadingClientJar => {
                        info!("Downloading client.jar from CDN...");
                    }
                    AssetProgress::ExtractingClientJar => {
                        info!("Extracting textures, models, fonts, and blockstates from client.jar...");
                    }
                    AssetProgress::FetchingAssetIndex => {
                        info!("Downloading asset index 1.8.json...");
                    }
                    AssetProgress::DownloadingObjects {
                        current,
                        total,
                        current_file,
                    } => {
                        if current == 1 || current % 50 == 0 || current == total {
                            info!(
                                "[{}/{}] Downloading assets: {}",
                                current, total, current_file
                            );
                        }
                    }
                    AssetProgress::Ready => {
                        info!("All assets are verified and ready.");
                    }
                })
                .await
        })?;
    } else {
        info!("Integrity check PASSED: All essential assets verified on disk.");
    }

    // Check CLI arguments for server ping mode
    let args: Vec<String> = std::env::args().collect();
    if let Some(pos) = args.iter().position(|a| a == "--ping") {
        let target = args
            .get(pos + 1)
            .map(|s| s.as_str())
            .unwrap_or("localhost:25565");
        let (host, port) = if let Some((h, p)) = target.split_once(':') {
            (h, p.parse::<u16>().unwrap_or(25565))
        } else {
            (target, 25565)
        };

        info!("===============================================================");
        info!(" Pinging server: {}:{}", host, port);
        info!("===============================================================");
        rt.block_on(async {
            match protocol::ping_server(host, port).await {
                Ok((status, latency)) => {
                    info!(">>> Server responded successfully! <<<");
                    info!(
                        version = %status.version.name,
                        protocol = status.version.protocol,
                        "Server version info"
                    );
                    info!(
                        online = status.players.online,
                        max = status.players.max,
                        "Players online"
                    );
                    if let Some(ref sample) = status.players.sample {
                        let names: Vec<&str> = sample.iter().map(|p| p.name.as_str()).collect();
                        info!(players = ?names, "Player sample");
                    }
                    info!(motd = %status.description.to_plain_text(), "MOTD");
                    info!(latency = ?latency, "Ping round-trip latency");
                }
                Err(e) => {
                    error!(error = %e, "Failed to ping server {}:{}", host, port);
                }
            }
        });
        return Ok(());
    }

    info!("Starting Winit event loop...");
    let event_loop = EventLoop::new()?;
    let mut app = App::new(config, config_path);
    event_loop.run_app(&mut app)?;

    info!("Client exited cleanly.");
    #[cfg(windows)]
    unsafe {
        timeEndPeriod(1);
    }
    Ok(())
}
