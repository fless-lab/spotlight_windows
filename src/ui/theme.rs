use egui::{Color32, FontFamily, FontId, Rounding, Stroke, TextStyle, Visuals};

/// Thème visuel moderne et élégant
pub fn setup_custom_theme(ctx: &egui::Context) {
    // Configuration des polices
    let mut style = (*ctx.style()).clone();

    style.text_styles.insert(
        TextStyle::Heading,
        FontId::new(24.0, FontFamily::Proportional),
    );
    style.text_styles.insert(
        TextStyle::Body,
        FontId::new(16.0, FontFamily::Proportional),
    );
    style.text_styles.insert(
        TextStyle::Button,
        FontId::new(16.0, FontFamily::Proportional),
    );
    style.text_styles.insert(
        TextStyle::Small,
        FontId::new(12.0, FontFamily::Proportional),
    );

    // Espacement
    style.spacing.item_spacing = egui::vec2(8.0, 8.0);
    style.spacing.window_margin = egui::Margin::same(12.0);
    style.spacing.button_padding = egui::vec2(10.0, 6.0);

    // Arrondi des coins
    style.visuals.window_rounding = Rounding::same(12.0);
    style.visuals.menu_rounding = Rounding::same(8.0);
    style.visuals.window_shadow.offset = egui::vec2(0.0, 4.0);
    style.visuals.window_shadow.blur = 20.0;

    ctx.set_style(style);

    // Thème sombre moderne
    let mut visuals = Visuals::dark();

    // Couleurs personnalisées
    visuals.window_fill = Color32::from_rgb(25, 25, 30);
    visuals.panel_fill = Color32::from_rgb(30, 30, 35);
    visuals.extreme_bg_color = Color32::from_rgb(15, 15, 20);

    // Couleur de sélection (bleu moderne)
    visuals.selection.bg_fill = Color32::from_rgb(0, 122, 255);
    visuals.selection.stroke = Stroke::new(1.0, Color32::from_rgb(0, 122, 255));

    // Widgets
    visuals.widgets.noninteractive.bg_fill = Color32::from_rgb(40, 40, 45);
    visuals.widgets.inactive.bg_fill = Color32::from_rgb(45, 45, 50);
    visuals.widgets.hovered.bg_fill = Color32::from_rgb(55, 55, 60);
    visuals.widgets.active.bg_fill = Color32::from_rgb(0, 122, 255);

    // Texte
    visuals.widgets.noninteractive.fg_stroke = Stroke::new(1.0, Color32::from_rgb(200, 200, 200));
    visuals.widgets.inactive.fg_stroke = Stroke::new(1.0, Color32::from_rgb(220, 220, 220));
    visuals.widgets.hovered.fg_stroke = Stroke::new(1.0, Color32::WHITE);
    visuals.widgets.active.fg_stroke = Stroke::new(1.0, Color32::WHITE);

    ctx.set_visuals(visuals);
}
