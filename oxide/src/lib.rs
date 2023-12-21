use std::ffi::c_void;
use std::ptr;
use std::cmp::min;
use std::cmp::max;
use windows::Win32::Graphics::Gdi::BITMAPINFO;
use std::time::{SystemTime, UNIX_EPOCH};

#[derive(Clone, Copy, Default)]
pub struct ButtonState {
    pub is_down: bool,
    pub was_down: bool
}

#[derive(Clone, Copy, Default)]
pub struct MouseState {
    pub pos: Vector2u32,
    pub prev_pos: Vector2u32,
    pub left: ButtonState,
    pub right: ButtonState,
    pub middle: ButtonState,
    pub wheel_delta: i16
}

#[derive(Clone, Copy, Default)]
pub struct InputController {
    pub mouse_state: MouseState,
    pub w: ButtonState,
    pub a: ButtonState,
    pub s: ButtonState,
    pub d: ButtonState,
    pub up: ButtonState,
    pub left: ButtonState,
    pub down: ButtonState,
    pub right: ButtonState,
    pub esc: ButtonState
}

impl InputController {
    // Makes all button states was_down and replaces is_down with new_input
    pub fn update(&mut self, new_input: InputController) {
        self.mouse_state.left.was_down = self.mouse_state.left.is_down;
        self.mouse_state.right.was_down = self.mouse_state.right.is_down;
        self.mouse_state.middle.was_down = self.mouse_state.middle.is_down;
        self.mouse_state.prev_pos = self.mouse_state.pos;
        self.w.was_down = self.w.is_down;
        self.a.was_down = self.a.is_down;
        self.s.was_down = self.s.is_down;
        self.d.was_down = self.d.is_down;
        self.up.was_down = self.up.is_down;
        self.left.was_down = self.left.is_down;
        self.down.was_down = self.down.is_down;
        self.right.was_down = self.right.is_down;
        self.esc.was_down = self.esc.is_down;

        self.mouse_state.left.is_down = new_input.mouse_state.left.is_down;
        self.mouse_state.right.is_down = new_input.mouse_state.right.is_down;
        self.mouse_state.middle.is_down = new_input.mouse_state.middle.is_down;
        self.mouse_state.pos = new_input.mouse_state.pos;
        self.w.is_down = new_input.w.is_down;
        self.a.is_down = new_input.a.is_down;
        self.s.is_down = new_input.s.is_down;
        self.d.is_down = new_input.d.is_down;
        self.up.is_down = new_input.up.is_down;
        self.left.is_down = new_input.left.is_down;
        self.down.is_down = new_input.down.is_down;
        self.right.is_down = new_input.right.is_down;
        self.esc.is_down = new_input.esc.is_down;
    }
}

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
    pub camera: Camera,
    pub last_perf_print: u128,
    pub curves: [Option<BezierCurve>; 10],
    pub selected_curve_index: Option<u32>,
    pub selected_control_point: u32
}

#[derive(Default, Clone, Copy)]
pub struct Camera {
    pub x: f32,
    pub y: f32,
    pub width: f32,
    pub height: f32,
    pub y_scale: f32
}

impl Camera {
    pub fn new(x: f32, y: f32, width: f32, height: f32) -> Camera {
        Camera {
            x,
            y,
            width,
            height,
            y_scale: 1.0
        }
    }

    fn get_bounding_box(self) -> Rectangle {
        Rectangle {
            x: self.x - self.width / 2.0,
            y: self.y - self.height / 2.0,
            width: self.width,
            height: self.height
        }
    }
}

#[derive(Default, Debug, Clone, Copy)]
pub struct Vector2 {
    pub x: f32,
    pub y: f32
}

impl Vector2 {
    fn zero() -> Self {
        Vector2 { x: 0.0, y: 0.0 }
    }
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

impl std::ops::AddAssign for Vector2 {
    fn add_assign(&mut self, other: Vector2) {
        *self = Vector2 {
            x: self.x + other.x,
            y: self.y + other.y
        }
    }
}

impl std::ops::Sub for Vector2 {
    type Output = Vector2;

    fn sub(self, other: Vector2) -> Self {
        Vector2 {
            x: self.y - other.x,
            y: self.y - other.y
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

#[derive(Clone, Copy, Default, PartialEq)]
pub struct Vector2u32 {
    pub x: u32,
    pub y: u32
}

impl Vector2u32 {
    pub fn new(value: u32) -> Self {
        Vector2u32 {
            x: value,
            y: value
        }
    }
}

impl std::ops::Add<Vector2u32> for Vector2u32 {
    type Output = Vector2u32;

    fn add(self, other: Vector2u32) -> Vector2u32 {
        Vector2u32 {
            x: self.x + other.x,
            y: self.y + other.y
        }
    }
}

impl std::ops::Sub<Vector2u32> for Vector2u32 {
    type Output = Vector2u32;

    fn sub(self, other: Vector2u32) -> Vector2u32 {
        Vector2u32 {
            x: max(self.x as i32 - other.x as i32, 0) as u32,
            y: max(self.y as i32 - other.y as i32, 0) as u32
        }
    }
}

#[derive(Clone, Copy)]
struct Vector2i32 {
    x: i32,
    y: i32
}

#[derive(Default, Clone, Copy, Debug)]
pub struct Rectangle {
    pub x: f32,
    pub y: f32,
    pub width: f32,
    pub height: f32
}

impl Rectangle {
    fn intersects(&self, other: Rectangle) -> bool {
        self.x + self.width >= other.x ||
        self.y + self.height >= other.y
    }
}

#[derive(Default, Clone, Copy)]
pub struct BezierCurve {
    pub p0: Vector2,
    pub p1: Vector2,
    pub p2: Vector2,
    pub p3: Vector2,
}

impl BezierCurve {
    pub fn new(p0: Vector2, p1: Vector2, p2: Vector2, p3: Vector2) -> Self {
        BezierCurve {
            p0,
            p1,
            p2,
            p3,
        }
    }

    // This function was stolen from here:
    // https://youtu.be/aVwxzDHniEw?si=1txEvDjoTSHT0zqk&t=665
    // NOTE:
    // A faster method might be to evaluate the whole curve and look for min and max values
    // Could be worth benchmarking to see what's faster
    // This is obviously more accurate tho
    fn get_bounding_box(&self) -> Rectangle {
        let tx = {
            let a = -3.0 * self.p0.x + 9.0 * self.p1.x -9.0 * self.p2.x + 3.0 * self.p3.x;
            let b = 6.0 * self.p0.x -12.0 * self.p1.x + 6.0 * self.p2.x;
            let c = -3.0 * self.p0.x + 3.0 * self.p1.x;

            let quad = (b.powf(2.0) - 4.0 * a * c).sqrt();

            // Avoid division by zero
            if a.abs() < 1e-6 {
                let tx0 = -c / b;
                (tx0, tx0)
            } else {
                let tx0 = (-b + quad) / (2.0 * a);
                let tx1 = (-b - quad) / (2.0 * a);
                (tx0, tx1)
            }
        };

        let ty = {
            let a = -3.0 * self.p0.y + 9.0 * self.p1.y -9.0 * self.p2.y + 3.0 * self.p3.y;
            let b = 6.0 * self.p0.y -12.0 * self.p1.y + 6.0 * self.p2.y;
            let c = -3.0 * self.p0.y + 3.0 * self.p1.y;

            let quad = (b.powf(2.0) - 4.0 * a * c).sqrt();

            // Avoid division by zero
            if a.abs() < 1e-6 {
                let ty0 = -c / b;
                (ty0, ty0)
            } else {
                let ty0 = (-b + quad) / (2.0 * a);
                let ty1 = (-b - quad) / (2.0 * a);
                (ty0, ty1)
            }
        };

        let mut points: Vec<Vector2> = Vec::new();

        points.push(self.evaluate(0.0));
        points.push(self.evaluate(1.0));

        if tx.0 < 1.0 && tx.0 > 0.0 {
            let point = self.evaluate(tx.0);
            points.push(point);
        }

        if ty.0 < 1.0 && ty.0 > 0.0 {
            let point = self.evaluate(ty.0);
            points.push(point);
        }

        if tx.1 < 1.0 && tx.1 > 0.0 {
            let point = self.evaluate(tx.1);
            points.push(point);
        }

        if ty.1 < 1.0 && ty.1 > 0.0 {
            let point = self.evaluate(ty.1);
            points.push(point);
        }

        let mut min_x = f32::MAX;
        let mut min_y = f32::MAX;
        let mut max_x = f32::MIN;
        let mut max_y = f32::MIN;

        for point in points {
            if point.x < min_x {
                min_x = point.x;
            } else if point.x > max_x {
                max_x = point.x;
            }

            if point.y < min_y {
                min_y = point.y;
            } else if point.y > max_y {
                max_y = point.y;
            }
        }

        Rectangle {
            x: min_x,
            y: min_y,
            width: max_x - min_x,
            height: max_y - min_y
        }
    }

    fn evaluate(&self, t: f32) -> Vector2 {
        self.p0 * (-t * -t * -t + 3.0 * t * t - 3.0 * t + 1.0) +
        self.p1 * (3.0 * t * t * t - 6.0 * t * t + 3.0 * t) +
        self.p2 * (-3.0 * t * t * t + 3.0 * t * t) +
        self.p3 * (t * t * t)
    }
}

static CAMERA_SPEED: f32 = 0.005;
static CAMERA_SPEED_DIAG: f32 = 0.0035;

#[no_mangle]
pub unsafe fn game_update_and_render(game_state: &mut GameState, input_controller: &mut InputController, buffer: &mut OffscreenBuffer) {
    handle_inputs(*input_controller, game_state);

    clear_buffer(buffer);
    draw_unit_grid(buffer, game_state.camera);
    draw_circle(buffer, game_state.camera, Vector2::zero(), 0.05, 0xFFFF0000);
    draw_bounding_boxes(buffer, game_state);
    draw_bezier_curves(buffer, game_state);
    draw_control_points(buffer, game_state.camera, game_state);

    let start = SystemTime::now();
    let time_now = start.duration_since(UNIX_EPOCH).expect("Time went backwards");
    if time_now.as_millis() - game_state.last_perf_print >= 1000 {
        println!("Frame time: {}", game_state.delta_time);
        println!("FPS: {}", 1000.0 / game_state.delta_time);
        game_state.last_perf_print = time_now.as_millis();
    }
}

fn handle_inputs(input: InputController, game_state: &mut GameState) {
    // Keyboard camera movement
    let move_up = input.w.is_down || input.up.is_down;
    let move_down = input.s.is_down || input.down.is_down;
    let move_left = input.a.is_down || input.left.is_down;
    let move_right = input.d.is_down || input.right.is_down;

    if move_up && move_left {
        (*game_state).camera.x -= CAMERA_SPEED_DIAG * game_state.delta_time;
        (*game_state).camera.y -= CAMERA_SPEED_DIAG * game_state.delta_time;
    } else if move_up && move_right {
        (*game_state).camera.x += CAMERA_SPEED_DIAG * game_state.delta_time;
        (*game_state).camera.y -= CAMERA_SPEED_DIAG * game_state.delta_time;
    } else if move_down && move_left {
        (*game_state).camera.x -= CAMERA_SPEED_DIAG * game_state.delta_time;
        (*game_state).camera.y += CAMERA_SPEED_DIAG * game_state.delta_time;
    } else if move_down && move_right {
        (*game_state).camera.x += CAMERA_SPEED_DIAG * game_state.delta_time;
        (*game_state).camera.y += CAMERA_SPEED_DIAG * game_state.delta_time;
    } else if move_up {
        (*game_state).camera.y -= CAMERA_SPEED * game_state.delta_time;
    } else if move_down {
        (*game_state).camera.y += CAMERA_SPEED * game_state.delta_time;
    } else if move_left {
        (*game_state).camera.x -= CAMERA_SPEED * game_state.delta_time;
    } else if move_right {
        (*game_state).camera.x += CAMERA_SPEED * game_state.delta_time;
    }

    // Mouse left click actions
    let left_down = input.mouse_state.left.is_down;
    let left_released = !left_down && input.mouse_state.left.was_down;
    let left_pressed = left_down && !input.mouse_state.left.was_down;

    if left_released {
        game_state.selected_curve_index = None;
    }

    let cursor_pos_world = screen_space_to_world_space(game_state.camera, input.mouse_state.pos);
    if left_pressed {
        let mut i = 0;
        while i < game_state.curves.len() as u32 {
            match game_state.curves[i as usize] {
                Some(curve) => {
                    if distance_f32(cursor_pos_world, curve.p1) < 0.02 {
                        game_state.selected_curve_index = Some(i);
                        game_state.selected_control_point = 0;
                        break;
                    } else if distance_f32(cursor_pos_world, curve.p2) < 0.02 {
                        game_state.selected_curve_index = Some(i);
                        game_state.selected_control_point = 1;
                        break;
                    }
                },
                None => {}
            }

            i += 1;
        }
    }

    if left_down {
        match game_state.selected_curve_index {
            Some(index) => {
                // Moving control point with mouse
                match &mut game_state.curves[index as usize] {
                    Some(ref mut value) => {
                        if game_state.selected_control_point == 0 {
                            (*value).p1 = cursor_pos_world;
                        } else if game_state.selected_control_point == 1 {
                            (*value).p2 = cursor_pos_world;
                        } else {
                            panic!("Curve can't have {} control points", index + 1);
                        }
                    },
                    None => {
                        panic!("Curve with index {} is None", index);
                    }
                }
            },
            None => {
                // Moving camera with mouse
                let mouse_delta = Vector2i32 {
                    x: input.mouse_state.prev_pos.x as i32 - input.mouse_state.pos.x as i32,
                    y: input.mouse_state.prev_pos.y as i32 - input.mouse_state.pos.y as i32
                };

                (*game_state).camera.x += mouse_delta.x as f32 / game_state.camera.y_scale;
                (*game_state).camera.y += mouse_delta.y as f32 / game_state.camera.y_scale;
            }
        }
    }

    // Reset camera
    let right_clicked = input.mouse_state.right.is_down && !input.mouse_state.right.was_down;
    if right_clicked {
        (*game_state).camera.x = 0.0;
        (*game_state).camera.y = 0.0;
    }
}

fn world_space_to_screen_space(camera: Camera, pos: Vector2) -> Vector2u32 {
    let x = ((pos.x - camera.x + camera.width / 2.0) * camera.y_scale) as u32;
    let y = ((pos.y - camera.y + camera.height / 2.0) * camera.y_scale) as u32;
    Vector2u32 { x, y }
}

fn world_space_to_screen_space_i32(camera: Camera, pos: Vector2) -> Vector2i32 {
    let x = ((pos.x - camera.x + camera.width / 2.0) * camera.y_scale) as i32;
    let y = ((pos.y - camera.y + camera.height / 2.0) * camera.y_scale) as i32;
    Vector2i32 { x, y }
}

fn screen_space_to_world_space(camera: Camera, pos: Vector2u32) -> Vector2 {
    let x = pos.x as f32 / camera.y_scale + camera.x - camera.width / 2.0;
    let y = pos.y as f32 / camera.y_scale + camera.y - camera.height / 2.0;
    Vector2 { x, y }
}

unsafe fn clear_buffer(buffer: &mut OffscreenBuffer) {
    ptr::write_bytes((*buffer).memory, 0u8, (buffer.height * buffer.width * buffer.bytes_per_pixel) as usize);
}

// TODO: ????
unsafe fn draw_unit_grid(buffer: &mut OffscreenBuffer, camera: Camera) {
    // Horizontal lines
    let camera_height_fpart = camera.height / 2.0 - ((camera.height / 2.0) as i32) as f32;
    let camera_y_fpart = if camera.y >= 0.0 {
        camera.y - (camera.y as i32) as f32
    } else {
        1.0 + (camera.y + (camera.y.abs() as i32) as f32)
    };
    let y_offset = camera_height_fpart + camera_y_fpart;

    let mut line_y: u32 = 0;
    while line_y < camera.height as u32 {
        let y = (((line_y as f32 - y_offset) * camera.y_scale) as i32).rem_euclid(buffer.height as i32) as u32;

        let mut x: u32 = 0;
        while x < buffer.width as u32 {
            draw_pixel_to_buffer(buffer, x, y, 0xFF444444);
            x += 1;
        }
        line_y += 1;
    }

    // Vertical lines
    let camera_width_fpart = camera.width / 2.0 - ((camera.width / 2.0) as i32) as f32;
    let camera_x_fpart = if camera.x >= 0.0 {
        camera.x - (camera.x as i32) as f32
    } else {
        1.0 + (camera.x + (camera.x.abs() as i32) as f32)
    };
    let x_offset = camera_width_fpart + camera_x_fpart;

    let mut line_x: u32 = 0;
    while line_x < camera.width as u32 {
        let x = (((line_x as f32 - x_offset) * camera.y_scale) as i32).rem_euclid(buffer.width as i32) as u32;

        let mut y: u32 = 0;
        while y < buffer.height as u32 {
            draw_pixel_to_buffer(buffer, x, y, 0xFF444444);
            y += 1;
        }
        line_x += 1;
    }
}

// TODO: Make sure transparent lines work properly
// TODO: Fix drawing out of bounds
// Xiaolin Wu's line algorithm
unsafe fn draw_line(buffer: &mut OffscreenBuffer, camera: Camera, a: Vector2, b: Vector2, color: u32) {
    let a_screen = world_space_to_screen_space_i32(camera, a);
    let b_screen = world_space_to_screen_space_i32(camera, b);

    let mut x0 = a_screen.x;
    let mut y0 = a_screen.y;
    let mut x1 = b_screen.x;
    let mut y1 = b_screen.y;

    let steep = y1 - y0 > x1 - x0;

    if steep {
        let old_x0 = x0;
        x0 = y0;
        y0 = old_x0;

        let old_x1 = x1;
        x1 = y1;
        y1 = old_x1;
    }

    if x0 > x1 {
        let old_x0 = x0;
        x0 = x1;
        x1 = old_x0;

        let old_y0 = y0;
        y0 = y1;
        y1 = old_y0;
    }

    let gradient = if x1 - x0 == 0 {
        1.0
    } else {
        (y1 - y0) as f32 / (x1 - x0) as f32
    };

    let mut y_intersect = y0 as f32;

    if steep {
        let mut x = x0;
        while x <= x1 {
            let y_intersect_fpart = y_intersect as f32 - (y_intersect as u32) as f32;
            let alpha = ((1.0 - y_intersect_fpart) * 255.0) as u32;
            let color_with_alpha = (color & 0x00FFFFFF) | (alpha << 24);

            draw_pixel_to_buffer(buffer, y_intersect as u32, max(x, 0) as u32, color);
            draw_pixel_to_buffer(buffer, max(y_intersect as i32 - 1, 0) as u32, max(x, 0) as u32, color_with_alpha);

            y_intersect += gradient;
            x += 1;
        }
    } else {
        let mut x = x0;
        while x <= x1 {
            let y_intersect_fpart = y_intersect as f32 - (y_intersect as u32) as f32;
            let alpha = ((1.0 - y_intersect_fpart) * 255.0) as u32;
            let color_with_alpha = (color & 0x00FFFFFF) | (alpha << 24);

            draw_pixel_to_buffer(buffer, max(x, 0) as u32, y_intersect as u32, color);
            draw_pixel_to_buffer(buffer, max(x, 0) as u32, max(y_intersect as i32 - 1, 0) as u32, color_with_alpha);

            y_intersect += gradient;
            x += 1;
        }
    }
}

unsafe fn draw_control_points(buffer: &mut OffscreenBuffer, camera: Camera, game_state: &GameState) {
    for curve in game_state.curves {
        match curve {
            Some(value) => {
                draw_line(buffer, camera, value.p0, value.p1, 0xFF888888);
                draw_line(buffer, camera, value.p2, value.p3, 0xFF888888);

                draw_circle(buffer, camera, value.p1, 0.02, 0xFF00FF00);
                draw_circle(buffer, camera, value.p2, 0.02, 0xFF00FF00);
            },
            None => {
                continue;
            }
        }
    }
}

unsafe fn draw_bounding_boxes(buffer: &mut OffscreenBuffer, game_state: &GameState) {
    let camera_bounding_box = game_state.camera.get_bounding_box();

    for curve in game_state.curves {
        match curve {
            Some(value) => {
                let bounding_box = value.get_bounding_box();

                if !bounding_box.intersects(camera_bounding_box) {
                    continue;
                }

                draw_rectangle(buffer, game_state.camera, bounding_box, 0x3300DDAA);
            }
            None => {
                continue;
            }
        }
    }
}

unsafe fn draw_rectangle(buffer: &mut OffscreenBuffer, camera: Camera, rectangle: Rectangle, color: u32) {
    let rect_top_left = Vector2 {
        x: rectangle.x,
        y: rectangle.y
    };

    let rect_bottom_right = Vector2 {
        x: rectangle.x + rectangle.width,
        y: rectangle.y + rectangle.height
    };

    // Where rectangle starts and ends in screen space
    let rect_top_left_screen = world_space_to_screen_space(camera, rect_top_left);
    let rect_bottom_right_screen = world_space_to_screen_space(camera, rect_bottom_right);

    let start_x = max(rect_top_left_screen.x, 0);
    let start_y = max(rect_top_left_screen.y, 0);
    let end_x = min(rect_bottom_right_screen.x, buffer.width);
    let end_y = min(rect_bottom_right_screen.y, buffer.height);

    let mut x = start_x;
    while x < end_x {
        let mut y = start_y;
        while y < end_y {
            draw_pixel_to_buffer(buffer, x, y, color);
            y += 1;
        }
        x += 1;
    }
}

// TODO: Fix circle staying still when moving between y=0 and y=1 (same for x)
unsafe fn draw_circle(buffer: &mut OffscreenBuffer, camera: Camera, position: Vector2, radius: f32, color: u32) {
    let screen_pos = world_space_to_screen_space_i32(camera, position);
    let screen_radius = (radius * camera.y_scale) as i32;

    let start_x = max(screen_pos.x as i32 - screen_radius, 0) as u32;
    let start_y = max(screen_pos.y as i32 - screen_radius, 0) as u32;
    let end_x = min(max(screen_pos.x + screen_radius, 0), buffer.width as i32 - 1) as u32;
    let end_y = min(max(screen_pos.y + screen_radius, 0), buffer.height as i32 - 1) as u32;

    let mut x = start_x;
    while x <= end_x {
        let mut y = start_y;
        while y <= end_y {
            let dist = distance_i32(screen_pos, Vector2i32 { x: x as i32, y: y as i32 });
            if dist <= screen_radius as f32 {
                draw_pixel_to_buffer(buffer, x, y, color);
            } else if dist <= screen_radius as f32 + 1.0 {
                // TODO: Make it so anti-aliasing works properly with transparent circles
                let dist_dec = dist - (dist as i32) as f32;
                let alpha = ((1.0 - dist_dec) * 255.0) as u32;
                let color_with_alpha = (color & 0x00FFFFFF) | (alpha << 24);
                draw_pixel_to_buffer(buffer, x, y, color_with_alpha);
            }
            y += 1;
        }
        x += 1;
    }
}

fn distance_f32(a: Vector2, b: Vector2) -> f32 {
    let dx = (a.x - b.x).abs();
    let dy = (a.y - b.y).abs();
    (dx * dx + dy * dy).sqrt()
}

fn distance_i32(a: Vector2i32, b: Vector2i32) -> f32 {
    let dx = (a.x as f32 - b.x as f32).abs();
    let dy = (a.y as f32 - b.y as f32).abs();
    (dx * dx + dy * dy).sqrt()
}

unsafe fn draw_bezier_curves(buffer: &mut OffscreenBuffer, game_state: &GameState) {
    for curve in game_state.curves {
        match curve {
            Some(value) => {
                draw_bezier_curve(buffer, game_state.camera, value);
            },
            None => {
                continue;
            }
        }
    }
}

unsafe fn draw_bezier_curve(buffer: &mut OffscreenBuffer, camera: Camera, bezier: BezierCurve) {
    let mut start = bezier.p0;
    let mut i = 0.1;
    while i <= 1.0 {
        let end = bezier.evaluate(i);

        draw_line(buffer, camera, start, end, 0xFFFFFFFF);
        start = end;

        i += 0.1;
    }

    draw_line(buffer, camera, start, bezier.p3, 0xFFFFFFFF);
}

unsafe fn draw_pixel_to_buffer(buffer: &mut OffscreenBuffer, x: u32, y: u32, color: u32) {
    let mut row: *mut u8 = (*buffer).memory as *mut u8;
    row = row.offset((*buffer).pitch as isize * y as isize);

    let mut pixel: *mut u32 = row as *mut u32;
    pixel = pixel.offset(x as isize);

    let alpha = get_alpha(color);
    if alpha == 1.0 {
        *pixel = color;
    } else {
        *pixel = lerp_color(*pixel, color, alpha);
    }
}

fn get_alpha(color: u32) -> f32 {
    (color >> 24) as f32 / 255.0
}

fn lerp_color(a: u32, b: u32, t: f32) -> u32 {
    let a_red = (a >> 16) as u8;
    let a_green = (a >> 8) as u8;
    let a_blue = a as u8;

    let b_red = (b >> 16) as u8;
    let b_green = (b >> 8) as u8;
    let b_blue = b as u8;

    let red = (a_red as f32 + t * (b_red as f32 - a_red as f32)) as u8;
    let green = (a_green as f32 + t * (b_green as f32 - a_green as f32)) as u8;
    let blue = (a_blue as f32 + t * (b_blue as f32 - a_blue as f32)) as u8;

    // NOTE: This version uses fewer variables, if a performance concern
    //let red = (a >> 16) as u8 + (t * ((b >> 16) as u8 - (a >> 16) as u8) as f32) as u8;
    //let green = (a >> 8) as u8 + (t * ((b >> 8) as u8 - (a >> 8) as u8) as f32) as u8;
    //let blue = a as u8 + (t * (b as u8 - a as u8) as f32) as u8;

    ((red as u32) << 16) | ((green as u32) << 8) | blue as u32
}
