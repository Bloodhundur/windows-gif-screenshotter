use fs_extra::dir;
use rdev::{listen, Button, Event, EventType};
use std::thread;
use tauri::{
    command, AppHandle, Builder, Emitter, LogicalPosition, LogicalSize, Manager, WebviewWindow,
};
use tauri_plugin_positioner::{Position, WindowExt};
use windows::Win32::Foundation::POINT;
use windows::Win32::UI::WindowsAndMessaging::GetCursorPos;
use xcap::Monitor;

use gif::{Encoder, Frame, Repeat};
use std::borrow::Cow;
use std::fs::File;
use image::{GenericImageView, DynamicImage};


#[command]
fn greet(name: &str) -> String {
    format!("Hello, {}! You've been greeted from Rust!", name)
}

#[command]
fn get_mouse_position() -> (i32, i32) {
    unsafe {
        let mut point = POINT { x: 0, y: 0 };
        if GetCursorPos(&mut point).is_ok() {
            (point.x, point.y)
        } else {
            (-1, -1)
        }
    }
}

fn screen_shot(pos1: &(i32, i32), pos2: &(i32, i32), overlay: &WebviewWindow) {
    dir::create_all("target/monitors", true).unwrap();

    let x = pos1.0.min(pos2.0);
    let y = pos1.1.min(pos2.1);
    let width = (pos2.0 - pos1.0).abs() as u32;
    let height = (pos2.1 - pos1.1).abs() as u32;

    for i in 1..=3 {
        // Take the screenshot
        let monitor = Monitor::from_point(x, y).unwrap();
        let image = monitor.capture_image().unwrap();

        let path = format!("target/monitors/screenshot-{}x{}-{}.png", width, height,i);
        image.save(&path).unwrap();

        //println!("Screenshot saved to {}", path);
        std::thread::sleep(std::time::Duration::from_secs(1));
    }
    
    convert_pngs_to_gif("target/monitors", "target/screenshot.gif");

}

fn convert_pngs_to_gif(png_folder: &str, output_path: &str) {
    let mut paths: Vec<_> = std::fs::read_dir(png_folder)
        .unwrap()
        .filter_map(|e| e.ok())
        .map(|e| e.path())
        .filter(|p| p.extension().map(|ext| ext == "png").unwrap_or(false))
        .collect();

    paths.sort(); // Ensure they're in order (e.g. -1, -2, -3)

    if paths.is_empty() {
        eprintln!("no pngs found in {}", png_folder);
        return;
    }

    let first_img = image::open(&paths[0]).unwrap().to_rgba8();
    let (width, height) = first_img.dimensions();

    let mut output = File::create(output_path).unwrap();
    let mut encoder: Encoder<&mut File> = Encoder::new(&mut output, width as u16, height as u16, &[]).unwrap();
    encoder.set_repeat(Repeat::Infinite).unwrap(); // Loop forever

    for path in paths {
        let img = image::open(&path).unwrap().to_rgba8();
        let mut pixels = img.into_raw();
        let frame = Frame::from_rgba_speed(width as u16, height as u16, &mut pixels, 10);
        encoder.write_frame(&frame).unwrap();
    }

    println!("giF saved to {}", output_path);
}

use std::time::{Duration, Instant};

fn start_global_mouse_listener(overlay: WebviewWindow) {
    thread::spawn(move || {
        let mut pos1: Option<(i32, i32)> = None;
        let mut last_update = Instant::now();

        if let Err(error) = listen(move |event: Event| {
            match event.event_type {
                EventType::ButtonPress(Button::Left) => {
                    // Store position at press (still use Windows call here)
                    pos1 = Some(get_mouse_position());
                }

                EventType::MouseMove { x, y } => {
                    if let Some(start_pos) = pos1 {
                        if last_update.elapsed() >= Duration::from_millis(16) {
                            // Use the actual event x/y values
                            let current_pos = (x as i32, y as i32);

                            let width = (start_pos.0 - current_pos.0).abs() as i32;
                            let height = (start_pos.1 - current_pos.1).abs() as i32;

                            fn emit_resize_event(
                                window: &tauri::WebviewWindow,
                                posx: i32,
                                posy: i32,
                                width: i32,
                                height: i32,
                            ) {
                                window
                                    .emit("resize_square", (posx, posy, width, height))
                                    .unwrap();
                            }
                            print!("emitted");

                            emit_resize_event(
                                &overlay,
                                current_pos.0,
                                current_pos.1,
                                width,
                                height,
                            );

                            last_update = Instant::now();
                        }
                    }
                }

                EventType::ButtonRelease(Button::Left) => {
                    let end_pos = get_mouse_position();
                    if let Some(start_pos) = pos1 {
                        screen_shot(&start_pos, &end_pos, &overlay);
                        pos1 = None;
                    }
                }

                _ => {}
            }
        }) {
            eprintln!("Error listening to events: {:?}", error);
        }
    });
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    Builder::default()
        .setup(|app| {
            #[cfg(desktop)]
            {
                let overlay = app.get_webview_window("overlay").unwrap();
                //overlay.show().unwrap();

                app.handle()
                    .plugin(tauri_plugin_positioner::init())
                    .unwrap();

                app.handle()
                    .emit("message-from-rust", "Hello from Rust!")
                    .unwrap();

                tauri::tray::TrayIconBuilder::new()
                    .on_tray_icon_event(|tray_handle, event| {
                        tauri_plugin_positioner::on_tray_event(tray_handle.app_handle(), &event);
                    })
                    .build(app)?;

                let win = app.get_webview_window("main").unwrap();
                let monitor = win.primary_monitor()?.unwrap();
                let screen_width = monitor.size().width;
                let screen_height = monitor.size().height;
                let window_height = screen_height / 30;
                let window_width = window_height * 4;

                win.set_size(LogicalSize::new(window_width, window_height))
                    .unwrap();
                win.set_position(LogicalPosition::new(
                    screen_width / 2 - window_width / 2,
                    screen_height / 20 - window_height / 2,
                ))
                .unwrap();

                overlay.set_position(LogicalPosition::new(0, 0)).unwrap();
                overlay
                    .set_size(LogicalSize::new(screen_width, screen_height))
                    .unwrap();

                start_global_mouse_listener(overlay);
            }
            Ok(())
        })
        .plugin(tauri_plugin_opener::init())
        .invoke_handler(tauri::generate_handler![greet, get_mouse_position])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
