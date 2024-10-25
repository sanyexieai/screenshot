use screenshot::impl_platform::{self, impl_hotkey_manager::HotkeyManager};
use slint::{Image, LogicalPosition, ModelRc, SharedPixelBuffer, VecModel};
use anyhow::Result;
use xcap::{image::{DynamicImage, GenericImage, RgbaImage}, Monitor};
use std::{sync::{Arc, Mutex}, thread, time::Duration};

slint::include_modules!();

fn main() {
    let manager = HotkeyManager::new();
    const MOD_CONTROL: u32 = 0x0002; // Ctrl 键修饰符
    const VK_A: u32 = 0x41;           // A 键虚拟键码

    // 注册快捷键 Ctrl + A
    manager.register_hotkey(1, MOD_CONTROL, VK_A, || {
        println!("Control + A pressed!"); // 输出热键触发信息
    });

    println!("Listening for hotkeys... Press Ctrl + C to exit.");

    // 保持主线程活着
    loop {
        thread::sleep(Duration::from_secs(1)); // 每隔1秒保持主线程活着
    }
}

// fn main() -> Result<()> {
//     let main = Main::new()?; 
//     let desktop_size = get_desktop_size()?;
//     print!("Desktop size: {} x {}\n", desktop_size.width, desktop_size.height);
    
//     // 设置窗口大小和位置
//     main.window().set_size(desktop_size);
//     main.window().set_position(LogicalPosition::new(0.0, 0.0));

//     // // 创建一个线程安全的共享 merged_img 对象
//     // let merged_img = Arc::new(Mutex::new(RgbaImage::new(desktop_size.width as u32, desktop_size.height as u32)));
//     // let merged_img_clone = Arc::clone(&merged_img);
//     let app_clone = main.as_weak();

//     main.on_drawing(move || {
//         let monitors = Monitor::all().unwrap();
        
//         // 初始化 image_list
//         let image_list = VecModel::from(vec![]);

//         // 并行处理每个监视器的图像捕获
//         monitors.iter().for_each(|monitor| {
//             let image = monitor.capture_image().unwrap();
//             let width = monitor.width();
//             let height = monitor.height();
//             let data = image.as_raw();

//             // 将帧数据转换为 SharedPixelBuffer
//             let mut buffer = SharedPixelBuffer::new(width, height);
//             buffer.make_mut_bytes().copy_from_slice(data); // 复制像素数据到缓冲区

//             // 生成图像数据
//             let image_data = Image::from_rgba8(buffer);

//             // // 加锁时只修改 merged_img_lock，而不是整个操作都加锁
//             // {
//             //     let mut merged_img_lock = merged_img_clone.lock().unwrap();
//             //     // 如果需要的话，可以在这里更新 merged_img
//             //     let dynamic_image = DynamicImage::ImageRgba8(RgbaImage::from_raw(width, height, data.to_vec()).unwrap());
//             //     _ = merged_img_lock.copy_from(&dynamic_image, monitor.x() as u32, monitor.y() as u32);
//             // }

//             image_list.push(ImageData {
//                 x: monitor.x(),
//                 y: monitor.y(),
//                 width: width as i32,
//                 height: height as i32,
//                 image: image_data,
//             });

//             print!("Monitor: {} - {} - {} - {}\n", monitor.x(), monitor.y(), monitor.width(), monitor.height());
//         });

//         app_clone.upgrade().unwrap().set_images(ModelRc::new(image_list));
//     });

//     main.on_save(move || {
//         // let merged_img_lock = merged_img.lock().unwrap(); // 锁定 merged_img
//         // merged_img_lock.save("merged.png").unwrap(); // 保存合并后的图像
//     });

//     main.run()?;
//     Ok(())
// }

// 获取桌面大小，包括多个显示器
fn get_desktop_size() -> Result<slint::LogicalSize> {
    impl_platform::impl_desktop::ImplDesktop::get_desktop_size()
        .map(|(width, height)| slint::LogicalSize::new(width as f32, height as f32))
}
