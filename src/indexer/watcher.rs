use super::{FileEntry, Indexer};
use crate::config::Config;
use anyhow::Result;
use notify::{Config as NotifyConfig, Event, EventKind, RecommendedWatcher, RecursiveMode, Watcher};
use std::path::PathBuf;
use std::sync::Arc;
use std::time::UNIX_EPOCH;
use tokio::sync::mpsc;
use tracing::{debug, error, info};

/// File watcher temps réel
pub struct FileWatcher {
    config: Arc<Config>,
    indexer: Arc<Indexer>,
}

impl FileWatcher {
    pub fn new(config: Arc<Config>, indexer: Arc<Indexer>) -> Self {
        Self { config, indexer }
    }

    /// Démarre le watching des fichiers
    pub async fn start(&self) -> Result<()> {
        info!("Démarrage du file watcher...");

        let (tx, mut rx) = mpsc::channel(1000);

        // Créer le watcher
        let mut watcher = RecommendedWatcher::new(
            move |res: Result<Event, notify::Error>| {
                if let Ok(event) = res {
                    let _ = tx.blocking_send(event);
                }
            },
            NotifyConfig::default(),
        )?;

        // Watcher tous les chemins configurés
        for path in &self.config.indexer.include_paths {
            if path.exists() {
                watcher.watch(path, RecursiveMode::Recursive)?;
                info!("Watching: {:?}", path);
            }
        }

        let indexer = self.indexer.clone();
        let exclude_paths = self.config.indexer.exclude_paths.clone();

        // Traiter les événements
        tokio::spawn(async move {
            while let Some(event) = rx.recv().await {
                if let Err(e) = Self::handle_event(event, &indexer, &exclude_paths).await {
                    error!("Erreur lors du traitement de l'événement: {}", e);
                }
            }
        });

        // Garder le watcher en vie
        std::mem::forget(watcher);

        Ok(())
    }

    /// Traite un événement de fichier
    async fn handle_event(
        event: Event,
        indexer: &Arc<Indexer>,
        exclude_paths: &[String],
    ) -> Result<()> {
        match event.kind {
            EventKind::Create(_) | EventKind::Modify(_) => {
                for path in event.paths {
                    if Self::should_exclude(&path, exclude_paths) {
                        continue;
                    }

                    if let Ok(metadata) = std::fs::metadata(&path) {
                        if let Some(entry) = Self::create_file_entry(&path, &metadata) {
                            debug!("Fichier ajouté/modifié: {:?}", path);
                            indexer.add_file(entry).await?;
                        }
                    }
                }

                // Commit périodique
                indexer.commit().await?;
            }
            EventKind::Remove(_) => {
                for path in event.paths {
                    debug!("Fichier supprimé: {:?}", path);
                    indexer.remove_file(&path).await?;
                }

                indexer.commit().await?;
            }
            _ => {}
        }

        Ok(())
    }

    /// Vérifie si un chemin doit être exclu
    fn should_exclude(path: &PathBuf, exclude_patterns: &[String]) -> bool {
        let path_str = path.to_string_lossy();

        for pattern in exclude_patterns {
            if path_str.contains(pattern) {
                return true;
            }
        }

        false
    }

    /// Crée une FileEntry à partir des métadonnées
    fn create_file_entry(path: &PathBuf, metadata: &std::fs::Metadata) -> Option<FileEntry> {
        let name = path.file_name()?.to_string_lossy().to_string();

        let extension = if !metadata.is_dir() {
            path.extension().map(|e| e.to_string_lossy().to_string())
        } else {
            None
        };

        let size = metadata.len();

        let modified = metadata
            .modified()
            .ok()?
            .duration_since(UNIX_EPOCH)
            .ok()?
            .as_secs() as i64;

        Some(FileEntry {
            path: path.to_path_buf(),
            name,
            extension,
            size,
            modified,
            is_directory: metadata.is_dir(),
        })
    }
}
