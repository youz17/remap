#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")] // 不显示 cmd 黑框
use windows_sys::Win32::{
    Foundation::{LPARAM, LRESULT, S_FALSE, WPARAM},
    UI::{
        Input::KeyboardAndMouse::{
            MapVirtualKeyA, VK_0, VK_1, VK_9, VK_B, VK_CAPITAL, VK_G, VK_H, VK_I, VK_J, VK_K, VK_L,
            VK_M, VK_N, VK_O, VK_OEM_3, VK_OEM_5, VK_SPACE, VK_U, VK_Y,
        },
        WindowsAndMessaging::{
            CallNextHookEx, DispatchMessageW, GetMessageW, SetWindowsHookExW, TranslateMessage,
            HC_ACTION, KBDLLHOOKSTRUCT, MSG, WH_KEYBOARD_LL, WM_KEYDOWN, WM_KEYUP, WM_SYSKEYDOWN,
            WM_SYSKEYUP,
        },
    },
};

mod key;
use key::{send_input, KeyInfo};

fn send_key(key: &KeyInfo, extra_info: usize, wparam: u32) {
    match wparam as u32 {
        WM_KEYDOWN | WM_SYSKEYDOWN => {
            send_input(key, extra_info, false);
        }
        WM_KEYUP | WM_SYSKEYUP => {
            send_input(key, extra_info, true);
        }
        _ => {}
    }
}

const fn get_caps_keymap() -> [KeyInfo; 256] {
    let mut map: [KeyInfo; 256] = [KeyInfo::invalid(); 256];
    map[VK_H as usize] = key::LEFT;
    map[VK_J as usize] = key::DOWN;
    map[VK_K as usize] = key::UP;
    map[VK_L as usize] = key::RIGHT;
    map[VK_U as usize] = key::PGUP;
    map[VK_N as usize] = key::PGDOWN;
    map[VK_I as usize] = key::HOME;
    map[VK_O as usize] = key::END;
    map[VK_OEM_5 as usize] = key::CAPS; // For the US standard keyboard, the '\|' key
    map
}

const CAPS_KEYMAP: [KeyInfo; 256] = get_caps_keymap();
// 这个值用于标识这个 caps 是映射后的 caps，在开了两个 remap 进程的时候有用, 远程桌面的时候也有用
// 没查到 dwExtraInfo 有啥用，真出问题的时候再说
const CAPS_MAGIC_NUMBER: usize = 0x534534;

static mut CAPS_IS_DOWN: bool = false;
static mut KEYMAP_LEVEL: i32 = 0;

// 处理 caps 相关逻辑, 返回 true 表示吃掉这个按键, 这个函数会发送按键
unsafe fn keymap_with_caps(kbd_info: &KBDLLHOOKSTRUCT, wparam: WPARAM) -> bool {
    let vk_code = kbd_info.vkCode as u16;
    if vk_code == key::CAPS.vk_code as u16 && kbd_info.dwExtraInfo != CAPS_MAGIC_NUMBER {
        match wparam as u32 {
            WM_KEYDOWN | WM_SYSKEYDOWN => {
                CAPS_IS_DOWN = true;
            }
            WM_KEYUP | WM_SYSKEYUP => {
                CAPS_IS_DOWN = false;
            }
            _ => {}
        }
        return true;
    }

    if CAPS_IS_DOWN {
        // level 切换的部分暂时放这里，其实按我想法，最好的是设置一个特殊键盘值，用于切换 level
        if vk_code >= VK_1 && vk_code <= VK_9 {
            KEYMAP_LEVEL = (vk_code - VK_0) as i32;
            return true;
        } else if vk_code == VK_OEM_3 {
            /* OEM_3 is `~ */
            KEYMAP_LEVEL = 0;
            return true;
        }

        let key_mapped = CAPS_KEYMAP[vk_code as usize];
        let extra_info = if key_mapped == key::CAPS {
            CAPS_MAGIC_NUMBER
        } else {
            0
        };

        if key_mapped.valid {
            send_key(&key_mapped, extra_info, wparam as u32);
            return true;
        }
    }
    return false;
}

const fn get_leveled_keymaps() -> [[KeyInfo; 256]; 1] {
    // 不要在这里映射 caps 相关的
    let mut level1_map = [KeyInfo::invalid(); 256];
    level1_map[VK_B as usize] = key::NUM1;
    level1_map[VK_N as usize] = key::NUM2;
    level1_map[VK_M as usize] = key::NUM3;

    level1_map[VK_G as usize] = key::NUM4;
    level1_map[VK_H as usize] = key::NUM5;
    level1_map[VK_J as usize] = key::NUM6;

    level1_map[VK_Y as usize] = key::NUM7;
    level1_map[VK_U as usize] = key::NUM8;
    level1_map[VK_I as usize] = key::NUM9;

    level1_map[VK_SPACE as usize] = key::NUM0;
    [level1_map]
}
const LEVELED_KEYMAPS: [[KeyInfo; 256]; 1] = get_leveled_keymaps();

unsafe fn keymap_with_level(kbd_info: &KBDLLHOOKSTRUCT, wparam: WPARAM) -> bool {
    if KEYMAP_LEVEL == 0 || KEYMAP_LEVEL as usize > LEVELED_KEYMAPS.len() {
        return false;
    }
    let key_mapped = LEVELED_KEYMAPS[KEYMAP_LEVEL as usize - 1][kbd_info.vkCode as usize];
    if key_mapped.valid {
        send_key(&key_mapped, 0, wparam as u32);
        true
    } else {
        false
    }
}

// 返回 非零值，可以防止接下来的 hook 和 target window 处理这个键
unsafe extern "system" fn low_level_keyboard_proc(
    code: i32,
    wparam: WPARAM,
    lparam: LPARAM,
) -> LRESULT {
    if code as u32 == HC_ACTION {
        let p = &mut *(lparam as *mut KBDLLHOOKSTRUCT);

        #[cfg(debug_assertions)]
        {
            let key_name = match p.vkCode as u16 {
                VK_CAPITAL => "caps".into(),
                VK_OEM_5 => "\\".into(),
                _ => {
                    let c = MapVirtualKeyA(p.vkCode, 2 /* map vk to char */);
                    if c != 0 {
                        format!("'{}'", char::from_u32(c).unwrap())
                    } else {
                        p.vkCode.to_string()
                    }
                }
            };

            let key_state = match wparam as u32 {
                WM_KEYDOWN => "down",
                WM_KEYUP => "up",
                WM_SYSKEYDOWN => "sys_down",
                WM_SYSKEYUP => "sys_up",
                _ => "unknow",
            };

            let vk = p.vkCode;
            println!("key: {key_name}, vk: {vk}, state: {key_state}, cap is down: {CAPS_IS_DOWN}");
        }
        if keymap_with_caps(p, wparam) || keymap_with_level(p, wparam) {
            return S_FALSE as LRESULT;
        }
    }
    CallNextHookEx(1, code, wparam, lparam)
}

fn main() -> Result<(), ()> {
    let hook = unsafe { SetWindowsHookExW(WH_KEYBOARD_LL, Some(low_level_keyboard_proc), 0, 0) };
    if hook == 0 {
        return Err(());
    }
    let mut messages: MSG = unsafe { std::mem::zeroed() };

    unsafe {
        while GetMessageW(&mut messages, 0, 0, 0) == 1 {
            TranslateMessage(&messages);
            DispatchMessageW(&messages);
        }
    }

    Ok(())
}
