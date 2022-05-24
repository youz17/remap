#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")] // 不显示 cmd 黑框
use windows_sys::Win32::{
    Foundation::{LPARAM, LRESULT, POINT, S_FALSE, WPARAM},
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

const fn get_keymap() -> [KeyInfo; 256] {
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

const KEYMAP: [KeyInfo; 256] = get_keymap();
const CAPS_MAGIC_NUMBER: usize = 0x534534;

static mut CAPS_IS_DOWN: bool = false;

// If the hook procedure processed the message, it may return a nonzero value to prevent
// the system from passing the message to the rest of the hook chain or the target window
// procedure
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
        if p.vkCode == key::CAPS.vk_code as u32 && p.dwExtraInfo != CAPS_MAGIC_NUMBER {
            if wparam == WM_KEYDOWN as usize {
                CAPS_IS_DOWN = true;
            } else if wparam == WM_KEYUP as usize {
                CAPS_IS_DOWN = false;
            }
            return S_FALSE as LRESULT;
        }

        if CAPS_IS_DOWN {
            let key_mapped = KEYMAP[p.vkCode as usize];
            let extra_info = if key_mapped == key::CAPS {
                CAPS_MAGIC_NUMBER
            } else {
                0
            };

            if key_mapped.valid {
                if wparam == WM_KEYDOWN as usize {
                    send_input(&key_mapped, extra_info, false);
                } else if wparam == WM_KEYUP as usize {
                    send_input(&key_mapped, extra_info, true);
                }
                return S_FALSE as LRESULT;
            }
        }
    }
    CallNextHookEx(1, code, wparam, lparam)
}

fn main() -> Result<(), ()> {
    let hook = unsafe { SetWindowsHookExW(WH_KEYBOARD_LL, Some(low_level_keyboard_proc), 0, 0) };
    if hook == 0 {
        return Err(());
    }
    let mut messages = MSG {
        hwnd: 0,
        message: 0,
        lParam: 0,
        wParam: 0,
        pt: POINT { x: 0, y: 0 },
        time: 0,
    };

    unsafe {
        while GetMessageW(&mut messages as *mut MSG, 0, 0, 0) == 1 {
            TranslateMessage(&messages as *const MSG);
            DispatchMessageW(&messages as *const MSG);
        }
    }

    Ok(())
}
