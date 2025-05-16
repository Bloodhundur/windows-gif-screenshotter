use fs_extra::dir;
use rdev::{listen, Button, Event, EventType};
use std::thread;
use tauri::{
    command, AppHandle, Builder, LogicalPosition, LogicalSize, Manager, WebviewWindow,
};
use tauri_plugin_positioner::{Position, WindowExt};
use windows::Win32::Foundation::POINT;
use windows::Win32::UI::WindowsAndMessaging::GetCursorPos;
use xcap::Monitor;

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

    // Resize overlay (optional, still keeping it)
    overlay
        .set_position(LogicalPosition::new(x as f64, y as f64))
        .unwrap();
    overlay
        .set_size(LogicalSize::new(width as f64, height as f64))
        .unwrap();

    // Take the screenshot
    let monitor = Monitor::from_point(x, y).unwrap();
    let image = monitor.capture_image().unwrap();

    let path = format!("target/monitors/screenshot-{}x{}.png", width, height);
    image.save(&path).unwrap();

    println!("📸 Screenshot saved to {}", path);
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

                            let width = (start_pos.0 - current_pos.0).abs() as f64;
                            let height = (start_pos.1 - current_pos.1).abs() as f64;

                            overlay
                                .set_position(LogicalPosition::new(
                                    start_pos.0 as f64,
                                    start_pos.1 as f64,
                                ))
                                .ok();
                            overlay
                                .set_size(LogicalSize::new(width, height))
                                .ok();

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
                overlay.show().unwrap();

                app.handle().plugin(tauri_plugin_positioner::init());

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

                win.set_size(LogicalSize::new(window_width, window_height));
                win.set_position(LogicalPosition::new(
                    screen_width / 2 - window_width / 2,
                    screen_height / 20 - window_height / 2,
                ));

                start_global_mouse_listener(overlay);
            }
            Ok(())
        })
        .plugin(tauri_plugin_opener::init())
        .invoke_handler(tauri::generate_handler![greet, get_mouse_position])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
