// Games made using `agb` are no_std which means you don't have access to the standard
// rust library. This is because the game boy advance doesn't really have an operating
// system, so most of the content of the standard library doesn't apply.
//
// Provided you haven't disabled it, agb does provide an allocator, so it is possible
// to use both the `core` and the `alloc` built in crates.
#![no_std]
// `agb` defines its own `main` function, so you must declare your game's main function
// using the #[agb::entry] proc macro. Failing to do so will cause failure in linking
// which won't be a particularly clear error message.
#![no_main]
#![cfg_attr(test, feature(custom_test_frameworks))]
#![cfg_attr(test, reexport_test_harness_main = "test_main")]
#![cfg_attr(test, test_runner(agb::test_runner::test_runner))]

mod rng;
mod gamemode;
mod gfx;

use agb::{display, syscall};
use agb::display::Priority::P0;
use agb::display::tiled::{TileSetting, VRamManager, MapLoan, TileSet, RegularMap};
use agb::interrupt::Interrupt::VBlank;
use agb::input::{Button, ButtonController, Tri};
use alloc::vec::Vec;
use agb::fixnum::Vector2D;
use gamemode::TileType::{FLOOR, DOOR_CLOSED};
use agb::display::object::{ObjectController, SpriteBorrow};
use agb::input::Tri::{Positive, Negative};
use agb::mgba::Mgba;
use core::borrow::BorrowMut;
use bitfield::bitfield;
use crate::gfx::load_splashtiles;
use agb::display::Priority;
use core::panic::PanicInfo;
use alloc::string::ToString;
use core::fmt::Write;

extern crate alloc;
#[macro_use]
extern crate enum_ordinalize;

// The main function must take 1 arguments and never return. The agb::entry decorator
// ensures that everything is in order. `agb` will call this after setting up the stack
// and interrupt handlers correctly. It will also handle creating the `Gba` struct for you.
#[agb::entry]
fn main(mut gba: agb::Gba) -> ! {
    // video_test(&mut vram, &mut bg, &tileset, palette_ids)

    title_screen(&mut gba);

    gamemode::show_floor0(&mut gba);
}

bitfield! {
    struct RGB5(u16);
    u16;
    get_r, set_r: 4,0;
    get_g, set_g: 9,5;
    get_b, set_b: 14,10;
}

trait FromColor<T> {
    fn from_rgb(r: u16, g: u16, b: u16) -> T;
}

impl FromColor<RGB5> for RGB5 {
    fn from_rgb(r: u16, g: u16, b: u16) -> RGB5 {
        let mut rgb = RGB5(0);
        rgb.set_r(r);
        rgb.set_g(g);
        rgb.set_b(b);
        rgb
    }
}

// #[panic_handler]
// fn panic(info: &PanicInfo) -> ! {
//     agb::mgba::Mgba::new().unwrap().write_str(info.to_string().as_str());
//     loop{}
// }

fn title_screen(gba: &mut agb::Gba) {
    let (tileset, palette_ids, palettes) = load_splashtiles();
    let (tiled, mut vram) = gba.display.video.tiled0();
    vram.set_background_palettes(palettes);
    let mut bg = tiled.background(Priority::P3);
    let mut fg = tiled.background(Priority::P2);
    let mut sel = tiled.background(Priority::P1);
    for y in 0..20 {
        for x in 0..30 {
            let tile_id = y * 30 + x;
            fg.set_tile(
                &mut vram, (x, y).into(),
                &tileset,
                TileSetting::new(tile_id, false, false, palette_ids[(tile_id) as usize]));
        }
    }
    // start of third tiled layer
    for y in 0u16..20u16 {
        for x in 0u16..30u16 {
            sel.set_tile(
                &mut vram, (x,y).into(),
                &tileset,
                TileSetting::new(30*20+1, false, false, palette_ids[30*20+1])
            );
        }
    }
    for y in 0..12 {
        for x in 0..6 {
            let tile_id = 30 * (21 + y) + x;
            sel.set_tile(
                &mut vram, (x, y).into(),
                &tileset,
                TileSetting::new(tile_id, false, false, palette_ids[tile_id as usize])
            );
        }
    }
    //end of third tiled layer
    for y in 0..32 {
        for x in 0..32 {
            let tile_id = (30 * 20) + ((y + x) % 2);
            bg.set_tile(
                &mut vram, (x, y).into(),
                &tileset,
                TileSetting::new(tile_id, false, false, palette_ids[(tile_id) as usize]));
        }
    }
    let mut index = 0i16;
    let blank = agb::interrupt::VBlank::get();
    bg.commit();
    fg.commit();
    sel.commit();
    let mut input = agb::input::ButtonController::new();
    loop {
        input.update();
        if input.is_pressed(Button::START) {
            break;
        }
        index += 1;
        bg.set_scroll_pos((0, (index.abs()) as u16).into());
        bg.commit();
        bg.show();
        fg.show();
        sel.show();
        rng::get_random();
        blank.wait_for_vblank();
        rng::get_random();
        blank.wait_for_vblank();
        rng::get_random();
        blank.wait_for_vblank();
        rng::get_random();
        blank.wait_for_vblank();
    }
}

