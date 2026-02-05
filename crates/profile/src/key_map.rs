use std::collections::HashMap;
use std::sync::OnceLock;

static NAME_TO_SCANCODE: OnceLock<HashMap<&'static str, u16>> = OnceLock::new();
static SCANCODE_TO_NAME: OnceLock<HashMap<u16, &'static str>> = OnceLock::new();

fn init_maps() -> (HashMap<&'static str, u16>, HashMap<u16, &'static str>) {
    let mut n2s = HashMap::new();
    let mut s2n = HashMap::new();

    macro_rules! map {
        ($name:expr, $scancode:expr) => {
            n2s.insert($name, $scancode);
            s2n.insert($scancode, $name);
        };
    }

    // Numbers
    map!("1", 0x02);
    map!("2", 0x03);
    map!("3", 0x04);
    map!("4", 0x05);
    map!("5", 0x06);
    map!("6", 0x07);
    map!("7", 0x08);
    map!("8", 0x09);
    map!("9", 0x0A);
    map!("0", 0x0B);

    // Alpha
    map!("A", 0x1E);
    map!("B", 0x30);
    map!("C", 0x2E);
    map!("D", 0x20);
    map!("E", 0x12);
    map!("F", 0x21);
    map!("G", 0x22);
    map!("H", 0x23);
    map!("I", 0x17);
    map!("J", 0x24);
    map!("K", 0x25);
    map!("L", 0x26);
    map!("M", 0x32);
    map!("N", 0x31);
    map!("O", 0x18);
    map!("P", 0x19);
    map!("Q", 0x10);
    map!("R", 0x13);
    map!("S", 0x1F);
    map!("T", 0x14);
    map!("U", 0x16);
    map!("V", 0x2F);
    map!("W", 0x11);
    map!("X", 0x2D);
    map!("Y", 0x15);
    map!("Z", 0x2C);

    // Function Keys
    map!("F1", 0x3B);
    map!("F2", 0x3C);
    map!("F3", 0x3D);
    map!("F4", 0x3E);
    map!("F5", 0x3F);
    map!("F6", 0x40);
    map!("F7", 0x41);
    map!("F8", 0x42);
    map!("F9", 0x43);
    map!("F10", 0x44);
    map!("F11", 0x57);
    map!("F12", 0x58);

    // Special
    map!("Escape", 0x01);
    map!("Minus", 0x0C);
    map!("Equal", 0x0D);
    map!("Backspace", 0x0E);
    map!("Tab", 0x0F);
    map!("Enter", 0x1C);
    map!("Ctrl", 0x1D); // Left Ctrl
    map!("LeftCtrl", 0x1D);
    map!("SemiColon", 0x27);
    map!("Quote", 0x28);
    map!("BackQuote", 0x29); // Tilde
    map!("LeftShift", 0x2A);
    map!("Shift", 0x2A); // Default to Left Shift
    map!("BackSlash", 0x2B);
    map!("Comma", 0x33);
    map!("Period", 0x34);
    map!("Slash", 0x35);
    map!("RightShift", 0x36);
    map!("LeftAlt", 0x38);
    map!("Alt", 0x38); // Default to Left Alt
    map!("Space", 0x39);
    map!("CapsLock", 0x3A);

    // Numpad (Example subset)
    map!("Num7", 0x47);
    map!("Num8", 0x48);
    map!("Num9", 0x49);
    map!("NumMinus", 0x4A);
    map!("Num4", 0x4B);
    map!("Num5", 0x4C);
    map!("Num6", 0x4D);
    map!("NumPlus", 0x4E);
    map!("Num1", 0x4F);
    map!("Num2", 0x50);
    map!("Num3", 0x51);
    map!("Num0", 0x52);
    map!("NumDot", 0x53);

    // Japanese Keys
    map!("HanZen", 0x29);
    map!("Henkan", 0x79);
    map!("Muhenkan", 0x7B);
    map!("Hiragana", 0x70);

    (n2s, s2n)
}

fn get_maps() -> (
    &'static HashMap<&'static str, u16>,
    &'static HashMap<u16, &'static str>,
) {
    let n2s = NAME_TO_SCANCODE.get_or_init(|| init_maps().0);
    let s2n = SCANCODE_TO_NAME.get_or_init(|| init_maps().1);
    (n2s, s2n)
}

pub fn get_scancode(name: &str) -> Option<u16> {
    if let Some(code) = get_maps().0.get(name) {
        return Some(*code);
    }
    // Fallback for hex strings
    if let Some(hex_part) = name.strip_prefix("0x") {
        return u16::from_str_radix(hex_part, 16).ok();
    }
    None
}

pub fn get_name(scancode: u16) -> Option<&'static str> {
    get_maps().1.get(&scancode).copied()
}
