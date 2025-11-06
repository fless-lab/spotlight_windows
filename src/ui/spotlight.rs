use crate::indexer::Indexer;
use crate::search::{SearchEngine, SearchResult};
use crate::tray::SystemTray;
use crate::WindowEvent;
use egui::{
    Align, Color32, FontId, Key, Layout, Margin, Pos2, Rect, Rounding, ScrollArea, Sense, Shadow,
    Stroke, TextEdit, Vec2, Visuals,
};
use std::sync::mpsc::{channel, Receiver, Sender};
use std::sync::Arc;
use std::time::Instant;

#[cfg(target_os = "windows")]
use std::os::windows::process::CommandExt;

// === CONSTANTES DE DESIGN SPOTLIGHT ===
const WINDOW_WIDTH: f32 = 720.0;
const WINDOW_MAX_HEIGHT: f32 = 600.0;
const SEARCH_BAR_HEIGHT: f32 = 64.0;
const RESULT_ITEM_HEIGHT: f32 = 72.0;
const ANIMATION_DURATION: f32 = 0.15; // 150ms comme Spotlight
const BLUR_RADIUS: f32 = 40.0;

// Couleurs Spotlight authentiques (helpers car from_rgba_unmultiplied n'est pas const)
fn bg_color() -> Color32 { Color32::from_rgba_unmultiplied(28, 28, 30, 245) }
fn bg_blur_color() -> Color32 { Color32::from_rgba_unmultiplied(22, 22, 24, 250) }
const TEXT_PRIMARY: Color32 = Color32::WHITE;
const TEXT_SECONDARY: Color32 = Color32::from_gray(152);
const TEXT_TERTIARY: Color32 = Color32::from_gray(99);
fn selection_bg() -> Color32 { Color32::from_rgba_unmultiplied(0, 122, 255, 35) }
fn selection_border() -> Color32 { Color32::from_rgba_unmultiplied(0, 122, 255, 100) }
fn separator_color() -> Color32 { Color32::from_rgba_unmultiplied(255, 255, 255, 8) }
fn shadow_color() -> Color32 { Color32::from_rgba_unmultiplied(0, 0, 0, 120) }

/// Palette Spotlight - Design moderne et épuré
pub struct SpotlightPalette {
    query_sender: Sender<String>,
    result_receiver: Receiver<Vec<SearchResult>>,
    window_event_receiver: Receiver<WindowEvent>,
    window_event_sender: Sender<WindowEvent>,
    system_tray: Option<SystemTray>,
    indexer: Arc<Indexer>,
    query: String,
    results: Vec<SearchResult>,
    selected_index: usize,
    is_searching: bool,
    is_visible: bool,
    last_tooltip_update: Instant,
    // Animation et état visuel
    show_animation_start: Option<Instant>,
    hide_animation_start: Option<Instant>,
    last_frame_time: Instant,
    scroll_offset: f32,
    target_scroll_offset: f32,
}

impl SpotlightPalette {
    pub fn new(
        search_engine: Arc<SearchEngine>,
        window_event_receiver: Receiver<WindowEvent>,
        system_tray: Option<SystemTray>,
        window_event_sender: Sender<WindowEvent>,
        indexer: Arc<Indexer>,
    ) -> Self {
        let (query_tx, query_rx) = channel::<String>();
        let (result_tx, result_rx) = channel::<Vec<SearchResult>>();

        // Thread de recherche
        std::thread::spawn(move || {
            let runtime = tokio::runtime::Runtime::new().unwrap();
            while let Ok(query) = query_rx.recv() {
                let search_engine = search_engine.clone();
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
        Self {
            query_sender: query_tx,
            result_receiver: result_rx,
            window_event_receiver,
            window_event_sender,
            system_tray,
            indexer,
            query: String::new(),
            results: Vec::new(),
            selected_index: 0,
            is_searching: false,
            is_visible: false, // Commence caché (Ctrl+Space pour afficher)
            last_tooltip_update: now,
            show_animation_start: None,
            hide_animation_start: None,
            last_frame_time: now,
            scroll_offset: 0.0,
            target_scroll_offset: 0.0,
        }
    }

    /// Calcule le facteur d'animation fade-in (0.0 = invisible, 1.0 = visible)
    fn get_fade_factor(&self) -> f32 {
        if let Some(start) = self.show_animation_start {
            let elapsed = start.elapsed().as_secs_f32();
            (elapsed / ANIMATION_DURATION).min(1.0)
        } else if let Some(start) = self.hide_animation_start {
            let elapsed = start.elapsed().as_secs_f32();
            1.0 - (elapsed / ANIMATION_DURATION).min(1.0)
        } else if self.is_visible {
            1.0
        } else {
            0.0
        }
    }

    /// Fonction d'easing pour animations fluides (ease-out quad)
    fn ease_out_quad(t: f32) -> f32 {
        1.0 - (1.0 - t) * (1.0 - t)
    }

    /// Fonction d'easing pour animations fluides (ease-in-out cubic)
    fn ease_in_out_cubic(t: f32) -> f32 {
        if t < 0.5 {
            4.0 * t * t * t
        } else {
            1.0 - (-2.0 * t + 2.0).powi(3) / 2.0
        }
    }

    fn perform_search(&mut self) {
        if self.query.is_empty() {
            self.results.clear();
            self.is_searching = false;
            return;
        }
        let _ = self.query_sender.send(self.query.clone());
        self.is_searching = true;
    }

    fn open_selected(&mut self, ctx: &egui::Context) {
        if let Some(result) = self.results.get(self.selected_index) {
            #[cfg(target_os = "windows")]
            {
                let _ = std::process::Command::new("cmd")
                    .args(&["/c", "start", "", &result.path.to_string_lossy()])
                    .creation_flags(0x08000000)
                    .spawn();
            }
            self.is_visible = false;
            ctx.send_viewport_cmd(egui::ViewportCommand::Visible(false));
            self.query.clear();
            self.results.clear();
        }
    }

    fn get_icon_for_result(&self, result: &SearchResult) -> &str {
        if result.is_directory {
            "📁"
        } else {
            match result.extension.as_deref() {
                Some("exe") | Some("msi") => "⚙️",
                Some("txt") | Some("md") => "📄",
                Some("pdf") => "📕",
                Some("docx") | Some("doc") => "📘",
                Some("xlsx") | Some("xls") => "📗",
                Some("pptx") | Some("ppt") => "📙",
                Some("jpg") | Some("jpeg") | Some("png") | Some("gif") | Some("bmp") => "🖼️",
                Some("mp3") | Some("wav") | Some("flac") | Some("m4a") => "🎵",
                Some("mp4") | Some("avi") | Some("mkv") | Some("mov") => "🎬",
                Some("zip") | Some("rar") | Some("7z") | Some("tar") | Some("gz") => "📦",
                Some("rs") | Some("py") | Some("js") | Some("ts") | Some("java") => "💻",
                Some("html") | Some("css") | Some("json") | Some("xml") => "🌐",
                _ => "📄",
            }
        }
    }
}

impl eframe::App for SpotlightPalette {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        // Mettre à jour le tooltip du tray toutes les 2 secondes
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
            self.last_tooltip_update = std::time::Instant::now();
        }

        // Vérifier les événements du system tray
        if let Some(ref tray) = self.system_tray {
            if let Some(tray_event) = tray.try_recv_event() {
                let event = match tray_event {
                    crate::tray::TrayEvent::Show => WindowEvent::Show,
                    crate::tray::TrayEvent::Quit => WindowEvent::Quit,
                };
                let _ = self.window_event_sender.send(event);
            }
        }

        // Vérifier les événements de fenêtre (Ctrl+Space, tray icon)
        while let Ok(event) = self.window_event_receiver.try_recv() {
            match event {
                WindowEvent::Show => {
                    self.is_visible = true;
                    ctx.send_viewport_cmd(egui::ViewportCommand::Visible(true));
                    ctx.send_viewport_cmd(egui::ViewportCommand::Focus);
                }
                WindowEvent::Hide => {
                    self.is_visible = false;
                    ctx.send_viewport_cmd(egui::ViewportCommand::Visible(false));
                }
                WindowEvent::Toggle => {
                    self.is_visible = !self.is_visible;
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

        // Recevoir les résultats
        if let Ok(results) = self.result_receiver.try_recv() {
            self.results = results;
            self.is_searching = false;
            self.selected_index = 0;
        }

        // Style Spotlight
        let mut style = (*ctx.style()).clone();
        style.visuals = if ctx.style().visuals.dark_mode {
            spotlight_dark_visuals()
        } else {
            spotlight_light_visuals()
        };
        ctx.set_style(style);

        // Ne dessiner l'UI que si visible
        if !self.is_visible {
            // Ne rien dessiner - la fenêtre reste cachée proprement
            return;
        }

        // Fenêtre centrée style Spotlight
        egui::CentralPanel::default()
            .frame(
                egui::Frame::none()
                    .fill(Color32::from_rgba_unmultiplied(28, 28, 30, 250)) // Fond semi-transparent
                    .rounding(Rounding::same(12.0))
                    .shadow(Shadow {
                        offset: Vec2::new(0.0, 8.0),
                        blur: 24.0,
                        spread: 0.0,
                        color: Color32::from_black_alpha(80),
                    })
                    .inner_margin(Margin::same(0.0)),
            )
            .show(ctx, |ui| {
                ui.set_max_width(720.0);
                ui.set_min_width(720.0);

                // === TITLEBAR DRAGGABLE + BOUTON FERMER ===
                egui::Frame::none()
                    .inner_margin(Margin {
                        left: 16.0,
                        right: 8.0,
                        top: 8.0,
                        bottom: 4.0,
                    })
                    .show(ui, |ui| {
                        ui.horizontal(|ui| {
                            // Zone draggable
                            let titlebar_rect = ui.allocate_space(Vec2::new(640.0, 20.0)).1;
                            if ui.interact(titlebar_rect, ui.id().with("titlebar"), Sense::drag()).dragged() {
                                // Permet de déplacer la fenêtre
                                if ctx.input(|i| i.pointer.interact_pos()).is_some() {
                                    ctx.send_viewport_cmd(egui::ViewportCommand::StartDrag);
                                }
                            }

                            // Bouton X (fermer)
                            let close_button = ui.add(
                                egui::Button::new(
                                    egui::RichText::new("✕")
                                        .size(16.0)
                                        .color(Color32::from_gray(200))
                                )
                                .fill(Color32::TRANSPARENT)
                                .stroke(Stroke::NONE)
                                .rounding(Rounding::same(4.0))
                            );

                            if close_button.clicked() {
                                std::process::exit(0); // Fermer l'app
                            }

                            if close_button.hovered() {
                                ui.ctx().set_cursor_icon(egui::CursorIcon::PointingHand);
                            }
                        });
                    });

                // === BARRE DE RECHERCHE ===
                egui::Frame::none()
                    .inner_margin(Margin {
                        left: 20.0,
                        right: 20.0,
                        top: 16.0,
                        bottom: 16.0,
                    })
                    .show(ui, |ui| {
                        ui.horizontal(|ui| {
                            // Icône loupe
                            ui.label(
                                egui::RichText::new("🔍")
                                    .size(24.0)
                                    .color(Color32::from_gray(140)),
                            );

                            ui.add_space(12.0);

                            // Champ de recherche
                            let text_edit = TextEdit::singleline(&mut self.query)
                                .font(FontId::proportional(20.0))
                                .text_color(Color32::from_gray(240))
                                .hint_text(
                                    egui::RichText::new("Rechercher des fichiers...")
                                        .size(20.0)
                                        .color(Color32::from_gray(100)),
                                )
                                .desired_width(f32::INFINITY)
                                .frame(false);

                            let response = ui.add(text_edit);
                            response.request_focus();

                            if response.changed() {
                                self.perform_search();
                            }
                        });
                    });

                // === DIVIDER ===
                ui.add_space(0.0);
                ui.separator();

                // === RÉSULTATS ===
                let mut clicked_index: Option<usize> = None;

                if !self.results.is_empty() {
                    ScrollArea::vertical()
                        .max_height(400.0)
                        .auto_shrink([false, true])
                        .show(ui, |ui| {
                            for (index, result) in self.results.iter().enumerate() {
                                let is_selected = index == self.selected_index;

                                // Frame pour chaque résultat
                                let frame = if is_selected {
                                    egui::Frame::none()
                                        .fill(Color32::from_rgba_unmultiplied(0, 122, 255, 25))
                                        .rounding(Rounding::same(8.0))
                                        .inner_margin(Margin::symmetric(16.0, 12.0))
                                } else {
                                    egui::Frame::none()
                                        .fill(Color32::TRANSPARENT)
                                        .inner_margin(Margin::symmetric(16.0, 12.0))
                                };

                                let response = frame
                                    .show(ui, |ui| {
                                        ui.horizontal(|ui| {
                                            // Icône
                                            ui.label(
                                                egui::RichText::new(self.get_icon_for_result(result))
                                                    .size(32.0),
                                            );

                                            ui.add_space(12.0);

                                            // Texte
                                            ui.vertical(|ui| {
                                                // Nom
                                                ui.label(
                                                    egui::RichText::new(&result.name)
                                                        .size(15.0)
                                                        .color(Color32::from_gray(240))
                                                        .strong(),
                                                );

                                                // Chemin
                                                let path_text = result.path.to_string_lossy();
                                                let display_path = if path_text.len() > 60 {
                                                    format!("...{}", &path_text[path_text.len() - 57..])
                                                } else {
                                                    path_text.to_string()
                                                };

                                                ui.label(
                                                    egui::RichText::new(display_path)
                                                        .size(13.0)
                                                        .color(Color32::from_gray(140)),
                                                );
                                            });
                                        });
                                    })
                                    .response;

                                // Interaction
                                if response.hovered() {
                                    self.selected_index = index;
                                }

                                if response.clicked() {
                                    clicked_index = Some(index);
                                }
                            }
                        });
                }

                // Traiter le clic après la boucle
                if let Some(idx) = clicked_index {
                    self.selected_index = idx;
                    self.open_selected(ctx);
                }

                // Afficher message si aucun résultat
                if self.results.is_empty() && !self.query.is_empty() && !self.is_searching {
                    // Aucun résultat
                    ui.add_space(40.0);
                    ui.vertical_centered(|ui| {
                        ui.label(
                            egui::RichText::new("Aucun résultat")
                                .size(16.0)
                                .color(Color32::from_gray(120)),
                        );
                    });
                    ui.add_space(40.0);
                }

                // === FOOTER (optionnel) ===
                if !self.results.is_empty() {
                    ui.separator();
                    egui::Frame::none()
                        .inner_margin(Margin::symmetric(20.0, 10.0))
                        .show(ui, |ui| {
                            ui.horizontal(|ui| {
                                ui.label(
                                    egui::RichText::new("↑↓ Naviguer")
                                        .size(11.0)
                                        .color(Color32::from_gray(100)),
                                );
                                ui.label(
                                    egui::RichText::new("•")
                                        .size(11.0)
                                        .color(Color32::from_gray(60)),
                                );
                                ui.label(
                                    egui::RichText::new("Enter Ouvrir")
                                        .size(11.0)
                                        .color(Color32::from_gray(100)),
                                );
                                ui.label(
                                    egui::RichText::new("•")
                                        .size(11.0)
                                        .color(Color32::from_gray(60)),
                                );
                                ui.label(
                                    egui::RichText::new("Esc Fermer")
                                        .size(11.0)
                                        .color(Color32::from_gray(100)),
                                );
                            });
                        });
                }
            });

        // === GESTION CLAVIER ===
        ctx.input(|i| {
            if i.key_pressed(Key::Escape) {
                self.is_visible = false;
                ctx.send_viewport_cmd(egui::ViewportCommand::Visible(false));
                self.query.clear();
                self.results.clear();
            }

            if i.key_pressed(Key::ArrowDown) {
                if self.selected_index < self.results.len().saturating_sub(1) {
                    self.selected_index += 1;
                }
            }

            if i.key_pressed(Key::ArrowUp) {
                self.selected_index = self.selected_index.saturating_sub(1);
            }

            if i.key_pressed(Key::Enter) {
                self.open_selected(ctx);
            }
        });

        // Forcer le repaint pour les animations
        ctx.request_repaint();
    }
}

/// Visuals Spotlight Dark Mode
fn spotlight_dark_visuals() -> Visuals {
    Visuals {
        dark_mode: true,
        override_text_color: Some(Color32::from_gray(240)),
        window_fill: Color32::from_rgba_unmultiplied(28, 28, 30, 250),
        panel_fill: Color32::from_rgba_unmultiplied(28, 28, 30, 250),
        faint_bg_color: Color32::from_rgba_unmultiplied(255, 255, 255, 8),
        extreme_bg_color: Color32::from_rgba_unmultiplied(10, 10, 10, 250),
        code_bg_color: Color32::from_rgb(35, 35, 38),
        selection: egui::style::Selection {
            bg_fill: Color32::from_rgba_unmultiplied(0, 122, 255, 40),
            stroke: Stroke::new(1.0, Color32::from_rgba_unmultiplied(0, 122, 255, 100)),
        },
        ..Visuals::dark()
    }
}

/// Visuals Spotlight Light Mode
fn spotlight_light_visuals() -> Visuals {
    Visuals {
        dark_mode: false,
        override_text_color: Some(Color32::from_gray(20)),
        window_fill: Color32::from_rgba_unmultiplied(248, 248, 250, 245),
        panel_fill: Color32::from_rgba_unmultiplied(248, 248, 250, 245),
        faint_bg_color: Color32::from_rgba_unmultiplied(0, 0, 0, 8),
        extreme_bg_color: Color32::from_rgba_unmultiplied(255, 255, 255, 250),
        code_bg_color: Color32::from_rgb(240, 240, 242),
        selection: egui::style::Selection {
            bg_fill: Color32::from_rgba_unmultiplied(0, 122, 255, 30),
            stroke: Stroke::new(1.0, Color32::from_rgba_unmultiplied(0, 122, 255, 80)),
        },
        ..Visuals::light()
    }
}
