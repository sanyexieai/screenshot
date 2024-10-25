use std::collections::HashMap;
use std::ptr;
use std::thread;
use std::time::Duration;
use winapi::um::winuser::{GetMessageW, MSG, WM_HOTKEY, RegisterHotKey, UnregisterHotKey};
use std::{process};
use std::sync::{Arc, Mutex};
use crossbeam::channel::{bounded, Sender};

type Hotkey = (u32, u32);
type Callback = Box<dyn Fn() + Send + Sync>;

pub struct HotkeyManager {
    callbacks: Arc<Mutex<HashMap<Hotkey, Callback>>>,
    exit_sender: Sender<()>,
}

impl HotkeyManager {
    pub fn new() -> Self {
        let (exit_sender, exit_receiver) = bounded::<()>(1);
        let callbacks: Arc<Mutex<HashMap<Hotkey, Callback>>> = Arc::new(Mutex::new(HashMap::new()));
        let cb_clone = Arc::clone(&callbacks);

        // 启动消息监听线程
        thread::spawn(move || {
            let mut msg: MSG = unsafe { std::mem::zeroed() };

            while unsafe { GetMessageW(&mut msg, ptr::null_mut(), 0, 0) } > 0 {
                // 检查退出信号
                if exit_receiver.try_recv().is_ok() {
                    break;
                }

                if msg.message == WM_HOTKEY {
                    let hotkey = (
                        (msg.lParam as u32) & 0xFFFF,
                        ((msg.lParam as u32) >> 16) & 0xFFFF,
                    );

                    println!("Hotkey received: {:?}", hotkey); // 输出热键信息
                    if let Some(callback) = cb_clone.lock().unwrap().get(&hotkey) {
                        callback();
                    } else {
                        println!("No callback found for: {:?}", hotkey); // 没有找到回调
                    }
                }
            }

            unsafe {
                for id in 1..=cb_clone.lock().unwrap().len() as i32 {
                    UnregisterHotKey(ptr::null_mut(), id);
                }
            }
            process::exit(0);
        });

        Self { callbacks, exit_sender }
    }

    pub fn register_hotkey<F>(&self, id: u32, modifiers: u32, vk: u32, callback: F)
    where
        F: Fn() + Send + Sync + 'static,
    {
        let hotkey = (modifiers, vk);
        unsafe {
            if RegisterHotKey(ptr::null_mut(), id as i32, modifiers, vk) == 0 {
                eprintln!("Failed to register hotkey: {:?} + {:?}", modifiers, vk);
                return;
            } else {
                println!("Successfully registered hotkey: {:?} + {:?}", modifiers, vk);
            }
        }
        self.callbacks.lock().unwrap().insert(hotkey, Box::new(callback));
    }

    pub fn stop_listening(&self) {
        let _ = self.exit_sender.send(());
    }
}