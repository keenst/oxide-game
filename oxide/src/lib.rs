use std::ffi::c_void;
use windows::Win32::Graphics::Gdi::BITMAPINFO;

pub struct OffscreenBuffer {
    pub info: BITMAPINFO,
    pub memory: *mut c_void,
    pub width: i32,
    pub height: i32,
    pub bytes_per_pixel: i32,
    pub pitch: i32
}

pub struct WindowDimensions {
    pub width: i32,
    pub height: i32
}

pub struct GameState {
    pub green_offset: u8,
    pub blue_offset: u8
}

#[no_mangle]
pub unsafe fn game_update_and_render(game_state: &mut GameState, buffer: &mut OffscreenBuffer) {
    game_state.blue_offset = ((1 + game_state.blue_offset as i32) % 255) as u8;
    game_state.green_offset = ((1 + game_state.green_offset as i32) % 255) as u8;

    render_weird_gradient(buffer, game_state.blue_offset, game_state.green_offset);
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
