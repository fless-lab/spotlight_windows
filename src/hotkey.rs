use anyhow::Result;
use std::sync::mpsc;
use std::thread;
use tracing::{error, info};
use windows::Win32::Foundation::HWND;
use windows::Win32::UI::Input::KeyboardAndMouse::HOT_KEY_MODIFIERS;
use windows::Win32::UI::WindowsAndMessaging::{
    DispatchMessageW, GetMessageW, RegisterHotKey, TranslateMessage, UnregisterHotKey, MSG,
    WM_HOTKEY,
};

/// Gestionnaire de hotkey global Windows
pub struct HotkeyManager {
    hotkey_id: i32,
}

impl HotkeyManager {
    pub fn new() -> Self {
        Self { hotkey_id: 1 }
    }

    /// Enregistre un hotkey global (Alt+Space par défaut)
    pub fn register_hotkey(&self) -> Result<mpsc::Receiver<()>> {
        let (tx, rx) = mpsc::channel();

        thread::spawn(move || {
            unsafe {
                // Enregistrer le hotkey Alt+Space
                // VK_SPACE = 0x20
                let hwnd = HWND(std::ptr::null_mut());
                let result = RegisterHotKey(
                    hwnd,
                    1,
                    HOT_KEY_MODIFIERS(0x0001), // MOD_ALT = 0x0001
                    0x20, // VK_SPACE
                );

                if result.is_err() {
                    error!("Impossible d'enregistrer le hotkey global");
                    return;
                }

                info!("Hotkey global enregistré: Alt+Space");

                // Boucle de messages Windows
                let mut msg = MSG::default();

                loop {
                    let result = GetMessageW(&mut msg, hwnd, 0, 0);

                    if result.is_err() || result.unwrap().0 == 0 {
                        break;
                    }

                    if msg.message == WM_HOTKEY {
                        // Hotkey déclenché
                        if tx.send(()).is_err() {
                            break;
                        }
                    }

                    let _ = TranslateMessage(&msg);
                    DispatchMessageW(&msg);
                }

                // Cleanup
                let _ = UnregisterHotKey(hwnd, 1);
            }
        });

        Ok(rx)
    }
}

impl Drop for HotkeyManager {
    fn drop(&mut self) {
        unsafe {
            let hwnd = HWND(std::ptr::null_mut());
            let _ = UnregisterHotKey(hwnd, self.hotkey_id);
        }
    }
}
