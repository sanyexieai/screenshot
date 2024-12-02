use screenshots::Screen;
use slint::LogicalPosition;
use std::error::Error;
use chrono::Local;
use anyhow::Result;
use global_hotkey::{GlobalHotKeyManager, hotkey::{HotKey, Modifiers, Code}, GlobalHotKeyEvent};
use std::sync::mpsc::channel;
use std::rc::Rc;
use std::cell::RefCell;
use slint::Window;
use raw_window_handle::{HasRawWindowHandle, RawWindowHandle};

// 导入UI组件
slint::include_modules!();

#[cfg(target_os = "windows")]
use windows_sys::Win32::UI::WindowsAndMessaging::{
    SetWindowPos, HWND_TOPMOST, SWP_NOMOVE, SWP_NOSIZE
};

// 定义窗口组件并导入PreviewWindow
slint::slint! {
    import { PreviewWindow } from "ui/preview_window.slint";

    export component BackgroundWindow inherits Window {
        background: transparent;
        no-frame: true;
        width: 1px;
        height: 1px;
        visible: false;
    }

    export { PreviewWindow }
}

// 添加预览窗口状态结构体
struct PreviewWindowState {
    window: Rc<PreviewWindow>,
}

impl PreviewWindowState {
    fn new(image_data: Vec<u8>, width: u32, height: u32) -> Rc<RefCell<Self>> {
        let window = Rc::new(PreviewWindow::new().unwrap());
        
        // 创建slint图像
        let mut pixel_buffer = slint::SharedPixelBuffer::<slint::Rgba8Pixel>::new(width, height);
        let buffer = pixel_buffer.make_mut_bytes();
        
        // 复制并转换图像数据 - 假设输入是RGBA格式
        for i in 0..width as usize * height as usize {
            let src_idx = i * 4;
            let dst_idx = i * 4;
            if src_idx + 3 < image_data.len() && dst_idx + 3 < buffer.len() {
                buffer[dst_idx] = image_data[src_idx + 2];     // B
                buffer[dst_idx + 1] = image_data[src_idx + 1]; // G
                buffer[dst_idx + 2] = image_data[src_idx];     // R
                buffer[dst_idx + 3] = image_data[src_idx + 3]; // A
            }
        }
        
        let slint_image = slint::Image::from_rgba8(pixel_buffer);
        window.set_screenshot(slint_image);
        
        let instance = Rc::new(RefCell::new(Self {
            window: window.clone(),
        }));
        
        let window_weak = window.as_weak();
        window.on_pin_to_desktop(move || {
            // 设置窗口始终置顶
            #[cfg(target_os = "windows")]
            if let Some(window) = window_weak.upgrade() {
                use windows_sys::Win32::UI::WindowsAndMessaging::{
                    SetWindowPos, HWND_TOPMOST, SWP_NOMOVE, SWP_NOSIZE
                };
                use windows_sys::Win32::Foundation::HWND;
                
                // Get the raw window handle
                // if let raw_window_handle::RawWindowHandle::Win32(handle) = window.window().raw_window_handle() {
                //     unsafe {
                //         SetWindowPos(
                //             handle.hwnd as HWND,
                //             HWND_TOPMOST,
                //             0, 0, 0, 0,
                //             SWP_NOMOVE | SWP_NOSIZE
                //         );
                //     }
                // }
            }
        });
        
        let window_weak = window.as_weak();
        window.on_close_window(move || {
            if let Some(window) = window_weak.upgrade() {
                window.hide().unwrap();
            }
        });
        
        instance
    }
    
    fn show(&self) {
        self.window.show().unwrap();
    }
}

fn main() -> Result<(), Box<dyn Error>> {
    // 创建一个隐藏的主窗口来保持事件循环
    let main_window = BackgroundWindow::new()?;  // 使用BackgroundWindow而不是MainWindow
    
    // 初始化热键管理器
    let manager = GlobalHotKeyManager::new()?;
    let hotkey = HotKey::new(Some(Modifiers::ALT), Code::KeyQ);
    manager.register(hotkey)?;
    
    let receiver = GlobalHotKeyEvent::receiver();
    println!("截图工具已启动，按 Alt+Q 开始截图");
    
    // 创建一个通道用于同步截图完成事件
    let (tx, rx) = channel();
    
    // 标记当前是否正处理截图
    let mut is_capturing = false;
    
    // 创建一个定时器来检查热键事件
    let timer = slint::Timer::default();
    timer.start(
        slint::TimerMode::Repeated,
        std::time::Duration::from_millis(100),
        move || {
            if let Ok(event) = receiver.try_recv() {
                if event.id == hotkey.id() && !is_capturing {
                    println!("热键事件触发");
                    is_capturing = true;
                    // 创建并显示截图窗口
                    if let Err(e) = show_screenshot_window(tx.clone()) {
                        println!("显示截图窗口失败: {}", e);
                    }
                }
            }
            // 检查截图是否完成
            while let Ok(_) = rx.try_recv() {
                // 截图完成，重置状态
                is_capturing = false;
            }
        },
    );

    // 运行主事件循环
    main_window.run()?;
    
    Ok(())
}

fn show_screenshot_window(tx: std::sync::mpsc::Sender<()>) -> Result<(), Box<dyn Error>> {
    let app = AppWindow::new()?;
    
    // 获取所有屏幕的总区域
    let (total_width, total_height, min_x, min_y) = get_total_screen_area()?;
    let desktop_size = slint::LogicalSize::new(total_width as f32, total_height as f32);
    
    // 设置窗口大小和位置，考虑多显示器的偏移
    app.window().set_size(desktop_size);
    app.window().set_position(LogicalPosition::new(min_x as f32, min_y as f32));
    
    // 添加日志回调
    app.on_debug_log(|msg| {
        println!("UI Debug: {}", msg);
    });
    
    // 处理选区完成
    let app_weak = app.as_weak();
    let tx_clone = tx.clone();
    app.on_selection_complete(move |area| {
        println!("Selection complete callback triggered");
        if let Some(app) = app_weak.upgrade() {
            // 先隐藏整个窗口
            app.hide().unwrap();
            println!("Window hidden");
            
            // 调整截图区域，排除边框（边框宽度为1px）
            let border_width: u32 = 1;
            if let Err(e) = capture_area(
                (area.x as i32 + min_x + border_width as i32), 
                (area.y as i32 + min_y + border_width as i32), 
                (area.width as u32).saturating_sub(border_width * 2), 
                (area.height as u32).saturating_sub(border_width * 2)
            ) {
                println!("截图失败: {}", e);
            }
            
            println!("Screenshot taken");
            // 通知截图完成
            tx_clone.send(()).unwrap();
            println!("Completion signal sent");
        }
    });
    
    // 处理取消截图
    let app_weak = app.as_weak();
    app.on_cancel_capture(move || {
        if let Some(app) = app_weak.upgrade() {
            app.hide().unwrap();
            tx.send(()).unwrap();
        }
    });
    
    // 显示窗口
    println!("Showing window");
    app.show()?;
    println!("Window shown");
    
    Ok(())
}

fn capture_area(x: i32, y: i32, width: u32, height: u32) -> Result<(), Box<dyn Error>> {
    // 确保宽度和高度大于0
    if width == 0 || height == 0 {
        return Ok(());
    }

    let screens = Screen::all()?;
    
    // 找到包含选区的屏幕
    for screen in screens {
        let display_info = screen.display_info;
        if x >= display_info.x 
            && y >= display_info.y 
            && x + width as i32 <= display_info.x + display_info.width as i32
            && y + height as i32 <= display_info.y + display_info.height as i32 {
            
            // 捕获选定区域
            let image = screen.capture_area(
                x - display_info.x, 
                y - display_info.y, 
                width, 
                height
            )?;
            
            // 打印实际的图像尺寸和数据大小，用于调试
            println!("截图尺寸: {}x{}", width, height);
            println!("图像数据大小: {}", image.to_vec().len());
            
            // 保存图片
            let timestamp = Local::now().format("%Y%m%d_%H%M%S").to_string();
            let filename = format!("screenshot_{}.png", timestamp);
            image.save(&filename)?;
            
            // 显示预览窗口
            let preview = PreviewWindowState::new(image.to_vec(), width, height);
            preview.borrow().show();
            
            println!("区域截图已保存为: {}", filename);
            break;
        }
    }

    Ok(())
}

fn get_total_screen_area() -> Result<(i32, i32, i32, i32)> {
    let screens = Screen::all()?;
    let mut min_x = i32::MAX;
    let mut min_y = i32::MAX;
    let mut max_x = i32::MIN;
    let mut max_y = i32::MIN;
    
    for screen in screens {
        let info = screen.display_info;
        min_x = min_x.min(info.x);
        min_y = min_y.min(info.y);
        max_x = max_x.max(info.x + info.width as i32);
        max_y = max_y.max(info.y + info.height as i32);
    }
    
    Ok((max_x - min_x, max_y - min_y, min_x, min_y))
}