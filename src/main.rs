use screenshot::impl_platform::{self, impl_hotkey_manager::HotkeyManager};
use slint::{Image, LogicalPosition, ModelRc, SharedPixelBuffer, VecModel, Weak};
use anyhow::Result;
use xcap::{image::{DynamicImage, GenericImage, RgbaImage}, Monitor};
use std::{sync::{mpsc, Arc, Mutex}, thread, time::Duration};

slint::include_modules!();

// 定义一个回调函数类型，要求实现 Send
type Callback = Arc<Mutex<Box<dyn FnMut(Main) + Send>>>;

fn main() -> Result<()>{
    let main = Main::new()?; 
    // 创建一个 Arc<Mutex<i32>>，初始化值为 0
    let app = Arc::new(Mutex::new(main.as_weak()));

    // 创建一个回调函数，使用 Arc<Mutex<...>> 包装以支持跨线程调用
    let callback: Callback = Arc::new(Mutex::new(Box::new(|main| {
        let image_list: VecModel<ImageData> = VecModel::from(vec![]);
        main.set_images(ModelRc::new(image_list));
        // println!("Callback called with value: {}", value);
    })));

    // 创建一个线程
    let app_clone = Arc::clone(&app);
    let callback_clone = Arc::clone(&callback); // 克隆回调

    let thread = thread::spawn(move || {
        // 在线程中修改计数器
        {
            let mut window = app_clone.lock().unwrap().upgrade().unwrap(); // 获取锁
            // 调用回调函数
            let mut callback = callback_clone.lock().unwrap(); // 获取锁
            callback(window); // 使用解引用操作符调用
        }
    });
    Ok(())
}

// fn main() -> Result<()> {
//     let main = Main::new()?; 
//     // let app_clone = main.as_weak();
//     let app =  Arc::new(Mutex::new(main.as_weak()));
//     let app_clone = Arc::clone(&app);
//     // 创建一个共享的二维动态数组
//     let shared_data: Arc<Mutex<Vec<Vec<i32>>>> = Arc::new(Mutex::new(Vec::new()));
//     thread::spawn(move || {
//         // 初始化 image_list
//         let image_list = VecModel::from(vec![]);
//         image_list.push(ImageData {
//             x: 1,
//             y: 1,
//             width: 1,
//             height: 1,
//             image: Image::from_rgba8(SharedPixelBuffer::new(1, 1)),
//         });
//         // let app_clone = Arc::clone(&app);
//         // let app_clone = app.clone();
//         // let t = app_clone.upgrade().unwrap();
//         // let app_clone = t.as_weak();
//         const MOD_CONTROL: u32 = 0x0002; // Ctrl 键修饰符
//         const VK_A: u32 = 0x41;           // A 键虚拟键码x
//         // 注册快捷键 Ctrl + A
//         HotkeyManager::register_hotkey(1, MOD_CONTROL, VK_A, move || {
//             // app_clone_clone;
//             // screenshot(app_clone).unwrap();
//             // let app_clone = t.as_weak();
//             // let image_list: VecModel<ImageData> = VecModel::from(vec![]);
//             // tx.send(image_list).unwrap();
//             // app_clone_clone.upgrade().unwrap().set_images(ModelRc::new(image_list));
//             app_clone.lock().unwrap().upgrade().unwrap().set_images(ModelRc::new(image_list));
//             println!("Control + A pressed!"); // 输出热键触发信息
//         });
//         // app_clone.lock().unwrap().upgrade().unwrap().set_images(ModelRc::new(image_list));
//     });
//     main.run()?;
//     Ok(())
// }

// fn main() -> Result<()> {
//     let main = Main::new()?; 
//     let desktop_size = get_desktop_size()?;
//     print!("Desktop size: {} x {}\n", desktop_size.width, desktop_size.height);
    
//     // 设置窗口大小和位置
//     main.window().set_size(desktop_size);
//     main.window().set_position(LogicalPosition::new(0.0, 0.0));
//     // 创建一个共享的二维动态数组
//     let shared_data: Arc<Mutex<Vec<Vec<i32>>>> = Arc::new(Mutex::new(Vec::new()));
//     let app_clone = main.as_weak();

//     register_hotkey(shared_data.clone(), ||{
//         app_clone.upgrade().unwrap().set_images(ModelRc::new(image_list));
//         println!("Control + A pressed!");
//     });
//     main.run()?;
//     Ok(())
// }


fn screenshot(app_clone: Weak<Main>) -> Result<()> {
        let desktop_size = get_desktop_size()?;
        // 创建一个线程安全的共享 merged_img 对象
        let merged_img = Arc::new(Mutex::new(RgbaImage::new(desktop_size.width as u32, desktop_size.height as u32)));
        let merged_img_clone = Arc::clone(&merged_img);
        let monitors = Monitor::all().unwrap();
        
        // 初始化 image_list
        let image_list: VecModel<ImageData> = VecModel::from(vec![]);

        // 并行处理每个监视器的图像捕获
        monitors.iter().for_each(|monitor| {
            let image = monitor.capture_image().unwrap();
            let width = monitor.width();
            let height = monitor.height();
            let data = image.as_raw();
            
            let mut buffer = SharedPixelBuffer::new(width, height);
            buffer.make_mut_bytes().clone_from_slice(data); // 使用 `clone_from_slice` 复制像素数据
        
            // 生成图像数据
            let image_data = Image::from_rgba8(buffer);

            // // 加锁时只修改 merged_img_lock，而不是整个操作都加锁
            // {
            //     let mut merged_img_lock = merged_img_clone.lock().unwrap();
            //     // 如果需要的话，可以在这里更新 merged_img
            //     let dynamic_image = DynamicImage::ImageRgba8(RgbaImage::from_raw(width, height, data.to_vec()).unwrap());
            //     _ = merged_img_lock.copy_from(&dynamic_image, monitor.x() as u32, monitor.y() as u32);
            // }

            image_list.push(ImageData {
                x: monitor.x(),
                y: monitor.y(),
                width: width as i32,
                height: height as i32,
                image: image_data,
            });

            print!("Monitor: {} - {} - {} - {}\n", monitor.x(), monitor.y(), monitor.width(), monitor.height());
        });
        Ok(())
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
