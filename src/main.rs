// #![windows_subsystem = "windows"]
use screenshots::Screen;
use slint::LogicalPosition;
use std::error::Error;
use anyhow::Result;
use global_hotkey::{GlobalHotKeyManager, hotkey::{HotKey, Modifiers, Code}, GlobalHotKeyEvent};
use std::sync::mpsc::channel;
use std::rc::Rc;
use std::cell::RefCell;
use arboard::Clipboard;
use image::{ImageBuffer, Rgba, RgbaImage};
use tray_icon::{Icon, TrayIcon, TrayIconBuilder, menu::{Menu, MenuEvent, MenuItem}};


// 导入UI组件
slint::include_modules!();

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
                buffer[dst_idx] = image_data[src_idx];     // R
                buffer[dst_idx + 1] = image_data[src_idx + 1]; // G
                buffer[dst_idx + 2] = image_data[src_idx + 2];     // B
                buffer[dst_idx + 3] = image_data[src_idx + 3]; // A
            }
        }
        
        let slint_image = slint::Image::from_rgba8(pixel_buffer);
        window.set_screenshot(slint_image);
        
        let instance = Rc::new(RefCell::new(Self {
            window: window.clone(),
        }));
        let window_weak = window.as_weak();
        window.on_close_window(move || {
            if let Some(window) = window_weak.upgrade() {
                window.hide().unwrap();
            }
        });
        
        let window_weak = window.as_weak();
        window.on_move_window(move |offset_x, offset_y| {
            if let Some(window) = window_weak.upgrade() {
                let pos = window.window().position();
                let scale = window.window().scale_factor();
                let logical_pos = pos.to_logical(scale);
                window.window().set_position(slint::LogicalPosition::new(
                    logical_pos.x + offset_x,
                    logical_pos.y + offset_y
                ));
            }
        });
        instance
    }
    
    fn show(&self) {
        self.window.show().unwrap();
    }
}

fn main() -> Result<(), Box<dyn Error>> {
    // 创建托盘菜单
    let menu = Menu::new();
    let quit_item = MenuItem::new("退出", true, None);
    menu.append_items(&[&quit_item])?;
    
    // 创建托盘图标
    let icon = Icon::from_path("resources/app.ico", None)?;
    let _tray_icon = TrayIconBuilder::new()
        .with_menu(Box::new(menu))
        .with_tooltip("截图工具")
        .with_icon(icon)
        .build()?;
    
    // 处理托盘菜单事件
    let menu_channel = MenuEvent::receiver();
    let timer_menu = slint::Timer::default();
    timer_menu.start(
        slint::TimerMode::Repeated,
        std::time::Duration::from_millis(100),
        move || {
            if let Ok(event) = menu_channel.try_recv() {
                if event.id == quit_item.id() {
                    std::process::exit(0);
                }
            }
        },
    );
    
    // 创建一个隐藏的主窗口来保持事件循环
    let main_window = BackgroundWindow::new()?;  // 使用BackgroundWindow而不是MainWindow
        
    // 初始化热键管理器
    let manager = GlobalHotKeyManager::new()?;
    let hotkey_alt_q = HotKey::new(Some(Modifiers::ALT), Code::KeyQ);
    let hotkey_esc = HotKey::new(None, Code::Escape);
    manager.register(hotkey_alt_q)?;
    manager.register(hotkey_esc)?;
    
    let receiver = GlobalHotKeyEvent::receiver();
    println!("截图工具已启动，按 Alt+Q 开始截图");
    
    // 创建一个通道用于同步截图完成事件
    let (tx, rx) = channel();
    
    // 标记当前是否正处理截图
    let mut is_capturing = false;
    let mut app_weak = None::<slint::Weak<AppWindow>>;
    
    // 创建一个定时器来检查热键事件
    let timer = slint::Timer::default();
    timer.start(
        slint::TimerMode::Repeated,
        std::time::Duration::from_millis(100),
        move || {
            if let Ok(event) = receiver.try_recv() {
                if event.id == hotkey_alt_q.id() && !is_capturing {
                    is_capturing = true;
                    if let Ok(weak) = show_screenshot_window(tx.clone()) {
                        app_weak = Some(weak);
                    }
                } else if event.id == hotkey_esc.id() && is_capturing {
                    if let Some(weak) = &app_weak {
                        if let Some(app) = weak.upgrade() {
                            app.hide().unwrap();
                            tx.send(()).unwrap();
                        }
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

fn show_screenshot_window(tx: std::sync::mpsc::Sender<()>) -> Result<slint::Weak<AppWindow>, Box<dyn Error>> {
    let app = AppWindow::new()?;
    let weak = app.as_weak();
    
    // 获取所有屏幕的总区域
    let (total_width, total_height, min_x, min_y) = get_total_screen_area()?;
    let desktop_size = slint::LogicalSize::new(total_width as f32, total_height as f32);
    
    // 设置窗口大小和位置，考虑多显示器的偏移
    app.window().set_size(desktop_size);
    app.window().set_position(LogicalPosition::new(min_x as f32, min_y as f32));
    
    // 在显示窗口之前设置遮罩
    app.set_show_mask(true);
    
    // 添加日志回调
    app.on_debug_log(|msg| {
        println!("UI Debug: {}", msg);
    });
    
    // 处理选区完成
    let app_weak = app.as_weak();
    let tx_clone = tx.clone();
    app.on_selection_complete(move |area| {
        if let Some(app) = app_weak.upgrade() {
            app.set_show_decorations(false);
            app.hide().unwrap();
            // 调整截图区域，避免绿边
            let capture_x = area.x as i32 + min_x + 2;  // 向右偏移1像素
            let capture_y = area.y as i32 + min_y + 2;  // 向下偏移1像素
            let capture_width = area.width as u32 - 4;   // 宽度减少2像素
            let capture_height = area.height as u32 - 4;  // 高度减少2像素
            
            // 延时200ms
            std::thread::sleep(std::time::Duration::from_millis(200));
            if let Err(e) = capture_area(
                capture_x,
                capture_y,
                capture_width,
                capture_height
            ) {
                println!("截图失败: {}", e);
            }
            
            tx_clone.send(()).unwrap();
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
    
    Ok(weak)
}

fn capture_area(x: i32, y: i32, width: u32, height: u32) -> Result<(), Box<dyn Error>> {
    if width == 0 || height == 0 {
        return Ok(());
    }

    let screens = Screen::all()?;
    
    // 创建一个完整的图像缓冲区
    let mut full_image = RgbaImage::new(width, height);
    
    // 遍历所有屏幕，将各部分截图拼接到完整图像中
    for screen in screens {
        let display_info = screen.display_info;
        
        // 计算当前屏幕与选区的交叉区域
        let screen_x = display_info.x;
        let screen_y = display_info.y;
        let screen_right = screen_x + display_info.width as i32;
        let screen_bottom = screen_y + display_info.height as i32;
        
        let capture_right = x + width as i32;
        let capture_bottom = y + height as i32;
        
        // 计算交叉区域
        let intersect_left = x.max(screen_x);
        let intersect_top = y.max(screen_y);
        let intersect_right = capture_right.min(screen_right);
        let intersect_bottom = capture_bottom.min(screen_bottom);
        
        // 如果有交叉区域
        if intersect_left < intersect_right && intersect_top < intersect_bottom {
            // 计算在完整图像中的偏移
            let offset_x = (intersect_left - x) as u32;
            let offset_y = (intersect_top - y) as u32;
            
            // 计算需要捕获的区域大小
            let capture_width = (intersect_right - intersect_left) as u32;
            let capture_height = (intersect_bottom - intersect_top) as u32;
            
            // 捕获当前屏幕的部分
            let part_image = screen.capture_area(
                intersect_left - screen_x,
                intersect_top - screen_y,
                capture_width,
                capture_height
            )?;
            
            // 将部分图像复制到完整图像中
            let part_data = part_image.to_vec();
            for y in 0..capture_height {
                for x in 0..capture_width {
                    let src_idx = ((y * capture_width + x) * 4) as usize;
                    if src_idx + 3 < part_data.len() {
                        full_image.put_pixel(
                            offset_x + x,
                            offset_y + y,
                            Rgba([
                                part_data[src_idx],
                                part_data[src_idx + 1],
                                part_data[src_idx + 2],
                                255
                            ])
                        );
                    }
                }
            }
        }
    }
    
    // 复制到剪贴板
    let mut clipboard = Clipboard::new()?;
    let image_data = full_image.clone().into_raw();
    clipboard.set_image(arboard::ImageData {
        width: width as usize,
        height: height as usize,
        bytes: image_data.clone().into(),
    })?;
    
    // 创建预览窗口
    let preview = PreviewWindowState::new(image_data, width, height);
    // 打印截图区域
    println!("截图区域: x={}, y={}, width={}, height={}", x, y, width, height);
    preview.borrow().window.window().set_position(slint::LogicalPosition::new(
        x as f32,
        y as f32
    ));
    preview.borrow().show();

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