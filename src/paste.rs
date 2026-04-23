use arboard::Clipboard;
use core_graphics::event::{CGEvent, CGEventFlags, CGEventTapLocation};
use core_graphics::event_source::{CGEventSource, CGEventSourceStateID};

const KEYCODE_V: u16 = 9;

pub fn paste_via_clipboard(clipboard: &mut Clipboard, text: &str) {
    let prev = clipboard.get_text().ok();
    clipboard.set_text(text).expect("failed to set clipboard");
    std::thread::sleep(std::time::Duration::from_millis(50));

    cmd_v();

    std::thread::sleep(std::time::Duration::from_millis(150));

    match prev {
        Some(s) => clipboard.set_text(s).ok(),
        None => clipboard.clear().ok(),
    };
}

fn cmd_v() {
    let source = CGEventSource::new(CGEventSourceStateID::HIDSystemState)
        .expect("failed to create CGEventSource");

    let key_down = CGEvent::new_keyboard_event(source.clone(), KEYCODE_V, true)
        .expect("failed to create key down event");
    key_down.set_flags(CGEventFlags::CGEventFlagCommand);
    key_down.post(CGEventTapLocation::HID);

    let key_up = CGEvent::new_keyboard_event(source, KEYCODE_V, false)
        .expect("failed to create key up event");
    key_up.set_flags(CGEventFlags::CGEventFlagCommand);
    key_up.post(CGEventTapLocation::HID);
}
