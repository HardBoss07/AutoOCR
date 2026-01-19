#![windows_subsystem = "windows"]

use arboard::{Clipboard, ImageData};
use device_query::{DeviceQuery, DeviceState, Keycode};
use image::{DynamicImage, RgbImage};
use leptess::LepTess;
use std::{path::PathBuf, thread, time::Duration};
use tray_icon::{
    Icon, TrayIconBuilder,
    menu::{Menu, MenuEvent, MenuItem},
};
use winit::event_loop::{ControlFlow, EventLoop};
use winrt_notification::Toast;

fn main() {
    // Initialize the Winit Event Loop (Necessary for Tray Menu responsiveness)
    let event_loop = EventLoop::new().unwrap();

    // Setup Tray Menu
    let tray_menu = Menu::new();
    let quit_item = MenuItem::new("Quit AutoOCR", true, None);
    let quit_id = quit_item.id();
    tray_menu.append(&quit_item).unwrap();

    let icon = load_icon();
    // Keep this variable in scope to keep the tray icon alive
    let _tray_icon = TrayIconBuilder::new()
        .with_menu(Box::new(tray_menu))
        .with_tooltip("AutoOCR - Shift+Alt+O")
        .with_icon(icon)
        .build()
        .unwrap();

    // Spawn OCR Background Thread
    thread::spawn(move || {
        let device_state = DeviceState::new();
        let mut clipboard = Clipboard::new().unwrap();

        // Define path to tessdata relative to the EXE
        let mut tessdata_path = std::env::current_exe().unwrap();
        tessdata_path.pop();
        tessdata_path.push("tessdata");

        loop {
            let keys = device_state.get_keys();

            // Trigger: Shift + Alt + O
            if keys.contains(&Keycode::LShift)
                && keys.contains(&Keycode::LAlt)
                && keys.contains(&Keycode::O)
            {
                if let Ok(image) = clipboard.get_image() {
                    if let Some(text) = perform_ocr(&image, &tessdata_path) {
                        let cleaned = text.trim().to_string();
                        if !cleaned.is_empty() {
                            let _ = clipboard.set_text(cleaned);
                            notify("AutoOCR", "Text copied to clipboard!");
                        } else {
                            notify("AutoOCR", "OCR finished, but no text found.");
                        }
                    } else {
                        notify("AutoOCR", "OCR Failed. Check tessdata folder.");
                    }
                }
                // Cooldown to prevent multiple triggers in one press
                thread::sleep(Duration::from_millis(1000));
            }
            // Low sleep to keep CPU usage minimal
            thread::sleep(Duration::from_millis(50));
        }
    });

    // Run the Event Loop (Handles Right-Click Menu events)
    event_loop
        .run(move |_event, event_loop_window_target| {
            event_loop_window_target.set_control_flow(ControlFlow::WaitUntil(
                std::time::Instant::now() + Duration::from_millis(100),
            ));

            if let Ok(event) = MenuEvent::receiver().try_recv() {
                if event.id == quit_id {
                    event_loop_window_target.exit();
                }
            }
        })
        .unwrap();
}

fn perform_ocr(img: &ImageData, path: &PathBuf) -> Option<String> {
    // Check for existence to avoid crashing
    if !path.exists() {
        return None;
    }

    // Initialize with your 5 language packs
    let mut lt = LepTess::new(Some(path.to_str()?), "eng+deu+hin+pol+rus").ok()?;

    // Convert RGBA to RGB (strip Alpha channel)
    let mut rgb_data = Vec::with_capacity(img.width * img.height * 3);
    for chunk in img.bytes.chunks_exact(4) {
        rgb_data.push(chunk[0]); // R
        rgb_data.push(chunk[1]); // G
        rgb_data.push(chunk[2]); // B
    }

    // Prepare image for Tesseract via Leptonica
    let img_buffer = RgbImage::from_raw(img.width as u32, img.height as u32, rgb_data)?;
    let mut buffer = std::io::Cursor::new(Vec::new());
    DynamicImage::ImageRgb8(img_buffer)
        .write_to(&mut buffer, image::ImageFormat::Png)
        .ok()?;

    lt.set_image_from_mem(buffer.get_ref()).ok()?;
    lt.get_utf8_text().ok()
}

fn load_icon() -> Icon {
    let mut icon_path = std::env::current_exe().unwrap();
    icon_path.pop();
    icon_path.push("favicon.ico"); // Expecting icon in the same folder as EXE

    let image = image::open(&icon_path)
        .or_else(|_| image::open("assets/favicon.ico")) // Fallback for dev
        .expect("Failed to find favicon.ico")
        .into_rgba8();
    
    let (width, height) = image.dimensions();
    let rgba = image.into_raw();
    Icon::from_rgba(rgba, width, height).expect("Failed to load icon")
}

fn notify(title: &str, message: &str) {
    let _ = Toast::new(Toast::POWERSHELL_APP_ID)
        .title(title)
        .text1(message)
        .show();
}
