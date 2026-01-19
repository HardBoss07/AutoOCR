#![windows_subsystem = "windows"]

use arboard::{Clipboard, ImageData};
use device_query::{DeviceQuery, DeviceState, Keycode};
use image::{DynamicImage, RgbImage};
use leptess::LepTess;
use std::{path::PathBuf, thread, time::Duration};
use tray_icon::{
    Icon, TrayIconBuilder,
    menu::{Menu, MenuItem},
};

fn main() {
    // Setup Tray Icon
    let tray_menu = Menu::new();
    let quit_item = MenuItem::new("Quit AutoOCR", true, None);
    tray_menu.append(&quit_item).unwrap();

    let icon = load_icon();
    let _tray_icon = TrayIconBuilder::new()
        .with_menu(Box::new(tray_menu))
        .with_tooltip("AutoOCR - Shift+Alt+O")
        .with_icon(icon)
        .build()
        .unwrap();

    // Spawn the OCR Listener in the background
    thread::spawn(move || {
        let device_state = DeviceState::new();
        let mut clipboard = Clipboard::new().unwrap();

        let mut tessdata_path = std::env::current_exe().unwrap();
        tessdata_path.pop();
        tessdata_path.push("tessdata");

        loop {
            let keys = device_state.get_keys();
            if keys.contains(&Keycode::LShift)
                && keys.contains(&Keycode::LAlt)
                && keys.contains(&Keycode::O)
            {
                if let Ok(image) = clipboard.get_image() {
                    if let Some(text) = perform_ocr(&image, &tessdata_path) {
                        let _ = clipboard.set_text(text);
                    }
                }
                thread::sleep(Duration::from_millis(1000));
            }
            thread::sleep(Duration::from_millis(50));
        }
    });

    // 3. Main thread loop to keep the Tray Icon active
    loop {
        thread::sleep(Duration::from_millis(100));
    }
}

// Helper functions
fn perform_ocr(img: &ImageData, path: &PathBuf) -> Option<String> {
    let mut lt = LepTess::new(Some(path.to_str()?), "eng+deu+fra").ok()?;
    let rgb_data: Vec<u8> = img
        .bytes
        .chunks_exact(4)
        .flat_map(|p| [p[0], p[1], p[2]])
        .collect();

    let img_buffer = RgbImage::from_raw(img.width as u32, img.height as u32, rgb_data)?;
    let dynamic_img = DynamicImage::ImageRgb8(img_buffer);

    let mut buffer = std::io::Cursor::new(Vec::new());
    dynamic_img
        .write_to(&mut buffer, image::ImageFormat::Png)
        .ok()?;

    lt.set_image_from_mem(buffer.get_ref()).ok()?;
    lt.get_utf8_text().ok()
}

fn load_icon() -> Icon {
    let (icon_rgba, icon_width, icon_height) = {
        let image = image::open("assets/favicon.ico")
            .expect("Failed to open icon")
            .into_rgba8();
        let (width, height) = image.dimensions();
        let rgba = image.into_raw();
        (rgba, width, height)
    };
    Icon::from_rgba(icon_rgba, icon_width, icon_height).expect("Failed to open icon")
}
