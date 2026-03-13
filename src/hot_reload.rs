use std::path::{Path, PathBuf};
use std::sync::mpsc::{self, Receiver};

use anyhow::Context;
use notify::{Event, RecommendedWatcher, RecursiveMode, Watcher};

pub struct ShaderHotReload {
    receiver: Receiver<PathBuf>,
    _watcher: RecommendedWatcher,
}

impl ShaderHotReload {
    pub fn new(path: impl AsRef<Path>) -> anyhow::Result<Option<Self>> {
        let path = path.as_ref();
        if !path.exists() {
            return Ok(None);
        }

        let (tx, rx) = mpsc::channel();
        let mut watcher = notify::recommended_watcher(move |result: notify::Result<Event>| {
            if let Ok(event) = result {
                for path in event.paths {
                    if path.extension().and_then(|ext| ext.to_str()) == Some("wgsl") {
                        let _ = tx.send(path);
                    }
                }
            }
        })
        .context("failed to create file watcher")?;

        watcher
            .watch(path, RecursiveMode::Recursive)
            .context("failed to watch shader directory")?;

        Ok(Some(Self {
            receiver: rx,
            _watcher: watcher,
        }))
    }

    pub fn drain(&self) -> Vec<PathBuf> {
        let mut changed = Vec::new();
        while let Ok(path) = self.receiver.try_recv() {
            changed.push(path);
        }
        changed
    }
}
