extern crate rand;
extern crate console;

extern crate glutin_window;
extern crate graphics;
extern crate opengl_graphics;
extern crate piston;

pub mod chip;

use chip::*;

use std::env;

use glutin_window::GlutinWindow as Window;
use opengl_graphics::{GlGraphics, OpenGL};
use piston::event_loop::*;
use piston::input::*;
use piston::window::WindowSettings;
use piston::{RenderEvent};
use piston_window::color::*;

const SIZE: f64 = 15.0;

fn get_bit_at(input: u64, n: u8) -> bool {
    if n < 64 {
        input & (1 << n) != 0
    } else {
        false
    }
}

struct System {
    gl: GlGraphics,
    chip8: Chip,
}

impl System {
    fn render(&mut self, args: &RenderArgs) {
        use graphics::*;

        let white: [f32; 4] = hex("ffffff");
        let black: [f32; 4] = hex("000000");
        
        let display = self.chip8.display.pixels;

        self.gl.draw(args.viewport(), |c, gl| {
            // Clear the screen.
            clear(black, gl);

            let transform = c
                .transform
                .trans(0.0, 0.0);   
                
            for i in 0..32 {
                let row = display[i];
                for j in 0..64 {
                    if get_bit_at(row, 63 - j) {
                        rectangle(white, [j as f64 * SIZE, i as f64 * SIZE, SIZE, SIZE], transform, gl);
                    }
                }
            }
        });
    }

    fn update(&mut self, _args: &UpdateArgs) {
        self.chip8.new_cycle();
    }
}

fn main() {
    let opengl = OpenGL::V3_2;
    let args: Vec<String> = env::args().collect();

    let mut window: Window = WindowSettings::new("Chip-8", [64 * SIZE as u32, 32 * SIZE as u32])
        .graphics_api(opengl)
        .exit_on_esc(true)
        .build()
        .unwrap();

    // Create a new game and run it.
    let mut system = System {
        gl: GlGraphics::new(opengl),
        chip8: Chip::new(args[1].as_str()),
    };

    let mut events = Events::new(EventSettings::new());//.ups(2);
    while let Some(e) = events.next(&mut window) {
        if let Some(args) = e.render_args() {
            system.render(&args);
        }

        if let Some(args) = e.update_args() {
            system.update(&args);
        }
    }
}
