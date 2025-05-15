use tauri::command;
use tauri::Builder;
use tauri::Manager;
use tauri_plugin_positioner::Position;
use tauri_plugin_positioner::WindowExt;
use windows::Win32::Foundation::POINT;
use windows::Win32::UI::WindowsAndMessaging::GetCursorPos;

#[command]
fn greet(name: &str) -> String {
    format!("Hello, {}! You've been greeted from Rust!", name)
}

#[command]
fn get_mouse_position() -> (i32, i32) {
    unsafe {
        let mut point = POINT { x: 0, y: 0 };
        // GetCursorPos returns Result<(), Error> in the `windows` crate
        if GetCursorPos(&mut point).is_ok() {
            println!("Mouse position: ({}, {})", point.x, point.y);
            (point.x, point.y)
        } else {
            (-1, -1)
        }
    }
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .setup(|app| {
            #[cfg(desktop)]
            {
                app.handle().plugin(tauri_plugin_positioner::init());
                tauri::tray::TrayIconBuilder::new()
                    .on_tray_icon_event(|tray_handle, event| {
                        tauri_plugin_positioner::on_tray_event(tray_handle.app_handle(), &event);
                    })
                    .build(app)?;

                // move window after tray is setup
                let win = app.get_webview_window("main").unwrap();
                let _ = win.move_window(Position::TopCenter);
            }
            Ok(())
        })
        .plugin(tauri_plugin_opener::init())
        .invoke_handler(tauri::generate_handler![greet, get_mouse_position,])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
