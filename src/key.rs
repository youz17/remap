use windows_sys::Win32::UI::Input::KeyboardAndMouse::{
    SendInput, INPUT, INPUT_0, INPUT_KEYBOARD, KEYBDINPUT, KEYEVENTF_EXTENDEDKEY, KEYEVENTF_KEYUP,
    VK_0, VK_1, VK_2, VK_3, VK_4, VK_5, VK_6, VK_7, VK_8, VK_9, VK_CAPITAL, VK_DOWN, VK_END,
    VK_HOME, VK_LEFT, VK_NEXT, VK_PRIOR, VK_RIGHT, VK_UP,
};

#[derive(Clone, Copy, PartialEq, Eq)]
pub struct KeyInfo {
    pub vk_code: u8,
    pub scan_code: u8,
    pub e0: bool,
    pub valid: bool,
}

impl KeyInfo {
    pub const fn new(vk_code: u8, scan_code: u8) -> Self {
        Self {
            vk_code,
            scan_code,
            e0: false,
            valid: true,
        }
    }

    pub const fn with_e0(vk_code: u8, scan_code: u8) -> Self {
        Self {
            vk_code,
            scan_code,
            e0: true,
            valid: true,
        }
    }

    pub const fn invalid() -> Self {
        Self {
            vk_code: 0,
            scan_code: 0,
            e0: false,
            valid: false,
        }
    }
}

pub const CAPS: KeyInfo = KeyInfo::new(VK_CAPITAL as u8, 0x3a);
pub const LEFT: KeyInfo = KeyInfo::with_e0(VK_LEFT as u8, 0x4b);
pub const RIGHT: KeyInfo = KeyInfo::with_e0(VK_RIGHT as u8, 0x4d);
pub const UP: KeyInfo = KeyInfo::with_e0(VK_UP as u8, 0x48);
pub const DOWN: KeyInfo = KeyInfo::with_e0(VK_DOWN as u8, 0x50);
pub const PGUP: KeyInfo = KeyInfo::with_e0(VK_PRIOR as u8, 0x49);
pub const PGDOWN: KeyInfo = KeyInfo::with_e0(VK_NEXT as u8, 0x51);
pub const HOME: KeyInfo = KeyInfo::with_e0(VK_HOME as u8, 0x47);
pub const END: KeyInfo = KeyInfo::with_e0(VK_END as u8, 0x4f);
pub const NUM0: KeyInfo = KeyInfo::new(VK_0 as u8, 0x0b);
pub const NUM1: KeyInfo = KeyInfo::new(VK_1 as u8, 0x02);
pub const NUM2: KeyInfo = KeyInfo::new(VK_2 as u8, 0x03);
pub const NUM3: KeyInfo = KeyInfo::new(VK_3 as u8, 0x04);
pub const NUM4: KeyInfo = KeyInfo::new(VK_4 as u8, 0x05);
pub const NUM5: KeyInfo = KeyInfo::new(VK_5 as u8, 0x06);
pub const NUM6: KeyInfo = KeyInfo::new(VK_6 as u8, 0x07);
pub const NUM7: KeyInfo = KeyInfo::new(VK_7 as u8, 0x08);
pub const NUM8: KeyInfo = KeyInfo::new(VK_8 as u8, 0x09);
pub const NUM9: KeyInfo = KeyInfo::new(VK_9 as u8, 0x0a);

pub fn send_input(key: &KeyInfo, extra_info: usize, up: bool) -> u32 {
    let flag = if up { KEYEVENTF_KEYUP } else { 0 };

    let flags = flag | if key.e0 { KEYEVENTF_EXTENDEDKEY } else { 0 };

    let input = INPUT {
        r#type: INPUT_KEYBOARD,
        Anonymous: INPUT_0 {
            ki: KEYBDINPUT {
                wVk: key.vk_code as u16,
                wScan: key.scan_code as u16,
                dwFlags: flags,
                time: 0,
                dwExtraInfo: extra_info,
            },
        },
    };

    // SendInput 按照 msdn 的说法 在 input 被其他线程阻塞的时候是会失败的, 还有啥 UIPI
    // 这里暂时不知道怎么处理，因为我也不能循环发送让他卡在这里啊。。。
    unsafe {
        SendInput(
            1,
            &input as *const INPUT,
            std::mem::size_of::<INPUT>() as i32,
        )
    }
}
