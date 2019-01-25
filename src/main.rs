extern crate sdl2;

mod gb;
use gb::GameBoy;

use std::env;
use std::fs;
use std::process;

use sdl2::event::Event;
use sdl2::rect::Rect;

const X_RES: u32 = 256; // 160
const Y_RES: u32 = 256; // 144

fn main() {
    let fname = match env::args().nth(1) {
        Some(fname) => fname,
        None => {
            println!("USAGE: chip8-sdl ROM-FILE");
            process::exit(1);
        }
    };

    let rom = match fs::read(&fname) {
        Ok(b) => b,
        Err(e) => {
            println!("could not open {}: {}", &fname, e);
            process::exit(1);
        }
    };

    let ctx = sdl2::init().expect("could not init SDL2");
    let video = ctx.video().expect("could not retrieve video subsystem");
    let mut events = ctx.event_pump().expect("could not retrieve event pump");

    let mut canvas = video
        .window("gb-rs", X_RES, Y_RES)
        .position_centered()
        .build()
        .expect("could not create window")
        .into_canvas()
        .build()
        .expect("could not create canvas");

    let tc = canvas.texture_creator();

    let mut surface = tc.create_texture_streaming(None, X_RES, Y_RES).unwrap();
    let mut gb = GameBoy::with_cartridge(&rom[..]);

    'outer: loop {
        while let Some(e) = events.poll_event() {
            if let Event::Quit { .. } = e {
                break 'outer;
            }
        }

        gb.run_to_vblank();

        surface
            .with_lock(Some(Rect::new(0, 0, X_RES, Y_RES)), |mut vbuf, _| {
                gb.rasterize(&mut vbuf);
            })
            .unwrap();

        canvas.copy(&surface, None, None).unwrap();
        canvas.present();
    }
}
