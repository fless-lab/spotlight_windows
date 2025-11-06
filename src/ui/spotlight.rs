use crate::search::{SearchEngine, SearchResult};
use egui::{
    Align, Color32, FontId, Key, Layout, Margin, Rounding, ScrollArea, Sense, Shadow, Stroke,
    TextEdit, Vec2, Visuals,
};
use std::sync::mpsc::{channel, Receiver, Sender};
use std::sync::Arc;

#[cfg(target_os = "windows")]
use std::os::windows::process::CommandExt;

/// Palette Spotlight - Design moderne et épuré
pub struct SpotlightPalette {
    query_sender: Sender<String>,
    result_receiver: Receiver<Vec<SearchResult>>,
    query: String,
    results: Vec<SearchResult>,
    selected_index: usize,
    is_searching: bool,
    is_visible: bool,
}

impl SpotlightPalette {
    pub fn new(search_engine: Arc<SearchEngine>) -> Self {
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

        Self {
            query_sender: query_tx,
            result_receiver: result_rx,
            query: String::new(),
            results: Vec::new(),
            selected_index: 0,
            is_searching: false,
            is_visible: true, // Commence visible pour test
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

    fn open_selected(&mut self) {
        if let Some(result) = self.results.get(self.selected_index) {
            #[cfg(target_os = "windows")]
            {
                let _ = std::process::Command::new("cmd")
                    .args(&["/c", "start", "", &result.path.to_string_lossy()])
                    .creation_flags(0x08000000)
                    .spawn();
            }
            self.is_visible = false;
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
    fn update(&mut self, ctx: &egui::Context, frame: &mut eframe::Frame) {
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

        // Cacher si non visible
        if !self.is_visible {
            // Masquer la fenêtre mais garder le process actif
            // TODO: Vraiment cacher avec hide() quand on aura le hotkey
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
                    self.open_selected();
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
                self.open_selected();
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
