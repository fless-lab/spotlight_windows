use super::theme::setup_custom_theme;
use crate::search::{SearchEngine, SearchResult};
use egui::{
    Color32, FontId, Key, ScrollArea, Sense, TextEdit, TextStyle, Vec2,
};
use std::sync::Arc;
use tokio::runtime::Runtime;

/// Application Spotlight principale
pub struct SpotlightApp {
    search_engine: Arc<SearchEngine>,
    runtime: Runtime,
    query: String,
    results: Vec<SearchResult>,
    selected_index: usize,
    is_searching: bool,
    show_window: bool,
}

impl SpotlightApp {
    pub fn new(search_engine: Arc<SearchEngine>) -> Self {
        let runtime = Runtime::new().expect("Failed to create Tokio runtime");

        Self {
            search_engine,
            runtime,
            query: String::new(),
            results: Vec::new(),
            selected_index: 0,
            is_searching: false,
            show_window: true,
        }
    }

    /// Affiche ou cache la fenêtre
    pub fn toggle_window(&mut self) {
        self.show_window = !self.show_window;

        if !self.show_window {
            self.clear_search();
        }
    }

    /// Efface la recherche
    fn clear_search(&mut self) {
        self.query.clear();
        self.results.clear();
        self.selected_index = 0;
    }

    /// Effectue une recherche
    fn perform_search(&mut self) {
        if self.query.is_empty() {
            self.results.clear();
            return;
        }

        let query = self.query.clone();
        let search_engine = self.search_engine.clone();

        self.is_searching = true;

        // Recherche asynchrone
        let results = self.runtime.block_on(async move {
            search_engine
                .search(&query, 50)
                .await
                .unwrap_or_else(|_| vec![])
        });

        self.results = results;
        self.is_searching = false;
        self.selected_index = 0;
    }

    /// Ouvre le fichier/dossier sélectionné
    fn open_selected(&mut self) {
        if let Some(_result) = self.results.get(self.selected_index) {
            #[cfg(target_os = "windows")]
            {
                let path = &self.results[self.selected_index].path;
                let _ = std::process::Command::new("explorer")
                    .arg(path.to_string_lossy().to_string())
                    .spawn();
            }

            self.toggle_window();
        }
    }

    /// Ouvre l'emplacement du fichier
    fn open_location(&mut self) {
        if let Some(_result) = self.results.get(self.selected_index) {
            #[cfg(target_os = "windows")]
            {
                let path = &self.results[self.selected_index].path;
                if let Some(parent) = path.parent() {
                    let _ = std::process::Command::new("explorer")
                        .arg(parent.to_string_lossy().to_string())
                        .spawn();
                }
            }

            self.toggle_window();
        }
    }
}

impl eframe::App for SpotlightApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        // Setup du thème une seule fois
        if ctx.style().text_styles.get(&TextStyle::Heading).is_none()
            || ctx.style().text_styles[&TextStyle::Heading].size != 24.0
        {
            setup_custom_theme(ctx);
        }

        // Si la fenêtre est cachée, ne rien afficher
        if !self.show_window {
            return;
        }

        egui::CentralPanel::default().show(ctx, |ui| {
            ui.vertical_centered(|ui| {
                ui.add_space(20.0);

                // Logo/Titre
                ui.heading("🔍 Spotlight Windows");
                ui.add_space(10.0);

                // Barre de recherche
                let search_response = ui.add_sized(
                    Vec2::new(ui.available_width() - 40.0, 40.0),
                    TextEdit::singleline(&mut self.query)
                        .hint_text("Rechercher des fichiers...")
                        .font(FontId::proportional(18.0))
                        .desired_width(f32::INFINITY),
                );

                // Auto-focus sur la barre de recherche
                if self.show_window {
                    search_response.request_focus();
                }

                // Détecter les changements dans la query
                if search_response.changed() {
                    self.perform_search();
                }

                // Gestion des touches
                if ctx.input(|i| i.key_pressed(Key::Escape)) {
                    self.toggle_window();
                }

                if ctx.input(|i| i.key_pressed(Key::ArrowDown)) {
                    if self.selected_index < self.results.len().saturating_sub(1) {
                        self.selected_index += 1;
                    }
                }

                if ctx.input(|i| i.key_pressed(Key::ArrowUp)) {
                    self.selected_index = self.selected_index.saturating_sub(1);
                }

                if ctx.input(|i| i.key_pressed(Key::Enter)) {
                    self.open_selected();
                }

                if ctx.input(|i| i.modifiers.ctrl && i.key_pressed(Key::L)) {
                    self.open_location();
                }

                ui.add_space(15.0);

                // Indicateur de recherche
                if self.is_searching {
                    ui.spinner();
                } else if !self.results.is_empty() {
                    ui.label(format!("{} résultat(s)", self.results.len()));
                }

                ui.add_space(10.0);

                // Résultats
                let results_clone = self.results.clone(); // Cloner pour éviter le borrow
                ScrollArea::vertical()
                    .max_height(400.0)
                    .show(ui, |ui| {
                        for (index, result) in results_clone.iter().enumerate() {
                            let is_selected = index == self.selected_index;

                            let (rect, response) = ui.allocate_exact_size(
                                Vec2::new(ui.available_width(), 60.0),
                                Sense::click(),
                            );

                            // Couleur de fond
                            let bg_color = if is_selected {
                                Color32::from_rgb(0, 122, 255)
                            } else if response.hovered() {
                                Color32::from_rgb(55, 55, 60)
                            } else {
                                Color32::from_rgb(40, 40, 45)
                            };

                            ui.painter().rect_filled(
                                rect,
                                egui::Rounding::same(8.0),
                                bg_color,
                            );

                            // Contenu - utiliser un scope limité
                            let _ = ui.allocate_new_ui(egui::UiBuilder::new().max_rect(rect), |ui| {
                                ui.add_space(10.0);

                                ui.vertical(|ui| {
                                    ui.add_space(8.0);

                                    // Icône + Nom
                                    ui.horizontal(|ui| {
                                        let icon = if result.is_directory {
                                            "📁"
                                        } else {
                                            match result.extension.as_deref() {
                                                Some("exe") => "⚙️",
                                                Some("txt") => "📄",
                                                Some("pdf") => "📕",
                                                Some("jpg") | Some("png") | Some("gif") => "🖼️",
                                                Some("mp3") | Some("wav") => "🎵",
                                                Some("mp4") | Some("avi") => "🎬",
                                                Some("zip") | Some("rar") => "📦",
                                                _ => "📄",
                                            }
                                        };

                                        ui.label(
                                            egui::RichText::new(icon).size(20.0),
                                        );

                                        ui.label(
                                            egui::RichText::new(&result.name)
                                                .size(16.0)
                                                .strong(),
                                        );
                                    });

                                    // Chemin
                                    ui.label(
                                        egui::RichText::new(result.path.to_string_lossy())
                                            .size(12.0)
                                            .color(Color32::from_rgb(150, 150, 150)),
                                    );

                                    // Métadonnées
                                    ui.horizontal(|ui| {
                                        ui.label(
                                            egui::RichText::new(&result.formatted_size())
                                                .size(11.0)
                                                .color(Color32::from_rgb(120, 120, 120)),
                                        );
                                        ui.label(
                                            egui::RichText::new("•")
                                                .size(11.0)
                                                .color(Color32::from_rgb(120, 120, 120)),
                                        );
                                        ui.label(
                                            egui::RichText::new(&result.formatted_modified())
                                                .size(11.0)
                                                .color(Color32::from_rgb(120, 120, 120)),
                                        );
                                    });
                                });
                            });

                            // Gestion du clic
                            if response.clicked() {
                                self.selected_index = index;
                                self.open_selected();
                            }

                            ui.add_space(5.0);
                        }
                    });

                ui.add_space(10.0);

                // Aide
                ui.separator();
                ui.horizontal(|ui| {
                    ui.label(
                        egui::RichText::new("Enter: Ouvrir")
                            .size(11.0)
                            .color(Color32::from_rgb(120, 120, 120)),
                    );
                    ui.label(
                        egui::RichText::new("•")
                            .size(11.0)
                            .color(Color32::from_rgb(120, 120, 120)),
                    );
                    ui.label(
                        egui::RichText::new("Ctrl+L: Ouvrir l'emplacement")
                            .size(11.0)
                            .color(Color32::from_rgb(120, 120, 120)),
                    );
                    ui.label(
                        egui::RichText::new("•")
                            .size(11.0)
                            .color(Color32::from_rgb(120, 120, 120)),
                    );
                    ui.label(
                        egui::RichText::new("Esc: Fermer")
                            .size(11.0)
                            .color(Color32::from_rgb(120, 120, 120)),
                    );
                });
            });
        });
    }
}
