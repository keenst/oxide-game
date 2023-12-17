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

static mut GAME_UPDATE_AND_RENDER: Option<libloading::Symbol<unsafe extern fn(game_state: &mut GameState, input_controller: &mut InputController, buffer: &mut OffscreenBuffer) -> ()>> = None;
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
    unsafe {
        let window: HWND = create_window().unwrap();

        let window_size = get_window_dimensions(window);

        resize_dib_section(&mut BACK_BUFFER, window_size.width, window_size.height)
            .expect("Unable to resize DIB section");

        let device_context: HDC = GetDC(window);

        let mut game_state = GameState::default();

        game_state.camera = Camera::new(0.0, 0.0, 16.0, 9.0);

        let mut time_last_frame: f64 = 0.0;

        let mut input = InputController::default();

        while IS_RUNNING {
            let mut new_input = input;
            process_pending_messages(&mut new_input);

            input.update(new_input);

            let dimensions = get_window_dimensions(window);

            game_state.camera.y_scale = dimensions.height as f32 / game_state.camera.height;
            game_state.camera.width = dimensions.width as f32 / game_state.camera.y_scale;

            game_update_and_render(&mut game_state, &mut input, &mut BACK_BUFFER);

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
            _ => DefWindowProcA(window, message, w_param, l_param)
        }
    }
}

unsafe fn process_pending_messages(input: &mut InputController) {
    let mut message: MSG = MSG::default();

    while PeekMessageA(&mut message, None, 0, 0, PM_REMOVE).into() {
        match message.message {
            WM_QUIT => {
                IS_RUNNING = false;
            },
            WM_SYSKEYDOWN | WM_KEYDOWN | WM_SYSKEYUP | WM_KEYUP => {
                let vk_code = message.wParam.0 as i32;
                let was_down: bool = (message.lParam.0 & (1 << 30)) != 0;
                let is_down: bool = (message.lParam.0 & (1 << 31)) == 0;

                match vk_code as u8 as char {
                    'W' => input.w.is_down = is_down,
                    'A' => input.a.is_down = is_down,
                    'S' => input.s.is_down = is_down,
                    'D' => input.d.is_down = is_down,
                    _ => match VIRTUAL_KEY(vk_code as u16) {
                        VK_UP => input.up.is_down = is_down,
                        VK_LEFT => input.left.is_down = is_down,
                        VK_DOWN => input.down.is_down = is_down,
                        VK_RIGHT => input.right.is_down = is_down,
                        VK_ESCAPE => input.esc.is_down = is_down,
                        _ => {}
                    }
                }

                if !was_down && is_down {
                    // F5
                    if VIRTUAL_KEY(vk_code as u16) == VK_F5 {
                        println!("reload");
                        crate::reload_lib();
                    }

                    // alt + F4
                    if VIRTUAL_KEY(vk_code as u16) == VK_F4 && message.lParam.0 & (1 << 29) != 0 {
                        IS_RUNNING = false;
                    }
                }
            }
            WM_MOUSEMOVE => {
                let mut mouse_point = POINT::default();
                GetCursorPos(&mut mouse_point).expect("Unable to get cursor position");
                ScreenToClient(message.hwnd, &mut mouse_point);
                input.mouse_state.pos = Vector2u32 {
                    x: mouse_point.x as u32,
                    y: mouse_point.y as u32
                };
            }
            WM_LBUTTONDOWN => input.mouse_state.left.is_down = true,
            WM_LBUTTONUP => input.mouse_state.left.is_down = false,
            WM_RBUTTONDOWN => input.mouse_state.right.is_down = true,
            WM_RBUTTONUP => input.mouse_state.right.is_down = false,
            WM_MBUTTONDOWN => input.mouse_state.middle.is_down = true,
            WM_MBUTTONUP => input.mouse_state.middle.is_down = false,
            WM_MOUSEWHEEL => {
                let wheel_delta = (message.wParam.0 >> 16) as i16;
                input.mouse_state.wheel_delta = wheel_delta;
            }
            _ => {
                TranslateMessage(&message);
                DispatchMessageA(&message);
            }
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

unsafe fn game_update_and_render(game_state: &mut GameState, input_controller: &mut InputController, buffer: &mut OffscreenBuffer) {
    match &GAME_UPDATE_AND_RENDER {
        Some(func) => {
            func(game_state, input_controller, buffer);
        },
        None => {
            let lib = match &LIBRARY {
                Some(value) => value,
                None => {
                    eprintln!("Library not initialized");
                    return
                }
            };

            let func: libloading::Symbol<unsafe extern fn(game_state: &mut GameState, input_controller: &mut InputController, buffer: &mut OffscreenBuffer) -> ()> =
                match lib.get(b"game_update_and_render") {
                    Ok(value) => value,
                    Err(error) => panic!("Unable to get game_update_and_render from oxide: {}", error)
                };

            GAME_UPDATE_AND_RENDER = Some(func.clone());

            func(game_state, input_controller, buffer);
        }
    };
}
