use screenshot::impl_platform;
use slint::{Image, LogicalPosition, ModelRc, SharedPixelBuffer, VecModel};
use anyhow::Result;
use xcap::Monitor;

slint::include_modules!();
fn main() -> Result<()> {
    let main = Main::new()?; 
    let desktop_size = get_desktop_size()?;
    print!("Desktop size: {} x {}\n", desktop_size.width, desktop_size.height);
    main.window().set_size(desktop_size);
    main.window().set_position(LogicalPosition::new(0.0, 0.0));
    let app_clone = main.as_weak();
    main.on_drawing(move || {
        let monitors = Monitor::all().unwrap();
        let image_list = VecModel::from(vec![]);
        for monitor in monitors {
            let image = monitor.capture_image().unwrap();

            let width = monitor.width(); 
            let height = monitor.height(); 
            let data = image.as_raw(); // 获取整个图像的原始数据

            // 将帧数据转换为 SharedPixelBuffer
            let mut buffer = SharedPixelBuffer::new(width, height);
            buffer.make_mut_bytes().copy_from_slice(data); // 复制像素数据到缓冲区
            let image_data =Image::from_rgba8(buffer);
            image_list.push(ImageData {
                x: monitor.x(),            // x 坐标 
                y: monitor.y(),            // y 坐标
                width: width as i32, // 宽度
                height: height as i32, // 高度
                image: image_data, // 图像数据
            });
            print!("Monitor: {} - {} - {} - {}\n", monitor.x(), monitor.y(), monitor.width(), monitor.height());
        }
        app_clone.upgrade().unwrap().set_images(ModelRc::new(image_list));
    });
    main.on_save(move || {
        // 退出程序
        std::process::exit(0);
    });
    main.run()?;
    Ok(())
}

//获取桌面大小 包括多个显示器
fn get_desktop_size() -> Result<slint::LogicalSize> {
    impl_platform::impl_desktop::ImplDesktop::get_desktop_size().map(|(width, height)| slint::LogicalSize::new(width as f32, height as f32))
}
