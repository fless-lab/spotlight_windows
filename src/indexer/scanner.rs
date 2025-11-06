use super::{FileEntry, Indexer};
use crate::config::Config;
use anyhow::Result;
use ignore::WalkBuilder;
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

            // Indexer tous les fichiers de manière concurrente avec tokio
            for entry in entries_vec {
                let indexer = indexer.clone();
                if let Err(e) = indexer.add_file(entry).await {
                    warn!("Erreur lors de l'indexation: {}", e);
                }
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

        // Extraire le contenu pour les fichiers texte
        let content = if !metadata.is_dir() {
            Self::extract_content(path, &extension, size)
        } else {
            None
        };

        Some(FileEntry {
            path: path.to_path_buf(),
            name,
            extension,
            size,
            modified,
            is_directory: metadata.is_dir(),
            content,
        })
    }

    /// Extrait le contenu d'un fichier texte ou PDF
    fn extract_content(path: &std::path::Path, extension: &Option<String>, size: u64) -> Option<String> {
        // Limite de taille: 10 MB (augmentée pour les PDF)
        const MAX_CONTENT_SIZE: u64 = 10 * 1024 * 1024;

        if size > MAX_CONTENT_SIZE {
            return None;
        }

        let ext_lower = extension.as_ref()?.to_lowercase();

        // PDF: Extraction spéciale
        if ext_lower == "pdf" {
            return Self::extract_pdf_content(path);
        }

        // Extensions de fichiers texte à indexer
        const TEXT_EXTENSIONS: &[&str] = &[
            "txt", "md", "rs", "toml", "json", "xml", "yaml", "yml",
            "js", "ts", "py", "go", "c", "cpp", "h", "hpp",
            "java", "cs", "rb", "php", "html", "css", "scss",
            "sh", "bash", "ps1", "bat", "cmd", "log", "ini", "cfg",
            "csv", "sql", "vue", "jsx", "tsx", "swift", "kt", "dart"
        ];

        // Vérifier si c'est un fichier texte
        if !TEXT_EXTENSIONS.contains(&ext_lower.as_str()) {
            return None;
        }

        // Lire le contenu texte
        std::fs::read_to_string(path).ok().map(|content| {
            // Limiter à 50,000 caractères (augmenté)
            if content.len() > 50_000 {
                content.chars().take(50_000).collect()
            } else {
                content
            }
        })
    }

    /// Extrait le texte d'un PDF
    fn extract_pdf_content(path: &std::path::Path) -> Option<String> {
        use pdf_extract::extract_text;

        match extract_text(path) {
            Ok(text) => {
                // Limiter à 50,000 caractères
                let trimmed = if text.len() > 50_000 {
                    text.chars().take(50_000).collect()
                } else {
                    text
                };
                Some(trimmed)
            }
            Err(e) => {
                warn!("Impossible d'extraire le PDF {:?}: {}", path, e);
                None
            }
        }
    }
}
