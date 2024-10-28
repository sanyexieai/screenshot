use std::collections::HashMap;
use std::ptr;
use winapi::shared::windef::HWND;
use winapi::um::winuser::{GetMessageW, MSG, WM_HOTKEY, RegisterHotKey, UnregisterHotKey};
use std::sync::{Mutex};
use crossbeam::channel::{bounded, Sender};

pub struct HotkeyManager {
    exit_sender: Sender<()>,
}

impl HotkeyManager {
    pub fn register_hotkey<F>( id: u32, modifiers: u32, vk: u32, callback: F)
    where
        F: Fn() + Send + Sync + 'static,
    {
        let hwnd: HWND = std::ptr::null_mut();
        let result: i32 = unsafe { RegisterHotKey(hwnd, id as i32, modifiers, vk) };
        if result == 0 {
            println!("Failed to register hotkey.");
            return;
        }
        loop {
            let mut msg = unsafe { std::mem::zeroed() };
            let result = unsafe { winapi::um::winuser::GetMessageW(&mut msg, hwnd, 0, 0) };
            if result == -1 {
                println!("Failed to get message.");
                break;
            }
            if msg.message == WM_HOTKEY {
                let hotkey = (
                    (msg.lParam as u32) & 0xFFFF,
                    ((msg.lParam as u32) >> 16) & 0xFFFF,
                );
                println!("Hotkey received: {:?}", hotkey); // 输出热键信息
                callback();
            }
        }
    }

    pub fn stop_listening(&self) {
        let _ = self.exit_sender.send(());
    }
}