use std::ffi::c_void;
use windows::Win32::Graphics::Gdi::BITMAPINFO;

pub struct OffscreenBuffer {
    pub info: BITMAPINFO,
    pub memory: *mut c_void,
    pub width: u32,
    pub height: u32,
    pub bytes_per_pixel: u32,
    pub pitch: u32
}

pub struct WindowDimensions {
    pub width: u32,
    pub height: u32
}

#[derive(Default)]
pub struct GameState {
    pub delta_time: f32,
    pub camera: Camera
}

#[derive(Default, Clone, Copy)]
pub struct Camera {
    pub x: f32,
    pub y: f32,
    pub width: f32,
    pub height: f32,
    pub y_scale: f32
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
        p1: Vector2 { x: 1.0, y: 0.0 },
        p2: Vector2 { x: 1.0, y: 1.0 },
        p3: Vector2 { x: 0.0, y: 1.0 }
    };

    draw_unit_grid(buffer, game_state.camera);
    render_bezier_curve(buffer, game_state.camera, bezier);
}

fn world_space_to_screen_space(camera: Camera, pos: Vector2) -> (u32, u32) {
    let x = ((pos.x - camera.x) * camera.y_scale) as u32;
    let y = ((pos.y - camera.y) * camera.y_scale) as u32;
    (x, y)
}

fn screen_space_to_world_space(camera: Camera, x: u32, y: u32) -> Vector2 {
    let world_x = (camera.x + x as f32) / camera.y_scale;
    let world_y = (camera.y + y as f32) / camera.y_scale;
    Vector2 { x: world_x, y: world_y }
}

unsafe fn clear_buffer(buffer: &mut OffscreenBuffer, color: u32) {
    let mut row: *mut u8 = (*buffer).memory as *mut u8;

    let mut y: u32 = 0;
    while y < (*buffer).height {
        y += 1;

        let mut pixel: *mut u32 = row as *mut u32;

        let mut x: u32 = 0;
        while x < (*buffer).width {
            *pixel = color;

            pixel = pixel.offset(1);
            x += 1;
        }

        row = row.offset((*buffer).pitch as isize);
    }
}

unsafe fn draw_unit_grid(buffer: &mut OffscreenBuffer, camera: Camera) {
    // horizontal lines
    let mut screen_x: u32 = 0;
    while screen_x < buffer.width as u32 {
        let mut y: u32 = 0;
        while y <= camera.height as u32 {
            let camera_y_dec = camera.y - (camera.y as i32) as f32;
            let camera_offset_y = (camera_y_dec + y as f32) * camera.y_scale;

            if camera_offset_y > 0.0 {
                draw_pixel_to_buffer(buffer, screen_x, camera_offset_y as u32, 0xFF444444);
            }

            y += 1;
        }
        screen_x += 1;
    }

    // vertical lines
    let mut screen_y: u32 = 0;
    while screen_y < buffer.height as u32 {
        let mut x: u32 = 0;
        while x <= camera.width as u32 {
            let camera_x_dec = camera.x - (camera.x as i32) as f32;
            let camera_offset_x = (camera_x_dec + x as f32) * camera.y_scale;

            if camera_offset_x > 0.0 {
                draw_pixel_to_buffer(buffer, camera_offset_x as u32, screen_y, 0xFF444444);
            }

            x += 1;
        }
        screen_y += 1;
    }
}

unsafe fn render_bezier_curve(buffer: &mut OffscreenBuffer, camera: Camera, bezier: BezierCurve) {
    let mut t = 0.0;
    while t < 1.0 {
        t += 0.01;

        let p = evaluate_bezier_curve(&bezier, t);
        let pos = world_space_to_screen_space(camera, p);

        if pos.0 > buffer.width || pos.1 > buffer.height { continue; }

        draw_pixel_to_buffer(buffer, pos.0 as u32, pos.1 as u32, 0xFFFFFFFF);
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
