use serde::{Deserialize, Serialize};
use std::path::PathBuf;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Config {
    pub indexer: IndexerConfig,
    pub ui: UiConfig,
    pub search: SearchConfig,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct IndexerConfig {
    /// Dossiers à indexer
    pub include_paths: Vec<PathBuf>,

    /// Dossiers à exclure
    pub exclude_paths: Vec<String>,

    /// Extensions de fichiers à indexer (vide = tous)
    pub file_extensions: Vec<String>,

    /// Nombre de threads pour l'indexation
    pub num_threads: usize,

    /// Taille maximale des fichiers à indexer (en MB)
    pub max_file_size_mb: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UiConfig {
    /// Largeur de la fenêtre
    pub window_width: f32,

    /// Hauteur de la fenêtre
    pub window_height: f32,

    /// Nombre de résultats à afficher
    pub max_results: usize,

    /// Hotkey pour ouvrir (défaut: Alt+Space)
    pub hotkey_modifiers: Vec<String>,
    pub hotkey_key: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SearchConfig {
    /// Taille du cache
    pub cache_size: u64,

    /// Score minimum pour fuzzy matching
    pub min_fuzzy_score: i64,

    /// Recherche dans le contenu des fichiers
    pub search_file_content: bool,
}

impl Default for Config {
    fn default() -> Self {
        Self {
            indexer: IndexerConfig {
                include_paths: vec![
                    PathBuf::from("C:\\Users"),
                    PathBuf::from("C:\\Program Files"),
                    PathBuf::from("C:\\ProgramData"),
                ],
                exclude_paths: vec![
                    "node_modules".to_string(),
                    ".git".to_string(),
                    "target".to_string(),
                    "$RECYCLE.BIN".to_string(),
                    "AppData\\Local\\Temp".to_string(),
                ],
                file_extensions: vec![],
                num_threads: num_cpus::get(),
                max_file_size_mb: 100,
            },
            ui: UiConfig {
                window_width: 800.0,
                window_height: 600.0,
                max_results: 50,
                hotkey_modifiers: vec!["Alt".to_string()],
                hotkey_key: "Space".to_string(),
            },
            search: SearchConfig {
                cache_size: 1000,
                min_fuzzy_score: 50,
                search_file_content: false,
            },
        }
    }
}

impl Config {
    pub fn load() -> anyhow::Result<Self> {
        let config_path = Self::config_path();

        if config_path.exists() {
            let content = std::fs::read_to_string(&config_path)?;
            let config: Config = toml::from_str(&content)?;
            Ok(config)
        } else {
            let config = Config::default();
            config.save()?;
            Ok(config)
        }
    }

    pub fn save(&self) -> anyhow::Result<()> {
        let config_path = Self::config_path();

        if let Some(parent) = config_path.parent() {
            std::fs::create_dir_all(parent)?;
        }

        let content = toml::to_string_pretty(self)?;
        std::fs::write(config_path, content)?;
        Ok(())
    }

    fn config_path() -> PathBuf {
        let mut path = dirs::config_dir().unwrap_or_else(|| PathBuf::from("."));
        path.push("spotlight_windows");
        path.push("config.toml");
        path
    }
}

// Ajout de num_cpus et dirs comme dépendances
