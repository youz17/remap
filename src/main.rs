#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")] // 不显示 cmd 黑框
use windows_sys::Win32::{
    Foundation::*, UI::Input::KeyboardAndMouse::*, UI::WindowsAndMessaging::*,
};

mod key;
use key::KeyInfo;

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

static mut CAPS_IS_DOWN: bool = false;
static mut SWITCH_CAPS: bool = false;

unsafe extern "system" fn low_level_keyboard_proc(
    code: i32,
    wparam: WPARAM,
    lparam: LPARAM,
) -> LRESULT {
    if code as u32 == HC_ACTION {
        let p = &mut *(lparam as *mut KBDLLHOOKSTRUCT);

        #[cfg(debug_assertions)]
        println!(
            "key: {}, CAPS_IS_DOWN: {}, state: {}",
            p.vkCode,
            CAPS_IS_DOWN,
            if wparam == 256 { "down" } else { "up" },
        );
        if p.vkCode == key::CAPS.vk_code as u32 && !SWITCH_CAPS {
            if wparam == WM_KEYDOWN as usize {
                CAPS_IS_DOWN = true;
            } else if wparam == WM_KEYUP as usize {
                CAPS_IS_DOWN = false;
            }
            return S_FALSE as LRESULT;
        } else {
            SWITCH_CAPS = false;
        }

        if CAPS_IS_DOWN {
            let key_mapped = KEYMAP[p.vkCode as usize];

            if key_mapped.vk_code == key::CAPS.vk_code {
                SWITCH_CAPS = true;
            }

            if key_mapped.valid {
                let flag = if key_mapped.e0 {
                    KEYEVENTF_EXTENDEDKEY
                } else {
                    0
                };
                if wparam == WM_KEYDOWN as usize {
                    keybd_event(key_mapped.vk_code, key_mapped.scan_code, flag, 0);
                } else if wparam == WM_KEYUP as usize {
                    keybd_event(
                        key_mapped.vk_code,
                        key_mapped.scan_code,
                        flag | KEYEVENTF_KEYUP,
                        0,
                    );
                }
                return S_FALSE as LRESULT;
            }
        }
    }
    CallNextHookEx(1, code, wparam, lparam)
}

fn main() -> Result<(), ()> {
    unsafe {
        let hook = SetWindowsHookExW(WH_KEYBOARD_LL, Some(low_level_keyboard_proc), 0, 0);
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

        while GetMessageW(&mut messages as *mut MSG, 0, 0, 0) == 1 {
            TranslateMessage(&messages as *const MSG);
            DispatchMessageW(&messages as *const MSG);
        }
        UnhookWindowsHookEx(hook);
    }

    Ok(())
}
