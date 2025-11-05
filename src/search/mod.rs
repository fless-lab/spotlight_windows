pub mod engine;
pub mod cache;

pub use engine::SearchEngine;

use serde::{Deserialize, Serialize};
use std::path::PathBuf;

/// Résultat de recherche
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SearchResult {
    pub path: PathBuf,
    pub name: String,
    pub extension: Option<String>,
    pub size: u64,
    pub modified: i64,
    pub is_directory: bool,
    pub score: f32,
}

impl SearchResult {
    /// Formatte la taille en format lisible
    pub fn formatted_size(&self) -> String {
        if self.is_directory {
            return "Dossier".to_string();
        }

        let size = self.size as f64;
        if size < 1024.0 {
            format!("{} B", size)
        } else if size < 1024.0 * 1024.0 {
            format!("{:.2} KB", size / 1024.0)
        } else if size < 1024.0 * 1024.0 * 1024.0 {
            format!("{:.2} MB", size / (1024.0 * 1024.0))
        } else {
            format!("{:.2} GB", size / (1024.0 * 1024.0 * 1024.0))
        }
    }

    /// Formatte la date de modification
    pub fn formatted_modified(&self) -> String {
        let datetime = chrono::DateTime::from_timestamp(self.modified, 0)
            .unwrap_or_else(|| chrono::DateTime::from_timestamp(0, 0).unwrap());
        datetime.format("%Y-%m-%d %H:%M:%S").to_string()
    }
}
