extern crate num_complex;
extern crate sdl2;
extern crate core;

use num_complex::Complex64;
use std::ops::Mul;
use std::ops::Add;
use sdl2::event::{Event, WindowEvent};
use sdl2::pixels::Color;
use sdl2::pixels;
use std::time::{Duration, Instant};
use sdl2::render::Canvas;
use sdl2::video::Window;
use sdl2::rect::Rect;
use sdl2::pixels::PixelFormatEnum;
use sdl2::keyboard::Keycode;
use std::collections::HashSet;
use core::cmp;
use sdl2::render::Texture;
use sdl2::surface::Surface;
use sdl2::image::SaveSurface;


#[derive(PartialEq)]
struct RenderParams {
    center: Complex64,
    width: u32,
    height: u32,
    real_domain: f64,
    max_iter: u32,
}

struct RenderProgress {
    done: bool,
    index_x: u32,
    index_y: u32,
    block_size: u32,
}

fn step(canvas: &mut Canvas<Window>, render_params: &RenderParams, render_progress: &mut RenderProgress) {

    let screen_tl_x = render_progress.index_x * render_progress.block_size;
    let screen_tl_y = render_progress.index_y * render_progress.block_size;

    if screen_tl_x > render_params.width {
        render_progress.index_x = 0;
        render_progress.index_y += 1;
    }

    if screen_tl_y > render_params.height {
        render_progress.index_y = 0;
        render_progress.block_size /= 2;
    }

    if render_progress.block_size < 1 {
        render_progress.done = true;
        return;
    }

    let screen_cx = screen_tl_x + (render_progress.block_size / 2);
    let screen_cy = screen_tl_y + (render_progress.block_size / 2);

    let z_0 = screen_to_world(screen_cx, screen_cy, &render_params);

    let n = mandelbrot(z_0, render_params.max_iter);

    let brightness = (255 * n / render_params.max_iter) as u8;
    let color = pixels::Color::RGB(brightness, brightness, brightness);

    canvas.set_draw_color(color);
    canvas.fill_rect(Rect::new(screen_tl_x as i32, screen_tl_y as i32, render_progress.block_size, render_progress.block_size)).unwrap();

    render_progress.index_x += 1;
}

fn screen_to_world(x: u32, y: u32, render_params: &RenderParams) -> Complex64 {
    let w = render_params.width as f64;
    let h = render_params.height as f64;
    let x = x as f64;
    let y = y as f64;
    let complex_domain = render_params.real_domain * h / w;

    let world_re = render_params.center.re + render_params.real_domain * (x - (w / 2.0)) / w;
    let world_im = render_params.center.im + complex_domain * (y - (h / 2.0)) / h;
    return Complex64{re: world_re, im:world_im};
}

fn mandelbrot(z_0: Complex64, max_iter: u32) -> u32 {
    let mut z = z_0.clone();
    let mut n = 0;
    while z.norm_sqr() < 4.0 && n < max_iter {
        z = z.mul(z).add(z_0);
        n+=1;
    }
    return n;
}

fn main() {
    let sdl = sdl2::init().unwrap();
    let video_subsystem = sdl.video().unwrap();
    let window = video_subsystem
        .window("fractal-explorer-rs", 800, 600)
        .resizable()
        .opengl()
        .allow_highdpi()
        .build()
        .unwrap();

    let mut canvas = window.into_canvas()
        .target_texture()
        .present_vsync()
        .build()
        .unwrap();

    let (window_width, window_height) = canvas.output_size().unwrap();

    let creator = canvas.texture_creator();

    let mut texture:Texture = creator
        .create_texture_target(PixelFormatEnum::ARGB8888, window_width, window_height)
        .unwrap();



    canvas.set_draw_color(Color::RGB(0, 0, 0));
    canvas.clear();

    let mut render_params = RenderParams {
        center: (Complex64::new(0.0, 0.0)),
        width: window_width,
        height: window_height,
        real_domain: 4.0,
        max_iter: 64,
    };

    let mut render_progress = RenderProgress {
        done: false,
        index_x: 0,
        index_y: 0,
        block_size: 128,
    };

    let move_up_keys:HashSet<Keycode> = [Keycode::W, Keycode::Up].iter().cloned().collect();
    let move_left_keys:HashSet<Keycode> = [Keycode::A, Keycode::Left].iter().cloned().collect();
    let move_down_keys:HashSet<Keycode> = [Keycode::S, Keycode::Down].iter().cloned().collect();
    let move_right_keys:HashSet<Keycode> = [Keycode::D, Keycode::Right].iter().cloned().collect();
    let zoom_in_keys:HashSet<Keycode> = [Keycode::I].iter().cloned().collect();
    let zoom_out_keys:HashSet<Keycode> = [Keycode::O].iter().cloned().collect();
    let iter_up_keys:HashSet<Keycode> = [Keycode::E].iter().cloned().collect();
    let iter_down_keys:HashSet<Keycode> = [Keycode::Q].iter().cloned().collect();
    let render_keys:HashSet<Keycode> = [Keycode::R].iter().cloned().collect();



    let mut event_pump = sdl.event_pump().unwrap();
    'main: loop {
        let start_time = Instant::now();
        let mut new_render_params = RenderParams {..render_params};
        for event in event_pump.poll_iter() {
            match event {
                Event::Quit {..} => break 'main,
                Event::Window {win_event: WindowEvent::Resized(..), ..} => {
                    let (new_width, new_height) = canvas.output_size().unwrap();
                    new_render_params.width = new_width;
                    new_render_params.height = new_height;
                    texture = creator
                        .create_texture_target(PixelFormatEnum::ARGB8888, new_render_params.width, new_render_params.height)
                        .unwrap();
                },
                Event::KeyDown {keycode:key, ..} => {
                    let key = key.unwrap();
                    if iter_down_keys.contains(&key) {
                        new_render_params.max_iter/=2;
                        new_render_params.max_iter = cmp::max(new_render_params.max_iter, 1);
                    } else if iter_up_keys.contains(&key) {
                        new_render_params.max_iter*=2;
                        new_render_params.max_iter = cmp::min(new_render_params.max_iter, 1<<20);
                    }
                    if render_keys.contains(&key) {
                        canvas.with_texture_canvas(&mut texture, |canvas_texture| {
                            println!("RENDERING TO FILE");
                            let mut buffer = canvas_texture.read_pixels(Rect::new(0,0, render_params.width, render_params.height), PixelFormatEnum::ARGB8888).unwrap();
                            let raw = buffer.as_mut();
                            let surface = Surface::from_data(raw, render_params.width, render_params.height, render_params.width*4, PixelFormatEnum::ARGB8888).unwrap();
                            surface.save("out.png").unwrap();
                        }).unwrap();
                    }
                }
                _ => {},
            }
        }
        let pressed_keys:HashSet<Keycode> = event_pump.keyboard_state().pressed_scancodes().filter_map(Keycode::from_scancode).collect();
        if pressed_keys.intersection(&move_up_keys).count() > 0 {
            new_render_params.center.im -= 0.02 * new_render_params.real_domain;
        }
        if pressed_keys.intersection(&move_down_keys).count() > 0 {
            new_render_params.center.im += 0.02 * new_render_params.real_domain;
        }
        if pressed_keys.intersection(&move_left_keys).count() > 0 {
            new_render_params.center.re -= 0.02 * new_render_params.real_domain;
        }
        if pressed_keys.intersection(&move_right_keys).count() > 0 {
            new_render_params.center.re += 0.02 * new_render_params.real_domain;
        }
        if pressed_keys.intersection(&zoom_in_keys).count() > 0 {
            new_render_params.real_domain *= 0.95;
        }
        if pressed_keys.intersection(&zoom_out_keys).count() > 0 {
            new_render_params.real_domain /= 0.95;
        }

        if render_params != new_render_params {
            render_params = RenderParams{..new_render_params};
            render_progress = RenderProgress {
                done: false,
                index_x: 0,
                index_y: 0,
                block_size: 128,
            };
        }

        if !render_progress.done {
            canvas.with_texture_canvas(&mut texture, |canvas_texture| {
                let mut time_to_think:i64 = 16000000i64 - start_time.elapsed().subsec_nanos() as i64;
                while time_to_think > 0 {
                    step(canvas_texture, &render_params, &mut render_progress);
                    time_to_think = 16000000i64 - start_time.elapsed().subsec_nanos() as i64;
                }
            }).unwrap();
            canvas.clear();
            let screen_rect = Rect::new(0,0, render_params.width, render_params.height);
            canvas.copy(&texture, screen_rect, screen_rect).unwrap();
            canvas.present();
        } else {
            let time_to_sleep:i64 = 16000000i64 - start_time.elapsed().subsec_nanos() as i64;
            if time_to_sleep > 0 {
                ::std::thread::sleep(Duration::new(0, time_to_sleep as u32));
            }
        }
    }
}