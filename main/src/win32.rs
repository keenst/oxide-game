use crate::oxide::*;
use crate::LIBRARY;
use std::time::SystemTime;
use std::time::UNIX_EPOCH;
use std::ffi::c_void;
use std::mem::size_of;
use std::ptr::null_mut;
use windows::core::*;
use windows::Win32::Foundation::*;
use windows::Win32::Graphics::Gdi::*;
use windows::Win32::System::LibraryLoader::GetModuleHandleA;
use windows::Win32::System::Memory::*;
use windows::Win32::UI::WindowsAndMessaging::*;
use windows::Win32::UI::Input::KeyboardAndMouse::*;

static mut GAME_UPDATE_AND_RENDER: Option<libloading::Symbol<unsafe extern fn(game_state: &mut GameState, buffer: &mut OffscreenBuffer) -> ()>> = None;
static mut IS_RUNNING: bool = true;
// TODO: Figure out how to do this without typing everything out
// default does not work on statics!
static mut BACK_BUFFER: OffscreenBuffer = OffscreenBuffer {
    info: BITMAPINFO {
        bmiHeader: BITMAPINFOHEADER {
            biSize: 0,
            biWidth: 0,
            biHeight: 0,
            biPlanes: 0,
            biBitCount: 0,
            biCompression: 0,
            biSizeImage: 0,
            biXPelsPerMeter: 0,
            biYPelsPerMeter: 0,
            biClrUsed: 0,
            biClrImportant: 0
        },
        bmiColors: [RGBQUAD {
            rgbBlue: 0,
            rgbGreen: 0,
            rgbRed: 0,
            rgbReserved: 0
        }]
    },
    memory: null_mut(),
    width: 0,
    height: 0,
    bytes_per_pixel: 0,
    pitch: 0,
};

pub fn start_program() {
    let mut message: MSG = MSG::default();

    unsafe {
        let window: HWND = create_window().unwrap();

        let window_size = get_window_dimensions(window);

        resize_dib_section(&mut BACK_BUFFER, window_size.width, window_size.height)
            .expect("Unable to resize DIB section");

        let device_context: HDC = GetDC(window);

        let mut game_state = GameState::default();

        game_state.camera = Camera {
            x: -8.5,
            y: -4.5,
            width: 16.0,
            height: 9.0,
            y_scale: 1.0
        };

        let mut time_last_frame: f64 = 0.0;

        while IS_RUNNING {
            while PeekMessageA(&mut message, None, 0, 0, PM_REMOVE).into() {
                if message.message == WM_QUIT {
                    IS_RUNNING = false;
                }

                TranslateMessage(&message);
                DispatchMessageA(&message);
            }

            let dimensions = get_window_dimensions(window);

            game_state.camera.y_scale = dimensions.height as f32 / game_state.camera.height;
            game_state.camera.width = dimensions.width as f32 / game_state.camera.y_scale;

            game_update_and_render(&mut game_state, &mut BACK_BUFFER);

            copy_buffer_to_window(
                &mut BACK_BUFFER,
                device_context,
                dimensions.width,
                dimensions.height)
                .expect("Unable to update window");

            let start = SystemTime::now();
            let current_time = start.duration_since(UNIX_EPOCH).expect("Time went backwards").as_micros() as f64 / 1000.0;
            game_state.delta_time = (current_time - time_last_frame) as f32;
            time_last_frame = current_time;
        }
    }
}

unsafe fn create_window() -> Result<HWND> {
    let instance: HMODULE = GetModuleHandleA(None)?;
    debug_assert!(instance.0 != 0);

    let class_name: PCSTR = s!("window");

    let wc = WNDCLASSA {
        hCursor: LoadCursorW(None, IDC_ARROW)?,
        hInstance: instance.into(),
        lpszClassName: class_name,
        style: CS_HREDRAW | CS_VREDRAW,
        lpfnWndProc: Some(wnd_proc),
        ..Default::default()
    };

    let atom: u16 = RegisterClassA(&wc);
    debug_assert!(atom != 0);

    Ok(CreateWindowExA(
        WINDOW_EX_STYLE::default(),
        class_name,
        s!("Oxide"),
        WS_OVERLAPPEDWINDOW | WS_VISIBLE,
        CW_USEDEFAULT,
        CW_USEDEFAULT,
        CW_USEDEFAULT,
        CW_USEDEFAULT,
        None,
        None,
        instance,
        None))
}

extern "system" fn wnd_proc(window: HWND, message: u32, w_param: WPARAM, l_param: LPARAM) -> LRESULT {
    unsafe {
        match message {
            WM_PAINT => {
                println!("WM_PAINT");

                let mut paint: PAINTSTRUCT = PAINTSTRUCT::default();
                let hdc = BeginPaint(window, &mut paint);

                let dimensions: WindowDimensions = get_window_dimensions(window);

                copy_buffer_to_window(&mut BACK_BUFFER, hdc, dimensions.width, dimensions.height)
                    .expect("Unable to update window");

                EndPaint(window, &mut paint);

                LRESULT(0)
            }
            WM_DESTROY => {
                println!("WM_DESTROY");
                PostQuitMessage(0);
                LRESULT(0)
            }
            WM_CLOSE => {
                println!("WM_CLOSE");
                DestroyWindow(window).expect("Unable to close window");
                LRESULT(0)
            }
            WM_SIZE => {
                println!("WM_SIZE");

                let dimensions = get_window_dimensions(window);
                resize_dib_section(&mut BACK_BUFFER, dimensions.width, dimensions.height)
                    .expect("Unable to resize dib section");

                LRESULT(0)
            }
            WM_SYSKEYDOWN | WM_KEYDOWN | WM_SYSKEYUP | WM_KEYUP => {
                let vk_code: i32 = w_param.0 as i32;
                let was_down: bool = (l_param.0 & (1 << 30)) != 0;
                let is_down: bool = (l_param.0 & (1 << 31)) == 0;
                if was_down != is_down {
                    match vk_code as u8 as char {
                        'W' => {
                            println!("W");
                        }
                        'A' => {}
                        'S' => {}
                        'D' => {}
                        'Q' => {}
                        'E' => {}
                        _ => match VIRTUAL_KEY(vk_code as u16) {
                            VK_UP => {}
                            VK_LEFT => {}
                            VK_DOWN => {}
                            VK_RIGHT => {}
                            VK_ESCAPE => {
                                println!("Escape: ");
                                if is_down {
                                    println!("Is down");
                                } else if was_down {
                                    println!("Was down");
                                }
                            }
                            VK_SPACE => {}
                            VK_F5 => {
                                if is_down {
                                    println!("reload");
                                    crate::reload_lib();
                                }
                            }
                            _ => {}
                        }
                    }
                }

                // alt + F4
                if VIRTUAL_KEY(vk_code as u16) == VK_F4 && l_param.0 & (1 << 29) != 0 {
                    IS_RUNNING = false;
                }

                LRESULT(0)
            }
            _ => DefWindowProcA(window, message, w_param, l_param)
        }
    }
}

unsafe fn resize_dib_section(buffer: &mut OffscreenBuffer, width: u32, height: u32) -> Result<()> {
    if (&buffer).memory != null_mut() {
        VirtualFree((&buffer).memory, 0, MEM_RELEASE)
            .expect("Unable to free memory");
    }

    (*buffer).width = width;
    (*buffer).height = height;
    (*buffer).bytes_per_pixel = 4;

    (*buffer).info = BITMAPINFO::default();

    (*buffer).info.bmiHeader.biSize = size_of::<BITMAPINFOHEADER>() as u32;
    (*buffer).info.bmiHeader.biWidth = (&buffer).width as i32;
    (*buffer).info.bmiHeader.biHeight = -((&buffer).height as i32);
    (*buffer).info.bmiHeader.biPlanes = 1;
    (*buffer).info.bmiHeader.biBitCount = 32;
    (*buffer).info.bmiHeader.biCompression = 0;

    let bitmap_memory_size = (width * height) * (*buffer).bytes_per_pixel;
    (*buffer).memory = VirtualAlloc(
        None,
        bitmap_memory_size.try_into().unwrap(),
        MEM_COMMIT,
        PAGE_READWRITE);

    (*buffer).pitch = (*buffer).width * (*buffer).bytes_per_pixel;

    Ok(())
}

unsafe fn copy_buffer_to_window(buffer: &mut OffscreenBuffer, device_context: HDC, width: u32, height: u32)
    -> Result<()> {
    StretchDIBits(
        device_context,
        0, 0, width as i32, height as i32,
        0, 0, (*buffer).width as i32, (*buffer).height as i32,
        Some((*buffer).memory as *const c_void),
        &(*buffer).info,
        DIB_RGB_COLORS, SRCCOPY);

    Ok(())
}

unsafe fn get_window_dimensions(window: HWND) -> WindowDimensions {
    let mut client_rect: RECT = Default::default();

    let result: Result<()> = GetClientRect(window, &mut client_rect);
    return match result {
        Ok(_) => {
            let width = client_rect.right - client_rect.left;
            let height = client_rect.bottom - client_rect.top;

            WindowDimensions { width: width as u32, height: height as u32 }
        }
        Err(_) => {
            WindowDimensions { width: 0, height: 0 }
        }
    }
}

unsafe fn game_update_and_render(game_state: &mut GameState, buffer: &mut OffscreenBuffer) {
    match &GAME_UPDATE_AND_RENDER {
        Some(func) => {
            func(game_state, buffer);
        },
        None => {
            let lib = match &LIBRARY {
                Some(value) => value,
                None => {
                    eprintln!("Library not initialized");
                    return
                }
            };

            let func: libloading::Symbol<unsafe extern fn(game_state: &mut GameState, buffer: &mut OffscreenBuffer) -> ()> =
                match lib.get(b"game_update_and_render") {
                    Ok(value) => value,
                    Err(error) => panic!("Unable to get game_update_and_render from oxide: {}", error)
                };

            GAME_UPDATE_AND_RENDER = Some(func.clone());

            func(game_state, buffer);
        }
    };
}
