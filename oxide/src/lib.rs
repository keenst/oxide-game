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

#[derive(Default)]
pub struct GameState {
    pub green_offset: u8,
    pub blue_offset: u8,
    pub delta_time: f32
}

#[derive(Debug, Clone, Copy)]
struct Vector2 {
    x: f32,
    y: f32
}

impl std::ops::Add for Vector2 {
    type Output = Vector2;

    fn add(self, other: Vector2) -> Vector2 {
        Vector2 {
            x: self.x + other.x,
            y: self.y + other.y
        }
    }
}

impl std::ops::Mul<f32> for Vector2 {
    type Output = Vector2;

    fn mul(self, scalar: f32) -> Vector2 {
        Vector2 {
            x: self.x * scalar,
            y: self.y * scalar
        }
    }
}

struct BezierCurve {
    p0: Vector2,
    p1: Vector2,
    p2: Vector2,
    p3: Vector2
}

#[no_mangle]
pub unsafe fn game_update_and_render(game_state: &mut GameState, buffer: &mut OffscreenBuffer) {
    clear_buffer(buffer, 0x00000000);

    let bezier = BezierCurve {
        p0: Vector2 { x: 0.0, y: 0.0 },
        p1: Vector2 { x: 50.0, y: 0.0 },
        p2: Vector2 { x: 50.0, y: 50.0 },
        p3: Vector2 { x: 0.0, y: 50.0 }
    };

    render_bezier_curve(buffer, bezier);
}

unsafe fn clear_buffer(buffer: &mut OffscreenBuffer, color: u32) {
    let mut row: *mut u8 = (*buffer).memory as *mut u8;

    let mut y: i32 = 0;
    while y < (*buffer).height {
        y += 1;

        let mut pixel: *mut u32 = row as *mut u32;

        let mut x: i32 = 0;
        while x < (*buffer).width {
            *pixel = color;

            pixel = pixel.offset(1);
            x += 1;
        }

        row = row.offset((*buffer).pitch as isize);
    }
}

unsafe fn render_bezier_curve(buffer: &mut OffscreenBuffer, bezier: BezierCurve) {
    let mut t = 0.0;
    while t < 1.0 {
        t += 0.01;

        let p = evaluate_bezier_curve(&bezier, t);

        draw_pixel_to_buffer(buffer, p.x as u32, p.y as u32, 0xFFFFFFFF);
    }
}

unsafe fn draw_pixel_to_buffer(buffer: &mut OffscreenBuffer, x: u32, y: u32, color: u32) {
    let mut row: *mut u8 = (*buffer).memory as *mut u8;
    row = row.offset((*buffer).pitch as isize * y as isize);

    let mut pixel: *mut u32 = row as *mut u32;
    pixel = pixel.offset(x as isize);

    *pixel = color;
}

fn evaluate_bezier_curve(bezier: &BezierCurve, t: f32) -> Vector2 {
    bezier.p0 * ((-t).powf(3.0) + 3.0 * t.powf(2.0) - 3.0 * t + 1.0) +
    bezier.p1 * (3.0 * t.powf(3.0) - 6.0 * t.powf(2.0) + 3.0 * t) +
    bezier.p2 * (-3.0 * t.powf(3.0) + 3.0 * t.powf(2.0)) +
    bezier.p3 * (t.powf(3.0))
}
