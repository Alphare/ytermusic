use std::{path::PathBuf, str::FromStr, sync::Arc};

use flume::Sender;
use ytpapi::{Playlist, YTApi};

use crate::{
    structures::performance,
    systems::logger::log_,
    term::{ManagerMessage, Screens},
};

pub fn spawn_api_task(updater_s: Arc<Sender<ManagerMessage>>) {
    tokio::task::spawn(async move {
        log_("API task on");
        let guard = performance::guard("API task");
        match YTApi::from_header_file(PathBuf::from_str("headers.txt").unwrap().as_path()).await {
            Ok(api) => {
                let api = Arc::new(api);
                for playlist in api.playlists() {
                    spawn_browse_playlist_task(playlist.clone(), api.clone(), updater_s.clone())
                }
            }
            Err(e) => {
                log_(format!("{:?}", e));
            }
        }
        drop(guard);
    });
}

fn spawn_browse_playlist_task(
    playlist: Playlist,
    api: Arc<YTApi>,
    updater_s: Arc<Sender<ManagerMessage>>,
) {
    tokio::task::spawn(async move {
        let guard = format!("Browse playlist {}", playlist.name);
        let guard = performance::guard(&guard);
        match api.browse_playlist(&playlist.browse_id).await {
            Ok(videos) => {
                updater_s
                    .send(
                        ManagerMessage::AddElementToChooser((
                            format!("{} ({})", playlist.name, playlist.subtitle),
                            videos,
                        ))
                        .pass_to(Screens::Playlist),
                    )
                    .unwrap();
            }
            Err(e) => {
                log_(format!("{:?}", e));
            }
        }

        drop(guard);
    });
}
