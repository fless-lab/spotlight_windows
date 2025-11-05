mod config;
mod indexer;
mod search;
mod ui;

// Hotkey désactivé pour cross-compile
// Compiler nativement sur Windows pour l'activer
// #[cfg(windows)]
// mod hotkey;

use anyhow::Result;
use config::Config;
use indexer::{scanner::Scanner, watcher::FileWatcher, Indexer};
use search::SearchEngine;
use std::sync::Arc;
use tracing::{error, info};
use tracing_subscriber::EnvFilter;
use ui::SpotlightApp;

#[tokio::main]
async fn main() -> Result<()> {
    // Configuration du logging
    tracing_subscriber::fmt()
        .with_env_filter(
            EnvFilter::try_from_default_env().unwrap_or_else(|_| EnvFilter::new("info")),
        )
        .init();

    info!("🚀 Démarrage de Spotlight Windows");

    // Charger la configuration
    let config = Arc::new(Config::load()?);
    info!("Configuration chargée");
    info!("📁 Chemins à indexer:");
    for path in &config.indexer.include_paths {
        info!("  - {:?}", path);
    }

    // Créer l'indexeur Tantivy
    let indexer = Arc::new(Indexer::new(config.clone())?);
    info!("Indexeur créé");

    // Scanner initial des fichiers
    let scanner = Scanner::new(config.clone(), indexer.clone());
    info!("Lancement du scan initial...");

    let scanner_clone = scanner;
    let indexer_clone = indexer.clone();
    let config_clone = config.clone();

    tokio::spawn(async move {
        if let Err(e) = scanner_clone.initial_scan().await {
            error!("Erreur lors du scan initial: {}", e);
        } else {
            info!("✅ Scan initial terminé avec succès");

            // Démarrer le file watcher après le scan initial
            let watcher = FileWatcher::new(config_clone.clone(), indexer_clone);
            if let Err(e) = watcher.start().await {
                error!("Erreur lors du démarrage du file watcher: {}", e);
            } else {
                info!("✅ File watcher démarré");
            }
        }
    });

    // Créer le moteur de recherche
    let search_engine = Arc::new(SearchEngine::new(indexer.clone(), config.clone()));
    info!("Moteur de recherche créé");

    // Créer l'application UI
    let app = SpotlightApp::new(search_engine.clone());

    // Configuration de la fenêtre eframe
    let native_options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default()
            .with_inner_size([config.ui.window_width, config.ui.window_height])
            .with_decorations(true)
            .with_transparent(false)
            .with_resizable(true)
            .with_always_on_top(),
        ..Default::default()
    };

    info!("Lancement de l'interface utilisateur");

    // Note: Le hotkey global nécessite une implémentation plus avancée
    // pour interagir avec l'application eframe. Pour l'instant, l'application
    // reste visible. Une future version pourrait utiliser un système de
    // messages pour communiquer entre le thread du hotkey et l'UI.

    // Lancer l'application
    eframe::run_native(
        "Spotlight Windows",
        native_options,
        Box::new(|_cc| Ok(Box::new(app))),
    )
    .map_err(|e| anyhow::anyhow!("Erreur eframe: {}", e))?;

    Ok(())
}
