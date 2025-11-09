use crate::indexer::{Indexer, IndexingProgress};
use crate::search::{SearchEngine, SearchResult};
use crate::tray::SystemTray;
use crate::WindowEvent;
use egui::{
    Align, Color32, FontId, Key, Layout, Margin, Pos2, Rect, Rounding, ScrollArea, Sense, Shadow,
    Stroke, TextEdit, TextStyle, Vec2, Visuals,
};
use std::sync::mpsc::{channel, Receiver, Sender};
use std::sync::Arc;
use std::time::Instant;
use tracing::info;

/// État de l'application
#[derive(Debug, Clone, Copy, PartialEq)]
enum AppState {
    FirstRun,      // Premier lancement - écran de setup
    Indexing,      // Indexation en cours
    Ready,         // Prêt à chercher
}

#[cfg(target_os = "windows")]
use std::os::windows::process::CommandExt;

// === CONSTANTES DE DESIGN SPOTLIGHT PREMIUM ===
const WINDOW_WIDTH: f32 = 720.0;
const SEARCH_BAR_HEIGHT: f32 = 68.0;
const RESULT_ITEM_HEIGHT: f32 = 64.0;
const ANIMATION_DURATION: f32 = 0.18; // 180ms animations fluides
const ICON_SIZE: f32 = 40.0;
const BORDER_RADIUS: f32 = 16.0; // Plus arrondi
const ITEM_SPACING: f32 = 4.0;

// Fonctions helper pour créer les couleurs
fn bg_main() -> Color32 {
    Color32::from_rgba_unmultiplied(20, 20, 22, 252)
}
fn bg_search() -> Color32 {
    Color32::from_rgba_unmultiplied(32, 32, 36, 255)
}
fn bg_result() -> Color32 {
    Color32::from_rgba_unmultiplied(28, 28, 32, 255)
}
fn bg_selection() -> Color32 {
    Color32::from_rgba_unmultiplied(0, 122, 255, 45)
}
fn bg_hover() -> Color32 {
    Color32::from_rgba_unmultiplied(255, 255, 255, 8)
}

const TEXT_PRIMARY: Color32 = Color32::WHITE;
const TEXT_SECONDARY: Color32 = Color32::from_gray(160);
const TEXT_TERTIARY: Color32 = Color32::from_gray(110);
const ACCENT_BLUE: Color32 = Color32::from_rgb(10, 132, 255);

/// Spotlight UI Premium - Design exceptionnel
pub struct SpotlightUI {
    // Core
    search_engine: Arc<SearchEngine>,
    indexer: Arc<Indexer>,
    query_sender: Sender<String>,
    result_receiver: Receiver<Vec<SearchResult>>,
    window_event_receiver: Receiver<WindowEvent>,
    window_event_sender: Sender<WindowEvent>,
    system_tray: Option<SystemTray>,

    // État
    app_state: AppState,
    query: String,
    results: Vec<SearchResult>,
    selected_index: usize,
    is_searching: bool,
    is_visible: bool,

    // Animation
    animation_progress: f32,
    last_frame_time: Instant,
    last_tooltip_update: Instant,
    item_hover_indices: Vec<f32>, // Animation hover pour chaque item

    // Setup
    setup_completed: bool,
}

impl SpotlightUI {
    pub fn new(
        search_engine: Arc<SearchEngine>,
        window_event_receiver: Receiver<WindowEvent>,
        system_tray: Option<SystemTray>,
        window_event_sender: Sender<WindowEvent>,
        indexer: Arc<Indexer>,
    ) -> Self {
        let (query_tx, query_rx) = channel::<String>();
        let (result_tx, result_rx) = channel::<Vec<SearchResult>>();

        // Thread de recherche async
        let search_engine_clone = search_engine.clone();
        std::thread::spawn(move || {
            let runtime = tokio::runtime::Runtime::new().unwrap();
            while let Ok(query) = query_rx.recv() {
                let search_engine = search_engine_clone.clone();
                let result_sender = result_tx.clone();
                runtime.spawn(async move {
                    let results = search_engine
                        .search(&query, 50)
                        .await
                        .unwrap_or_else(|_| vec![]);
                    let _ = result_sender.send(results);
                });
            }
        });

        let now = Instant::now();

        // Vérifier si le setup a été complété
        let setup_completed = Self::is_setup_completed();
        let app_state = if !setup_completed {
            AppState::FirstRun
        } else if indexer.num_documents() == 0 {
            AppState::FirstRun // Si pas de documents, considérer comme premier lancement
        } else {
            AppState::Ready
        };

        Self {
            search_engine,
            indexer,
            query_sender: query_tx,
            result_receiver: result_rx,
            window_event_receiver,
            window_event_sender,
            system_tray,
            app_state,
            query: String::new(),
            results: Vec::new(),
            selected_index: 0,
            is_searching: false,
            is_visible: true, // Démarre visible (Ctrl+Space nécessite compilation Windows native)
            animation_progress: 1.0, // Déjà animé
            last_frame_time: now,
            last_tooltip_update: now,
            item_hover_indices: vec![],
            setup_completed,
        }
    }

    /// Vérifie si le setup a été complété
    fn is_setup_completed() -> bool {
        let config_path = Self::setup_config_path();
        config_path.exists()
    }

    /// Retourne le chemin du fichier de configuration setup
    fn setup_config_path() -> std::path::PathBuf {
        let mut path = dirs::data_dir().unwrap_or_else(|| std::path::PathBuf::from("."));
        path.push("spotlight_windows");
        path.push("setup_completed");
        path
    }

    /// Marque le setup comme complété
    fn mark_setup_completed() -> anyhow::Result<()> {
        let config_path = Self::setup_config_path();
        if let Some(parent) = config_path.parent() {
            std::fs::create_dir_all(parent)?;
        }
        std::fs::write(&config_path, "1")?;
        Ok(())
    }

    fn perform_search(&mut self) {
        if self.query.is_empty() {
            self.results.clear();
            return;
        }
        let _ = self.query_sender.send(self.query.clone());
        self.is_searching = true;
    }

    fn open_file(&mut self, result: &SearchResult, ctx: &egui::Context) {
        #[cfg(target_os = "windows")]
        {
            let _ = std::process::Command::new("cmd")
                .args(&["/c", "start", "", &result.path.to_string_lossy()])
                .creation_flags(0x08000000)
                .spawn();
        }
        // Cacher la fenêtre après ouverture
        self.is_visible = false;
        self.animation_progress = 0.0;
        ctx.send_viewport_cmd(egui::ViewportCommand::Visible(false));
        self.query.clear();
        self.results.clear();
    }

    /// Easing function: ease-out cubic pour animations fluides
    fn ease_out_cubic(t: f32) -> f32 {
        1.0 - (1.0 - t).powi(3)
    }

    /// Easing function: ease-in-out cubic
    fn ease_in_out(t: f32) -> f32 {
        if t < 0.5 {
            4.0 * t * t * t
        } else {
            1.0 - (-2.0 * t + 2.0).powi(3) / 2.0
        }
    }

    fn get_file_icon(&self, result: &SearchResult) -> &str {
        if result.is_directory {
            return "📁";
        }

        match result.extension.as_deref() {
            // Applications
            Some("exe") | Some("msi") => "⚙️",

            // Documents
            Some("pdf") => "📕",
            Some("docx") | Some("doc") => "📘",
            Some("xlsx") | Some("xls") => "📗",
            Some("pptx") | Some("ppt") => "📙",
            Some("txt") | Some("md") => "📝",

            // Images
            Some("jpg") | Some("jpeg") | Some("png") | Some("gif") | Some("bmp") | Some("svg") | Some("ico") => "🖼️",

            // Audio
            Some("mp3") | Some("wav") | Some("flac") | Some("m4a") | Some("aac") | Some("ogg") => "🎵",

            // Vidéo
            Some("mp4") | Some("avi") | Some("mkv") | Some("mov") | Some("wmv") | Some("flv") => "🎬",

            // Archives
            Some("zip") | Some("rar") | Some("7z") | Some("tar") | Some("gz") | Some("bz2") => "📦",

            // Code
            Some("rs") => "🦀",
            Some("py") => "🐍",
            Some("js") | Some("ts") | Some("jsx") | Some("tsx") => "📜",
            Some("java") | Some("class") | Some("jar") => "☕",
            Some("cpp") | Some("c") | Some("h") | Some("hpp") => "⚡",
            Some("go") => "🐹",
            Some("php") => "🐘",
            Some("rb") => "💎",
            Some("cs") => "🔷",

            // Web
            Some("html") | Some("htm") => "🌐",
            Some("css") | Some("scss") | Some("sass") => "🎨",
            Some("json") | Some("xml") | Some("yaml") | Some("yml") | Some("toml") => "📋",

            // Défaut
            _ => "📄",
        }
    }

    /// Écran de premier lancement (First Run Setup)
    fn render_first_run_screen(&mut self, ui: &mut egui::Ui, ctx: &egui::Context) {
        ui.add_space(40.0);

        ui.vertical_centered(|ui| {
            // Icône
            ui.label(
                egui::RichText::new("🔍")
                    .size(72.0)
                    .color(TEXT_PRIMARY),
            );
            ui.add_space(16.0);

            // Titre
            ui.label(
                egui::RichText::new("Bienvenue sur Spotlight Windows")
                    .size(24.0)
                    .color(TEXT_PRIMARY)
                    .strong(),
            );
            ui.add_space(8.0);

            // Description
            ui.label(
                egui::RichText::new("Recherche ultra-rapide de vos fichiers")
                    .size(14.0)
                    .color(TEXT_SECONDARY),
            );
            ui.add_space(32.0);

            // Explication
            egui::Frame::none()
                .fill(bg_search())
                .rounding(Rounding::same(12.0))
                .inner_margin(Margin::same(20.0))
                .show(ui, |ui| {
                    ui.set_width(600.0);
                    ui.label(
                        egui::RichText::new("🗂️  Indexation initiale")
                            .size(16.0)
                            .color(TEXT_PRIMARY)
                            .strong(),
                    );
                    ui.add_space(12.0);

                    ui.label(
                        egui::RichText::new(
                            "Spotlight va scanner vos documents pour créer un index de recherche.\n\n\
                             • Documents, Downloads, Desktop\n\
                             • Recherche instantanée (< 50ms)\n\
                             • Recherche dans le contenu des fichiers\n\
                             • Durée : 1-2 minutes selon le nombre de fichiers\n\n\
                             Cette indexation ne se fait qu'une seule fois.\n\
                             Les nouveaux fichiers seront détectés automatiquement."
                        )
                        .size(13.0)
                        .color(TEXT_SECONDARY),
                    );
                });

            ui.add_space(32.0);

            // Bouton "Démarrer l'indexation"
            let button = egui::Button::new(
                egui::RichText::new("🚀  Démarrer l'indexation")
                    .size(16.0)
                    .color(Color32::WHITE)
                    .strong(),
            )
            .fill(ACCENT_BLUE)
            .rounding(Rounding::same(10.0))
            .min_size(Vec2::new(250.0, 48.0));

            if ui.add(button).clicked() {
                // Lancer l'indexation
                self.app_state = AppState::Indexing;

                // Lancer le scan initial en arrière-plan
                let indexer = self.indexer.clone();
                let config = std::sync::Arc::new(
                    crate::config::Config::load().unwrap_or_default()
                );
                tokio::spawn(async move {
                    let scanner = crate::indexer::scanner::Scanner::new(config.clone(), indexer.clone());
                    if let Err(e) = scanner.initial_scan().await {
                        tracing::error!("Erreur lors du scan initial: {}", e);
                    } else {
                        tracing::info!("✅ Scan initial terminé avec succès");

                        // Démarrer le file watcher après le scan initial
                        let watcher = crate::indexer::watcher::FileWatcher::new(config, indexer);
                        if let Err(e) = watcher.start().await {
                            tracing::error!("Erreur lors du démarrage du file watcher: {}", e);
                        } else {
                            tracing::info!("✅ File watcher démarré");
                        }
                    }
                });
            }

            ui.add_space(40.0);
        });
    }

    /// Écran d'indexation en cours avec progression
    fn render_indexing_screen(&mut self, ui: &mut egui::Ui, ctx: &egui::Context) {
        // Vérifier la progression
        let progress = self.indexer.get_progress();

        // Si terminé, passer à Ready
        if progress.is_complete && progress.current > 0 {
            self.app_state = AppState::Ready;
            let _ = Self::mark_setup_completed();
            self.setup_completed = true;
            return;
        }

        ui.add_space(40.0);

        ui.vertical_centered(|ui| {
            // Icône animée
            ui.label(
                egui::RichText::new("⏳")
                    .size(64.0)
                    .color(ACCENT_BLUE),
            );
            ui.add_space(16.0);

            // Titre
            ui.label(
                egui::RichText::new("Indexation en cours...")
                    .size(22.0)
                    .color(TEXT_PRIMARY)
                    .strong(),
            );
            ui.add_space(24.0);

            // Compteur
            ui.label(
                egui::RichText::new(format!(
                    "📊  {} fichiers scannés",
                    progress.total
                ))
                .size(18.0)
                .color(TEXT_SECONDARY),
            );

            if progress.current > 0 {
                ui.add_space(8.0);
                ui.label(
                    egui::RichText::new(format!(
                        "✅  {} fichiers indexés",
                        progress.current
                    ))
                    .size(15.0)
                    .color(TEXT_TERTIARY),
                );
            }

            ui.add_space(32.0);

            // Barre de progression (indéterminée si total == 0)
            egui::Frame::none()
                .fill(bg_search())
                .rounding(Rounding::same(8.0))
                .inner_margin(Margin::symmetric(4.0, 4.0))
                .show(ui, |ui| {
                    ui.set_width(500.0);
                    ui.set_height(8.0);

                    if progress.total > 0 {
                        let ratio = progress.current as f32 / progress.total as f32;
                        let filled_width = 492.0 * ratio;

                        let (_, painter) = ui.allocate_painter(
                            Vec2::new(492.0, 8.0),
                            Sense::hover(),
                        );

                        painter.rect_filled(
                            egui::Rect::from_min_size(
                                painter.clip_rect().min,
                                Vec2::new(filled_width, 8.0),
                            ),
                            Rounding::same(6.0),
                            ACCENT_BLUE,
                        );
                    }
                });

            ui.add_space(24.0);

            // Message
            ui.label(
                egui::RichText::new("Veuillez patienter, cela peut prendre 1-2 minutes...")
                    .size(13.0)
                    .color(TEXT_TERTIARY)
                    .italics(),
            );

            ui.add_space(40.0);
        });
    }

    /// UI de recherche normale (état Ready)
    fn render_search_ui(&mut self, ui: &mut egui::Ui, ctx: &egui::Context) {
        ui.add_space(16.0);

        // === BARRE DE RECHERCHE ===
        ui.add_space(4.0);
        ui.horizontal(|ui| {
            ui.add_space(16.0);
            ui.vertical(|ui| {
                ui.set_width(WINDOW_WIDTH - 32.0);
                self.render_search_bar(ui, ctx);
            });
        });
        ui.add_space(12.0);

        // === SÉPARATEUR ===
        if !self.results.is_empty() {
            ui.horizontal(|ui| {
                ui.add_space(16.0);
                ui.separator();
            });
            ui.add_space(8.0);
        }

        // === RÉSULTATS ===
        ui.horizontal(|ui| {
            ui.add_space(16.0);
            ui.vertical(|ui| {
                ui.set_width(WINDOW_WIDTH - 32.0);
                self.render_results(ui, ctx);
            });
        });

        ui.add_space(16.0);
    }

    fn render_search_bar(&mut self, ui: &mut egui::Ui, _ctx: &egui::Context) {
        let search_frame = egui::Frame::none()
            .fill(bg_search())
            .rounding(Rounding::same(12.0))
            .inner_margin(Margin::symmetric(20.0, 16.0));

        search_frame.show(ui, |ui| {
            ui.horizontal(|ui| {
                ui.spacing_mut().item_spacing.x = 12.0;

                // Icône de recherche
                ui.label(
                    egui::RichText::new("🔍")
                        .size(22.0)
                        .color(TEXT_SECONDARY),
                );

                // Champ de recherche
                let text_edit = TextEdit::singleline(&mut self.query)
                    .font(FontId::proportional(18.0))
                    .text_color(TEXT_PRIMARY)
                    .hint_text(
                        egui::RichText::new("Rechercher...")
                            .color(TEXT_TERTIARY)
                            .size(18.0),
                    )
                    .frame(false)
                    .desired_width(f32::INFINITY);

                let response = ui.add(text_edit);

                // Auto-focus
                if self.is_visible {
                    response.request_focus();
                }

                // Recherche en temps réel
                if response.changed() {
                    self.perform_search();
                }

                // Indicateur de recherche
                if self.is_searching && !self.results.is_empty() {
                    ui.spinner();
                }
            });
        });
    }

    fn render_result_item(
        &self,
        ui: &mut egui::Ui,
        _ctx: &egui::Context,
        result: &SearchResult,
        index: usize,
        is_selected: bool,
        hover_progress: f32,
    ) -> egui::Response {

        // Background avec animation
        let bg_color = if is_selected {
            bg_selection()
        } else if hover_progress > 0.01 {
            // Interpolation douce entre normal et hover
            let t = Self::ease_out_cubic(hover_progress);
            let bg_res = bg_result();
            let bg_hov = bg_hover();
            Color32::from_rgba_unmultiplied(
                ((1.0 - t) * bg_res.r() as f32 + t * bg_hov.r() as f32) as u8,
                ((1.0 - t) * bg_res.g() as f32 + t * bg_hov.g() as f32) as u8,
                ((1.0 - t) * bg_res.b() as f32 + t * bg_hov.b() as f32) as u8,
                bg_res.a(),
            )
        } else {
            Color32::TRANSPARENT
        };

        let frame = egui::Frame::none()
            .fill(bg_color)
            .rounding(Rounding::same(10.0))
            .inner_margin(Margin::symmetric(16.0, 12.0));

        let response = frame
            .show(ui, |ui| {
                ui.horizontal(|ui| {
                    ui.spacing_mut().item_spacing.x = 14.0;

                    // Icône du fichier
                    let icon = self.get_file_icon(result);
                    ui.label(egui::RichText::new(icon).size(32.0));

                    // Informations
                    ui.vertical(|ui| {
                        ui.spacing_mut().item_spacing.y = 2.0;

                        // Nom du fichier
                        let filename = result
                            .path
                            .file_name()
                            .and_then(|n| n.to_str())
                            .unwrap_or("Unknown");

                        ui.label(
                            egui::RichText::new(filename)
                                .size(15.0)
                                .color(TEXT_PRIMARY)
                                .strong(),
                        );

                        // Chemin
                        let path_str = result
                            .path
                            .parent()
                            .and_then(|p| p.to_str())
                            .unwrap_or("");

                        // Tronquer le chemin si trop long
                        let display_path = if path_str.len() > 60 {
                            format!("...{}", &path_str[path_str.len() - 57..])
                        } else {
                            path_str.to_string()
                        };

                        ui.label(
                            egui::RichText::new(display_path)
                                .size(12.0)
                                .color(TEXT_SECONDARY),
                        );
                    });
                });
            })
            .response;

        response
    }

    fn render_results(&mut self, ui: &mut egui::Ui, ctx: &egui::Context) {
        if self.query.is_empty() {
            // Message d'accueil
            ui.add_space(40.0);
            ui.vertical_centered(|ui| {
                let num_docs = self.indexer.num_documents();

                // Icône et message principal
                ui.label(
                    egui::RichText::new("🔍")
                        .size(48.0)
                        .color(TEXT_TERTIARY),
                );
                ui.add_space(12.0);
                ui.label(
                    egui::RichText::new("Tapez pour rechercher")
                        .size(16.0)
                        .color(TEXT_SECONDARY),
                );

                // Compteur de fichiers indexés
                if num_docs > 0 {
                    ui.label(
                        egui::RichText::new(format!("{} fichiers indexés", num_docs))
                            .size(13.0)
                            .color(TEXT_TERTIARY),
                    );
                } else {
                    ui.label(
                        egui::RichText::new("Indexation en cours...")
                            .size(13.0)
                            .color(TEXT_TERTIARY),
                    );
                }
            });
            return;
        }

        if self.results.is_empty() && !self.is_searching {
            // Aucun résultat
            ui.add_space(40.0);
            ui.vertical_centered(|ui| {
                ui.label(
                    egui::RichText::new("❌")
                        .size(40.0)
                        .color(TEXT_TERTIARY),
                );
                ui.add_space(8.0);
                ui.label(
                    egui::RichText::new("Aucun résultat")
                        .size(15.0)
                        .color(TEXT_SECONDARY),
                );
            });
            return;
        }

        // Liste des résultats avec scroll
        let mut clicked_index: Option<usize> = None;
        let mut hovered_states: Vec<(usize, bool)> = Vec::new();

        // S'assurer que item_hover_indices a la bonne taille
        while self.item_hover_indices.len() < self.results.len() {
            self.item_hover_indices.push(0.0);
        }

        ScrollArea::vertical()
            .max_height(400.0)
            .auto_shrink([false, true])
            .show(ui, |ui| {
                ui.spacing_mut().item_spacing.y = ITEM_SPACING;

                for (index, result) in self.results.iter().enumerate() {
                    let is_selected = index == self.selected_index;
                    let hover_progress = self.item_hover_indices.get(index).copied().unwrap_or(0.0);
                    let response =
                        self.render_result_item(ui, ctx, result, index, is_selected, hover_progress);

                    if response.clicked() {
                        clicked_index = Some(index);
                    }

                    // Enregistrer l'état hover pour mise à jour après
                    hovered_states.push((index, response.hovered()));
                }
            });

        // Mettre à jour les animations hover
        for (index, is_hovered) in hovered_states {
            if is_hovered {
                self.item_hover_indices[index] =
                    (self.item_hover_indices[index] + 0.15).min(1.0);
            } else {
                self.item_hover_indices[index] =
                    (self.item_hover_indices[index] - 0.1).max(0.0);
            }
        }

        // Gérer le clic après le ScrollArea (évite les problèmes de borrow)
        if let Some(idx) = clicked_index {
            self.selected_index = idx;
            if let Some(result) = self.results.get(idx).cloned() {
                self.open_file(&result, ctx);
            }
        }
    }
}

impl eframe::App for SpotlightUI {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        // === MISE À JOUR TOOLTIP TRAY ===
        if self.last_tooltip_update.elapsed().as_secs() >= 2 {
            if let Some(ref tray) = self.system_tray {
                let num_docs = self.indexer.num_documents();
                let tooltip = if num_docs > 0 {
                    format!("Spotlight Windows - {} fichiers indexés", num_docs)
                } else {
                    "Spotlight Windows - Indexation en cours...".to_string()
                };
                tray.set_tooltip(&tooltip);
            }
            self.last_tooltip_update = Instant::now();
        }

        // === ÉVÉNEMENTS SYSTEM TRAY ===
        if let Some(ref tray) = self.system_tray {
            if let Some(tray_event) = tray.try_recv_event() {
                let event = match tray_event {
                    crate::tray::TrayEvent::Show => WindowEvent::Show,
                    crate::tray::TrayEvent::Quit => WindowEvent::Quit,
                };
                let _ = self.window_event_sender.send(event);
            }
        }

        // === ÉVÉNEMENTS FENÊTRE (CTRL+SPACE) ===
        while let Ok(event) = self.window_event_receiver.try_recv() {
            match event {
                WindowEvent::Show => {
                    self.is_visible = true;
                    self.animation_progress = 0.0;
                    ctx.send_viewport_cmd(egui::ViewportCommand::Visible(true));
                    ctx.send_viewport_cmd(egui::ViewportCommand::Focus);
                }
                WindowEvent::Hide => {
                    self.is_visible = false;
                    self.animation_progress = 0.0;
                    ctx.send_viewport_cmd(egui::ViewportCommand::Visible(false));
                }
                WindowEvent::Toggle => {
                    self.is_visible = !self.is_visible;
                    self.animation_progress = 0.0;
                    ctx.send_viewport_cmd(egui::ViewportCommand::Visible(self.is_visible));
                    if self.is_visible {
                        ctx.send_viewport_cmd(egui::ViewportCommand::Focus);
                    }
                }
                WindowEvent::Quit => {
                    std::process::exit(0);
                }
            }
        }

        // === RÉSULTATS RECHERCHE ===
        if let Ok(results) = self.result_receiver.try_recv() {
            self.results = results;
            self.is_searching = false;
            self.selected_index = 0;
            self.item_hover_indices.clear();
        }

        // === ANIMATION PROGRESS ===
        let delta = self.last_frame_time.elapsed().as_secs_f32();
        self.last_frame_time = Instant::now();
        if self.is_visible {
            self.animation_progress = (self.animation_progress + delta / ANIMATION_DURATION).min(1.0);
        }

        // Ne pas dessiner si invisible
        if !self.is_visible && self.animation_progress < 0.01 {
            return;
        }

        // === STYLE GLOBAL ===
        let mut style = (*ctx.style()).clone();
        style.visuals.window_fill = Color32::TRANSPARENT;
        style.visuals.panel_fill = Color32::TRANSPARENT;
        style.visuals.window_stroke = Stroke::NONE;
        ctx.set_style(style);

        // === GESTION CLAVIER ===
        ctx.input(|i| {
            if i.key_pressed(Key::Escape) {
                // ESC ferme complètement l'application (pas de Ctrl+Space pour rouvrir en cross-compile)
                info!("ESC pressé - Fermeture de l'application");
                std::process::exit(0);
            }

            if i.key_pressed(Key::ArrowDown) && !self.results.is_empty() {
                self.selected_index = (self.selected_index + 1).min(self.results.len() - 1);
            }

            if i.key_pressed(Key::ArrowUp) && !self.results.is_empty() {
                self.selected_index = self.selected_index.saturating_sub(1);
            }

            if i.key_pressed(Key::Enter) && !self.results.is_empty() {
                if let Some(result) = self.results.get(self.selected_index).cloned() {
                    self.open_file(&result, ctx);
                }
            }
        });

        // === RENDU UI PRINCIPAL ===
        let fade = Self::ease_out_cubic(self.animation_progress);
        let scale = 0.95 + 0.05 * fade;

        egui::CentralPanel::default()
            .frame(egui::Frame::none())
            .show(ctx, |ui| {
                // Centrage et animation
                ui.vertical_centered(|ui| {
                    ui.add_space(60.0 * (1.0 - fade * 0.3)); // Slide down effect

                    // Conteneur principal avec ombre et arrondi
                    let window_frame = egui::Frame::none()
                        .fill(bg_main())
                        .rounding(Rounding::same(BORDER_RADIUS))
                        .shadow(Shadow {
                            offset: Vec2::new(0.0, 12.0 * fade),
                            blur: 48.0 * fade,
                            spread: 0.0,
                            color: Color32::from_rgba_unmultiplied(0, 0, 0, (160.0 * fade) as u8),
                        })
                        .inner_margin(Margin::same(0.0));

                    window_frame.show(ui, |ui| {
                        ui.set_width(WINDOW_WIDTH * scale);

                        // Router vers le bon écran selon l'état
                        match self.app_state {
                            AppState::FirstRun => {
                                self.render_first_run_screen(ui, ctx);
                            }
                            AppState::Indexing => {
                                self.render_indexing_screen(ui, ctx);
                            }
                            AppState::Ready => {
                                self.render_search_ui(ui, ctx);
                            }
                        }
                    });
                });
            });

        // Forcer le repaint pour animations fluides
        ctx.request_repaint();
    }
}
