use super::{FileEntry, Indexer};
use crate::config::Config;
use anyhow::Result;
use ignore::WalkBuilder;
use rayon::prelude::*;
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::Arc;
use std::time::UNIX_EPOCH;
use tracing::{info, warn};

/// Scanner de fichiers ultra-rapide avec parallélisme
pub struct Scanner {
    config: Arc<Config>,
    indexer: Arc<Indexer>,
}

impl Scanner {
    pub fn new(config: Arc<Config>, indexer: Arc<Indexer>) -> Self {
        Self { config, indexer }
    }

    /// Scan initial de tous les fichiers
    pub async fn initial_scan(&self) -> Result<()> {
        info!("Démarrage du scan initial...");
        let start = std::time::Instant::now();

        let indexed_count = Arc::new(AtomicU64::new(0));

        for root_path in &self.config.indexer.include_paths {
            if !root_path.exists() {
                warn!("Chemin introuvable: {:?}", root_path);
                continue;
            }

            info!("Scan de: {:?}", root_path);

            // Créer le walker avec ignore pour respecter .gitignore, etc.
            let walker = WalkBuilder::new(root_path)
                .threads(self.config.indexer.num_threads)
                .hidden(false)
                .git_ignore(true)
                .build_parallel();

            // Collecte des entrées à indexer
            let entries: Arc<std::sync::Mutex<Vec<FileEntry>>> =
                Arc::new(std::sync::Mutex::new(Vec::new()));

            let config = self.config.clone();
            let entries_clone = entries.clone();
            let indexed_count_clone = indexed_count.clone();

            walker.run(|| {
                let config = config.clone();
                let entries = entries_clone.clone();
                let indexed_count = indexed_count_clone.clone();

                Box::new(move |result| {
                    if let Ok(entry) = result {
                        let path = entry.path();

                        // Vérifier si le chemin doit être exclu
                        if Self::should_exclude(path, &config.indexer.exclude_paths) {
                            return ignore::WalkState::Skip;
                        }

                        // Extraire les métadonnées
                        if let Ok(metadata) = entry.metadata() {
                            if let Some(file_entry) = Self::create_file_entry(path, &metadata) {
                                // Vérifier la taille du fichier
                                if !metadata.is_dir()
                                    && file_entry.size > config.indexer.max_file_size_mb * 1024 * 1024
                                {
                                    return ignore::WalkState::Continue;
                                }

                                entries.lock().unwrap().push(file_entry);
                                indexed_count.fetch_add(1, Ordering::Relaxed);
                            }
                        }
                    }
                    ignore::WalkState::Continue
                })
            });

            // Indexer tous les fichiers en parallèle avec rayon
            let entries_vec = {
                let entries = entries.lock().unwrap();
                entries.clone() // Cloner pour libérer le lock
            };

            let indexer = self.indexer.clone();

            info!("Indexation de {} fichiers...", entries_vec.len());

            // Utiliser rayon pour paralléliser l'indexation
            let futures: Vec<_> = entries_vec
                .par_iter()
                .map(|entry| {
                    let indexer = indexer.clone();
                    let entry = entry.clone();
                    tokio::spawn(async move {
                        if let Err(e) = indexer.add_file(entry).await {
                            warn!("Erreur lors de l'indexation: {}", e);
                        }
                    })
                })
                .collect();

            // Attendre la fin de toutes les tâches
            for future in futures {
                let _ = future.await;
            }
        }

        // Commit l'index
        self.indexer.commit().await?;

        let elapsed = start.elapsed();
        let count = indexed_count.load(Ordering::Relaxed);

        info!(
            "Scan initial terminé: {} fichiers indexés en {:.2}s ({:.0} fichiers/s)",
            count,
            elapsed.as_secs_f64(),
            count as f64 / elapsed.as_secs_f64()
        );

        Ok(())
    }

    /// Vérifie si un chemin doit être exclu
    fn should_exclude(path: &std::path::Path, exclude_patterns: &[String]) -> bool {
        let path_str = path.to_string_lossy();

        for pattern in exclude_patterns {
            if path_str.contains(pattern) {
                return true;
            }
        }

        false
    }

    /// Crée une FileEntry à partir des métadonnées
    fn create_file_entry(path: &std::path::Path, metadata: &std::fs::Metadata) -> Option<FileEntry> {
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
