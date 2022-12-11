#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")] // 不显示 cmd 黑框
use windows_sys::Win32::{
    Foundation::{LPARAM, LRESULT, S_FALSE, WPARAM},
    UI::{
        Input::KeyboardAndMouse::{
            MapVirtualKeyA, VK_CAPITAL, VK_H, VK_I, VK_J, VK_K, VK_L, VK_N, VK_O, VK_OEM_5, VK_U,
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

// 处理 caps 相关逻辑, 返回 true 表示吃掉这个按键, 这个函数会发送按键
unsafe fn keymap_with_caps(kbd_info: &KBDLLHOOKSTRUCT, wparam: WPARAM) -> bool {
    if kbd_info.vkCode == key::CAPS.vk_code as u32 && kbd_info.dwExtraInfo != CAPS_MAGIC_NUMBER {
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
        let key_mapped = CAPS_KEYMAP[kbd_info.vkCode as usize];
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
                WM_KEYUP => "up",
                WM_KEYDOWN => "down",
                WM_SYSKEYDOWN => "sys_down",
                WM_SYSKEYUP => "sys_up",
                _ => "unknow",
            };

            println!("key: {key_name}, state: {key_state}, cap is down: {CAPS_IS_DOWN}");
        }
        if keymap_with_caps(p, wparam) {
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
