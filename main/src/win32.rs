use crate::*;
use std::{
    ffi::c_void, mem::size_of, ptr, ptr::null_mut
};
use windows::{
    core::*,
    Win32::{
        Foundation::*,
        Graphics::Gdi::*,
        System::LibraryLoader::GetModuleHandleA,
        System::Memory::*,
        UI::{
            WindowsAndMessaging::*,
            Input::KeyboardAndMouse::*
        }
    }
};

struct OffscreenBuffer {
    info: BITMAPINFO,
    memory: *mut c_void,
    width: i32,
    height: i32,
    bytes_per_pixel: i32,
    pitch: i32
}

struct WindowDimensions {
    width: i32,
    height: i32
}

static mut IS_RUNNING: bool = true;
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
static mut X_OFFSET: u8 = 0;
static mut Y_OFFSET: u8 = 0;

pub fn start_program() {
    let mut message: MSG = MSG::default();

    unsafe {
        let window: HWND = create_window().unwrap();

        resize_dib_section(&mut BACK_BUFFER, 480, 270)
            .expect("Unable to resize DIB section");

        let device_context: HDC = GetDC(window);

        let permanent_size: u64 = 64 * 1024 * 1024; // 64MB
        let transient_size: u64 = 4 * 1024 * 1024 * 1024; // 4GB

        let mut game_memory: GameMemory = GameMemory {
            is_initialized: true,
            permanent_storage_size: permanent_size,
            permanent_storage: VirtualAlloc(None, permanent_size as usize, MEM_RESERVE|MEM_COMMIT, PAGE_READWRITE),
            transient_storage_size: transient_size,
            transient_storage: VirtualAlloc(None, transient_size as usize, MEM_RESERVE|MEM_COMMIT, PAGE_READWRITE)
        };

        while IS_RUNNING {
            while PeekMessageA(&mut message, None, 0, 0, PM_REMOVE).into() {
                if message.message == WM_QUIT {
                    IS_RUNNING = false;
                }

                TranslateMessage(&message);
                DispatchMessageA(&message);
            }

            game_update_and_render(&mut game_memory, &mut BACK_BUFFER, X_OFFSET, Y_OFFSET);

            let dimensions: WindowDimensions = get_window_dimensions(window);
            copy_buffer_to_window(
                &mut BACK_BUFFER,
                device_context,
                dimensions.width,
                dimensions.height)
                .expect("Unable to update window");
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

unsafe fn resize_dib_section(buffer: &mut OffscreenBuffer, width: i32, height: i32) -> Result<()> {
    if (&buffer).memory != null_mut() {
        VirtualFree((&buffer).memory, 0, MEM_RELEASE)
            .expect("Unable to free memory");
    }

    (*buffer).width = width;
    (*buffer).height = height;
    (*buffer).bytes_per_pixel = 4;

    (*buffer).info = BITMAPINFO::default();

    (*buffer).info.bmiHeader.biSize = size_of::<BITMAPINFOHEADER>() as u32;
    (*buffer).info.bmiHeader.biWidth = (&buffer).width;
    (*buffer).info.bmiHeader.biHeight = -(&buffer).height;
    (*buffer).info.bmiHeader.biPlanes = 1;
    (*buffer).info.bmiHeader.biBitCount = 32;
    (*buffer).info.bmiHeader.biCompression = 0;

    let bitmap_memory_size: i32 = (width * height) * (*buffer).bytes_per_pixel;
    (*buffer).memory = VirtualAlloc(
        None,
        bitmap_memory_size.try_into().unwrap(),
        MEM_COMMIT,
        PAGE_READWRITE);

    (*buffer).pitch = (*buffer).width * (*buffer).bytes_per_pixel;

    Ok(())
}

unsafe fn copy_buffer_to_window(buffer: &mut OffscreenBuffer, device_context: HDC, width: i32, height: i32)
    -> Result<()> {
    StretchDIBits(
        device_context,
        0, 0, width, height,
        0, 0, (*buffer).width, (*buffer).height,
        Some((*buffer).memory as *const c_void),
        &(*buffer).info,
        DIB_RGB_COLORS, SRCCOPY);

    Ok(())
}

unsafe fn game_update_and_render(memory: &mut GameMemory, buffer: &mut OffscreenBuffer, x_offset: u8, y_offset: u8) {
    let game_state: &mut GameState = ptr::read_unaligned(memory.permanent_storage as *const &mut GameState);
    if !memory.is_initialized {
        game_state.green_offset = 0;
        game_state.blue_offset = 0;
    }
    memory.is_initialized = true;
    render_weird_gradient(buffer, x_offset, y_offset);
}

unsafe fn render_weird_gradient(buffer: &mut OffscreenBuffer, x_offset: u8, y_offset: u8) {
    let mut row: *mut u8 = (*buffer).memory as *mut u8;

    let mut y: i32 = 0;
    while y < (*buffer).height {
        y += 1;

        let mut pixel: *mut u32 = row as *mut u32;

        let mut x: i32 = 0;
        while x < (*buffer).width {
            let blue: u8 = ((x + x_offset as i32) % 255) as u8;
            let green: u8 = ((y + y_offset as i32) % 255) as u8;

            *pixel = (green as u32).wrapping_shl(8) | blue as u32;

            pixel = pixel.offset(1);
            x += 1;
        }

        row = row.offset((*buffer).pitch as isize);
    }
}

unsafe fn get_window_dimensions(window: HWND) -> WindowDimensions {
    let mut client_rect: RECT = Default::default();

    let result: Result<()> = GetClientRect(window, &mut client_rect);
    return match result {
        Ok(_) => {
            let width = client_rect.right - client_rect.left;
            let height = client_rect.bottom - client_rect.top;

            WindowDimensions { width, height }
        }
        Err(_) => {
            WindowDimensions { width: 0, height: 0 }
        }
    }
}
