use tray_icon::{
    menu::{Menu, MenuEvent, MenuItem},
    TrayIcon, TrayIconBuilder,
};
use std::sync::mpsc::{channel, Receiver, Sender};

pub enum TrayEvent {
    Show,
    Quit,
}

pub struct SystemTray {
    _tray_icon: TrayIcon,
    event_receiver: Receiver<TrayEvent>,
}

impl SystemTray {
    pub fn new() -> anyhow::Result<Self> {
        // Créer le menu
        let tray_menu = Menu::new();

        let show_item = MenuItem::new("Ouvrir Spotlight (Ctrl+Space)", true, None);
        let quit_item = MenuItem::new("Quitter", true, None);

        tray_menu.append(&show_item)?;
        tray_menu.append(&quit_item)?;

        // Icon simple programmatique (icône loupe 32x32)
        let image = create_search_icon();

        // Créer le tray icon
        let tray_icon = TrayIconBuilder::new()
            .with_menu(Box::new(tray_menu))
            .with_tooltip("Spotlight Windows - Prêt")
            .with_icon(image)
            .build()?;

        // Channel pour les events
        let (tx, rx) = channel();

        // Listen aux events du menu
        let tx_clone = tx.clone();
        let show_id = show_item.id().clone();
        let quit_id = quit_item.id().clone();

        std::thread::spawn(move || {
            let menu_channel = MenuEvent::receiver();
            loop {
                if let Ok(event) = menu_channel.recv() {
                    if event.id == show_id {
                        let _ = tx_clone.send(TrayEvent::Show);
                    } else if event.id == quit_id {
                        let _ = tx_clone.send(TrayEvent::Quit);
                    }
                }
            }
        });

        Ok(Self {
            _tray_icon: tray_icon,
            event_receiver: rx,
        })
    }

    pub fn try_recv_event(&self) -> Option<TrayEvent> {
        self.event_receiver.try_recv().ok()
    }

    pub fn set_tooltip(&self, text: &str) {
        let _ = self._tray_icon.set_tooltip(Some(text));
    }
}

/// Créer une icône loupe simple 32x32
fn create_search_icon() -> tray_icon::Icon {
    let width = 32;
    let height = 32;
    let mut rgba = vec![0u8; (width * height * 4) as usize];

    // Dessiner un cercle blanc (loupe) sur fond transparent
    for y in 0..height {
        for x in 0..width {
            let dx = x as i32 - 16;
            let dy = y as i32 - 16;
            let dist = ((dx * dx + dy * dy) as f32).sqrt();

            let idx = ((y * width + x) * 4) as usize;

            // Cercle blanc entre radius 8 et 10
            if dist >= 8.0 && dist <= 10.0 {
                rgba[idx] = 255; // R
                rgba[idx + 1] = 255; // G
                rgba[idx + 2] = 255; // B
                rgba[idx + 3] = 255; // A
            }
            // Handle (ligne) de la loupe
            else if x >= 20 && x <= 22 && y >= 20 && y <= 28 {
                rgba[idx] = 255;
                rgba[idx + 1] = 255;
                rgba[idx + 2] = 255;
                rgba[idx + 3] = 255;
            }
        }
    }

    tray_icon::Icon::from_rgba(rgba, width, height)
        .expect("Failed to create icon")
}
