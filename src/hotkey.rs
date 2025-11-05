use anyhow::Result;
use std::sync::mpsc;
use std::thread;
use tracing::{error, info};
use windows::Win32::Foundation::{HWND, LPARAM, LRESULT, WPARAM};
use windows::Win32::UI::Input::KeyboardAndMouse::{MOD_ALT, MOD_CONTROL, MOD_SHIFT, MOD_WIN};
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
                let result = RegisterHotKey(HWND(0), 1, MOD_ALT, 0x20);

                if result.is_err() {
                    error!("Impossible d'enregistrer le hotkey global");
                    return;
                }

                info!("Hotkey global enregistré: Alt+Space");

                // Boucle de messages Windows
                let mut msg = MSG::default();

                loop {
                    let result = GetMessageW(&mut msg, HWND(0), 0, 0);

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
                let _ = UnregisterHotKey(HWND(0), 1);
            }
        });

        Ok(rx)
    }
}

impl Drop for HotkeyManager {
    fn drop(&mut self) {
        unsafe {
            let _ = UnregisterHotKey(HWND(0), self.hotkey_id);
        }
    }
}
