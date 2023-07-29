#![deny(clippy::all)]
#![forbid(unsafe_code)]

use std::time::Instant;

use log::{debug, error};
use num::complex::Complex;
use pixels::{Error, Pixels, SurfaceTexture};
use winit::{
    dpi::LogicalSize,
    event::{Event, VirtualKeyCode},
    event_loop::{ControlFlow, EventLoop},
    window::WindowBuilder,
};
use winit_input_helper::WinitInputHelper;

const WIDTH: u32 = 1000;
const HEIGHT: u32 = 1000;
const MAX_ITERS: usize = 500;
fn main() -> Result<(), Error> {
    let event_loop = EventLoop::new();
    let mut input = WinitInputHelper::new();

    let window = {
        let size = LogicalSize::new(WIDTH as f64, HEIGHT as f64);
        let scaled_size = LogicalSize::new(WIDTH as f64, HEIGHT as f64);
        WindowBuilder::new()
            .with_title("mandelbrot rs")
            .with_inner_size(scaled_size)
            .with_min_inner_size(size)
            .build(&event_loop)
            .unwrap()
    };

    let mut pixels = {
        let window_size = window.inner_size();
        let surface_texture = SurfaceTexture::new(window_size.width, window_size.height, &window);
        Pixels::new(WIDTH, HEIGHT, surface_texture)?
    };

    let mut mandelbrot = MandelbrotGrid::new(WIDTH as usize, HEIGHT as usize);
    mandelbrot.update();
    let mut paused = false;

    let mut draw_state: Option<bool> = None;

    event_loop.run(move |event, _, control_flow| {
        // The one and only event that winit_input_helper doesn't have for us...
        if let Event::RedrawRequested(_) = event {
            mandelbrot.draw(pixels.frame_mut());
            if let Err(err) = pixels.render() {
                error!("pixels.render: {}", err);
                *control_flow = ControlFlow::Exit;
                return;
            }
        }

        // For everything else, for let winit_input_helper collect events to build its state.
        // It returns `true` when it is time to update our game state and request a redraw.
        if input.update(&event) {
            // Close events
            if input.key_pressed(VirtualKeyCode::Escape) || input.close_requested() {
                *control_flow = ControlFlow::Exit;
                return;
            }
            if input.key_pressed(VirtualKeyCode::P) {
                paused = !paused;
            }
            if input.key_pressed_os(VirtualKeyCode::Space) {
                // Space is frame-step, so ensure we're paused
                paused = true;
            }
            // Handle mouse. This is a bit involved since support some simple
            // line drawing (mostly because it makes nice looking patterns).
            let (mouse_cell, mouse_prev_cell) = input
                .mouse()
                .map(|(mx, my)| {
                    let (dx, dy) = input.mouse_diff();
                    let prev_x = mx - dx;
                    let prev_y = my - dy;

                    let (mx_i, my_i) = pixels
                        .window_pos_to_pixel((mx, my))
                        .unwrap_or_else(|pos| pixels.clamp_pixel_pos(pos));

                    let (px_i, py_i) = pixels
                        .window_pos_to_pixel((prev_x, prev_y))
                        .unwrap_or_else(|pos| pixels.clamp_pixel_pos(pos));

                    (
                        (mx_i as isize, my_i as isize),
                        (px_i as isize, py_i as isize),
                    )
                })
                .unwrap_or_default();

            if let Some(draw_alive) = draw_state {
                let release = input.mouse_released(0);
                let held = input.mouse_held(0);
                debug!("Draw at {mouse_prev_cell:?} => {mouse_cell:?}");
                debug!("Mouse held {held:?}, release {release:?}");
                // If they either released (finishing the drawing) or are still
                // in the middle of drawing, keep going.
                if release || held {
                    debug!("Draw line of {draw_alive:?}");
                }
                // If they let go or are otherwise not clicking anymore, stop drawing.
                if release || !held {
                    debug!("Draw end");
                    draw_state = None;
                }
            }
            // Resize the window
            if let Some(size) = input.window_resized() {
                if let Err(err) = pixels.resize_surface(size.width, size.height) {
                    error!("pixels.resize_surface {}", err);
                    *control_flow = ControlFlow::Exit;
                    return;
                }
            }
            let x_step = (mandelbrot.max_x - mandelbrot.min_x) * 0.2;
            let y_step = (mandelbrot.max_y - mandelbrot.min_y) * 0.2;
            if input.key_pressed_os(VirtualKeyCode::W) {
                mandelbrot.min_x = mandelbrot.min_x + x_step;
                mandelbrot.max_x = mandelbrot.max_x - x_step;
                mandelbrot.min_y = mandelbrot.min_y + y_step;
                mandelbrot.max_y = mandelbrot.max_y - y_step;
                mandelbrot.update();
            }
            if input.key_pressed_os(VirtualKeyCode::S) {
                mandelbrot.min_x = mandelbrot.min_x - x_step;
                mandelbrot.max_x = mandelbrot.max_x + x_step;
                mandelbrot.min_y = mandelbrot.min_y - y_step;
                mandelbrot.max_y = mandelbrot.max_y + y_step;
                mandelbrot.update();
            }
            if input.key_pressed_os(VirtualKeyCode::Left) {
                mandelbrot.min_x = mandelbrot.min_x - x_step;
                mandelbrot.max_x = mandelbrot.max_x - x_step;
                mandelbrot.update();
            }
            if input.key_pressed_os(VirtualKeyCode::Right) {
                mandelbrot.min_x = mandelbrot.min_x + x_step;
                mandelbrot.max_x = mandelbrot.max_x + x_step;
                mandelbrot.update();
            }
            if input.key_pressed_os(VirtualKeyCode::Up) {
                mandelbrot.min_y = mandelbrot.min_y - y_step;
                mandelbrot.max_y = mandelbrot.max_y - y_step;
                mandelbrot.update();
            }
            if input.key_pressed_os(VirtualKeyCode::Down) {
                mandelbrot.min_y = mandelbrot.min_y + y_step;
                mandelbrot.max_y = mandelbrot.max_y + y_step;
                mandelbrot.update();
            }
            if input.key_pressed_os(VirtualKeyCode::Space) {
                mandelbrot.update();
            }
            window.request_redraw();
        }
    });
}
fn get_mondelbrot(x: f64, y: f64) -> usize {
    let mut z = Complex::new(0.0, 0.0);
    let c = Complex::new(x, y);
    for i in 0..=MAX_ITERS {
        if z.norm() > 2.0 {
            return i;
        }
        z = z * z + c;
    }
    return MAX_ITERS;
}

#[derive(Clone, Debug, Default)]
struct Cell {
    steps: usize,
    color: Vec<u8>,
}
impl Cell {
    fn new() -> Self {
        Self {
            steps: 0,
            color: vec![0, 0, 0, 0],
        }
    }
}

#[derive(Clone, Debug)]
struct MandelbrotGrid {
    width: usize,
    height: usize,
    cells: Vec<Cell>,
    min_x: f64,
    max_x: f64,
    min_y: f64,
    max_y: f64,
}
impl MandelbrotGrid {
    fn new(width: usize, height: usize) -> Self {
        let size = width.checked_mul(height).expect("too big");
        Self {
            width,
            height,
            cells: vec![Cell::default(); size],
            min_x: -2.5,
            max_x: 2.5,
            min_y: -2.5,
            max_y: 2.5,
        }
    }

    fn update(&mut self) {
        let start_time = Instant::now();
        for y in 0..self.height {
            for x in 0..self.width {
                let idx = x + y * self.width;
                let x = ((x as f64 - 0.) / (self.width as f64 - 0.)) * (self.max_x - self.min_x)
                    + self.min_x;
                let y = ((y as f64 - 0.) / (self.height as f64 - 0.)) * (self.max_y - self.min_y)
                    + self.min_y;
                let steps = get_mondelbrot(x, y);
                //println!("{} {}, {} {}", old_x, x, old_y, y, steps);
                self.cells[idx].steps = steps;
                self.cells[idx].color = steps_to_rgb(steps);
            }
        }
        println!("Update elapsed: {:?}", start_time.elapsed());
    }

    fn draw(&mut self, screen: &mut [u8]) {
        debug_assert_eq!(screen.len(), 4 * self.cells.len());
        for (c, pix) in self.cells.iter().zip(screen.chunks_exact_mut(4)) {
            pix.copy_from_slice(&c.color);
        }
    }
}

fn steps_to_rgb(steps: usize) -> Vec<u8> {
    let norm_steps = steps as f64 / MAX_ITERS as f64;
    let hsl = (
        f64::powf(norm_steps * 360.0, 1.5) % 360.,
        50.,
        norm_steps * 100.,
    );
    let r = (norm_steps * 255.) as u8;
    //println!("{} {}", norm_steps, r);
    //return vec![r, r, r, 255];
    return hsl_to_rgba(hsl.0, hsl.1, hsl.2);
}
fn hsl_to_rgba(h: f64, s: f64, l: f64) -> Vec<u8> {
    // Normalize HSL values
    let h_norm = h / 360.0;
    let s_norm = s / 100.0;
    let l_norm = l / 100.0;

    // Calculate intermediate values
    let c = (1.0 - (2.0 * l_norm - 1.0).abs()) * s_norm;
    let x = c * (1.0 - ((h_norm * 6.0) % 2.0 - 1.0).abs());
    let m = l_norm - c / 2.0;

    // Derive RGB components
    let (r, g, b) = if h_norm < 1.0 / 6.0 {
        (c, x, 0.0)
    } else if h_norm < 2.0 / 6.0 {
        (x, c, 0.0)
    } else if h_norm < 3.0 / 6.0 {
        (0.0, c, x)
    } else if h_norm < 4.0 / 6.0 {
        (0.0, x, c)
    } else if h_norm < 5.0 / 6.0 {
        (x, 0.0, c)
    } else {
        (c, 0.0, x)
    };

    // Denormalize and convert to u8
    let r_u8 = ((r + m) * 255.0) as u8;
    let g_u8 = ((g + m) * 255.0) as u8;
    let b_u8 = ((b + m) * 255.0) as u8;
    let a_u8 = 255; // Alpha (255 means fully opaque)

    vec![r_u8, g_u8, b_u8, a_u8]
}
