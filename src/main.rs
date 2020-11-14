pub mod chip8;
pub mod util;
extern crate rand;
extern crate sdl2;
use sdl2::render::{RenderTarget, Texture, Canvas};
use sdl2::pixels::{Color, PixelFormatEnum};
use sdl2::video::{Window};
use sdl2::event::Event;
use sdl2::keyboard::*;
use std::fs;
use std::time::Duration;
use util::*;

type Exit = bool;

const SCALE: u32 = 16;

fn main() -> chip8::Result<()> {
    let mut chip8 = chip8::Chip8::new();
    let game = fs::read("games/TETRIS").unwrap();
    chip8.load(game)?;
    run_sdl(chip8)
}

fn init_canvas(ctx: &sdl2::Sdl) -> Canvas<Window> {
    let video_subsystem = ctx.video().unwrap(); 
    let window = video_subsystem.window("chip-8", 64 * SCALE, 32 * SCALE)
        .position_centered()
        .build()
        .unwrap(); 
    let mut canvas = window.into_canvas().build().unwrap(); 
    canvas.set_scale(SCALE as f32, SCALE as f32).unwrap();
    //canvas.set_blend_mode(sdl2::render::BlendMode::Blend);
    canvas.clear();
    canvas.present();
    canvas
}

fn poll_controls(chip8: &mut chip8::Chip8, event_pump: &mut sdl2::EventPump) -> bool {
    chip8.key = None;
    for e in event_pump.poll_iter() {
        match e {
            Event::Quit {..} |
            Event::KeyDown { keycode: Some(Keycode::Escape), .. } 
                => return true,
            Event::KeyDown { keycode: Some(Keycode::Return), .. } 
                => { chip8.run_once(); },
            _ => ()
        };
    }

    use Scancode::*;
    for s in event_pump.keyboard_state().pressed_scancodes() {
        if chip8.key == None {
            chip8.key = match s {
                X => Some(0x0),
                Num1 => Some(0x1),
                Num2 => Some(0x2),
                Num3 => Some(0x3),
                Q => Some(0x4),
                W => Some(0x5),
                E => Some(0x6),
                A => Some(0x7),
                S => Some(0x8),
                D => Some(0x9),
                Z => Some(0xA),
                C => Some(0xB),
                Num4 => Some(0xC),
                R => Some(0xD),
                F => Some(0xE),
                V => Some(0xF),
                _ => None,
            };
        }
    }
    false
}

pub fn run_sdl(mut chip8: chip8::Chip8) -> chip8::Result<()>{
    let sdl_context = sdl2::init().unwrap();
    let mut event_pump = sdl_context.event_pump().unwrap();
    let mut canvas = init_canvas(&sdl_context);
    let texture_creator = canvas.texture_creator();
    let mut tex = texture_creator.create_texture_streaming(PixelFormatEnum::RGBA8888, 64, 32).unwrap();
    'running: loop {
        canvas.clear();
        canvas.set_draw_color(Color::BLACK);
        if chip8.show {
            render(&mut chip8, &mut canvas, &mut tex)?;
            canvas.present();
        }
        match poll_controls(&mut chip8, &mut event_pump) {
            false => {}
            true => break 'running
        };
        // The rest of the game loop goes here...
        chip8.run_once()?;
        ::std::thread::sleep(Duration::new(0, 1_000_000_000u32 / 600));
    }
    Ok(())
}

pub fn render<T: RenderTarget>(chip8: &mut chip8::Chip8, canvas: &mut Canvas<T>, tex: &mut Texture) 
-> chip8::Result<()> { 
    let mut v = Vec::new();
    for y in 0..32 {
        for x in 0..64 {
            let c = match chip8.display.get_pixel(x, y) {
                true => Color::WHITE,
                false => Color::BLACK,
            };
            let mut unpacked = vec![c.a, c.b, c.g, c.r];
            v.append(&mut unpacked);
        }
    } 
    tex.update(None, &v, 256).unwrap();
    canvas.copy(tex, None, None).unwrap();
    Ok(())
}

