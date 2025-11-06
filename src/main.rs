// Désactiver la console Windows pour un comportement d'application native
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

mod config;
mod indexer;
mod search;
mod tray;
mod ui;

use anyhow::Result;
use config::Config;
use global_hotkey::{hotkey::{HotKey, Code, Modifiers}, GlobalHotKeyManager, GlobalHotKeyEvent};
use indexer::{scanner::Scanner, watcher::FileWatcher, Indexer};
use search::SearchEngine;
use std::sync::{Arc, mpsc::{channel, Sender}};
use tracing::{error, info};
use tracing_subscriber::EnvFilter;
use tray::SystemTray;
use ui::SpotlightUI; // Nouvelle UI premium

/// Events pour contrôler la visibilité de la fenêtre
#[derive(Debug, Clone)]
pub enum WindowEvent {
    Show,
    Hide,
    Toggle,
    Quit,
}

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

    // Vérifier si l'index contient déjà des documents
    let num_docs = indexer.num_documents();
    if num_docs > 0 {
        info!("✅ Index existant trouvé avec {} documents", num_docs);
        info!("⏭️  Scan initial ignoré (index déjà peuplé)");

        // Démarrer directement le file watcher
        let indexer_clone = indexer.clone();
        let config_clone = config.clone();
        tokio::spawn(async move {
            let watcher = FileWatcher::new(config_clone, indexer_clone);
            if let Err(e) = watcher.start().await {
                error!("Erreur lors du démarrage du file watcher: {}", e);
            } else {
                info!("✅ File watcher démarré");
            }
        });
    } else {
        info!("📊 Index vide - Lancement du scan initial...");

        // Scanner initial des fichiers
        let scanner = Scanner::new(config.clone(), indexer.clone());

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
    }

    // Créer le moteur de recherche
    let search_engine = Arc::new(SearchEngine::new(indexer.clone(), config.clone()));
    info!("Moteur de recherche créé");

    // Créer le canal pour les événements de fenêtre
    let (window_event_tx, window_event_rx) = channel::<WindowEvent>();

    // Initialiser le system tray
    let system_tray = match SystemTray::new() {
        Ok(tray) => {
            info!("✅ System tray initialisé");
            Some(tray)
        }
        Err(e) => {
            error!("⚠️  Erreur lors de l'initialisation du system tray: {}", e);
            None
        }
    };

    // Initialiser le hotkey manager (Ctrl+Space)
    let hotkey_manager = GlobalHotKeyManager::new().ok();
    let hotkey_id = if let Some(ref manager) = hotkey_manager {
        let hotkey = HotKey::new(Some(Modifiers::CONTROL), Code::Space);
        match manager.register(hotkey) {
            Ok(_) => {
                info!("✅ Hotkey Ctrl+Space enregistré");
                Some(hotkey.id())
            }
            Err(e) => {
                error!("⚠️  Erreur lors de l'enregistrement du hotkey: {}", e);
                None
            }
        }
    } else {
        error!("⚠️  Impossible d'initialiser le hotkey manager");
        None
    };

    // Thread pour écouter uniquement les événements du hotkey
    let window_event_tx_clone = window_event_tx.clone();
    std::thread::spawn(move || {
        loop {
            // Vérifier les événements du hotkey
            if let Some(expected_id) = hotkey_id {
                if let Ok(event) = GlobalHotKeyEvent::receiver().try_recv() {
                    if event.id == expected_id {
                        let _ = window_event_tx_clone.send(WindowEvent::Toggle);
                    }
                }
            }

            std::thread::sleep(std::time::Duration::from_millis(50));
        }
    });

    // Créer l'application UI Spotlight Premium
    let app = SpotlightUI::new(search_engine.clone(), window_event_rx, system_tray, window_event_tx, indexer.clone());

    // Configuration de la fenêtre style Spotlight
    let native_options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default()
            .with_inner_size([720.0, 520.0]) // Taille Spotlight
            .with_decorations(false) // Sans bordure Windows
            .with_transparent(true) // Transparent pour effet blur
            .with_resizable(false) // Taille fixe
            .with_always_on_top() // Toujours au-dessus
            .with_visible(false) // Commence caché (Ctrl+Space pour afficher)
            .with_position([
                (1920.0 - 720.0) / 2.0, // Centré horizontalement (ajuster selon résolution)
                200.0, // 24% de la hauteur ~= 200px sur 1080p
            ]),
        ..Default::default()
    };

    info!("✅ Lancement de l'interface utilisateur (fenêtre cachée - Ctrl+Space pour afficher)");

    // Lancer l'application
    eframe::run_native(
        "Spotlight Windows",
        native_options,
        Box::new(|_cc| Ok(Box::new(app))),
    )
    .map_err(|e| anyhow::anyhow!("Erreur eframe: {}", e))?;

    Ok(())
}
