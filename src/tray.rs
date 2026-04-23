use std::path::Path;
use std::sync::mpsc::Receiver;

use tray_icon::{Icon, TrayIcon, TrayIconBuilder};

pub fn load_icon(path: impl AsRef<Path>) -> Icon {
    let path = path.as_ref();
    let img = image::open(path)
        .unwrap_or_else(|e| panic!("failed to load icon {}: {e}", path.display()))
        .into_rgba8();
    let (w, h) = img.dimensions();
    Icon::from_rgba(img.into_raw(), w, h)
        .unwrap_or_else(|e| panic!("failed to create icon from {}: {e}", path.display()))
}

pub fn build(icon: Icon) -> TrayIcon {
    TrayIconBuilder::new()
        .with_icon(icon)
        .with_tooltip("Dictator")
        .build()
        .expect("failed to create tray icon")
}

pub fn run_loop_tick(seconds: f64) {
    #[link(name = "CoreFoundation", kind = "framework")]
    unsafe extern "C" {
        static kCFRunLoopDefaultMode: *const std::ffi::c_void;
        fn CFRunLoopRunInMode(
            mode: *const std::ffi::c_void,
            seconds: f64,
            returnAfterSourceHandled: bool,
        ) -> i32;
    }
    unsafe {
        CFRunLoopRunInMode(kCFRunLoopDefaultMode, seconds, false);
    }
}

pub fn process_icon_updates(
    tray: &TrayIcon,
    icon_rx: &Receiver<bool>,
    icon_idle: &Icon,
    icon_active: &Icon,
) {
    while let Ok(is_recording) = icon_rx.try_recv() {
        let icon = if is_recording { icon_active.clone() } else { icon_idle.clone() };
        tray.set_icon(Some(icon)).ok();
    }
}
