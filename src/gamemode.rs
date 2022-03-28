use agb::display::tiled::{VRamManager, RegularMap, TileSet, TileSetting, MapLoan};
use agb::display::Priority::P0;
use agb::input::Button;
use crate::rng;
use agb::fixnum::{Vector2D, FixedWidthUnsignedInteger};
use alloc::vec::Vec;
use core::borrow::BorrowMut;
use crate::gamemode::TileType::{FLOOR, DOOR_CLOSED};
use crate::gfx;
use alloc::boxed::Box;

const FLOOR_WIDTH: usize = 32;
const FLOOR_HEIGHT: usize = 20;

#[derive(Ordinalize, Copy, Clone, PartialOrd, PartialEq)]
pub enum TileType {
    WALL,
    DOOR_CLOSED,
    FLOOR,
    EMPTY,
}

pub struct PlayerState {
    pos: Vector2D<u16>
}

pub struct PosU8 { x: u8, y: u8 }

pub struct Floor {
    size: PosU8,
    tiles: Vec<TileType>,
}

struct GameState {
    floor: Floor,
    player: PlayerState,
}

impl GameState {
    pub fn get_tile(&self, x: usize, y: usize) -> TileType {
        self.floor.tiles[y * self.floor.size.x as usize + x]
    }
}

impl From<&[u8]> for PosU8 {
    fn from(l: &[u8]) -> Self {
        if l.len() != 2 {
            panic!("Expected vector of 2, got {}", l.len());
        }
        PosU8 {
            y: l[0],
            x: l[1],
        }
    }
}

mod floors {
    use crate::gamemode::{TileType, Floor};
    use alloc::vec::Vec;
    use alloc::boxed::Box;

    pub(crate) const F0: &[u8] = include_bytes!("../floors/floor_0.bff");
    pub(crate) const F1: &[u8] = include_bytes!("../floors/floor_1.bff");
    pub(crate) const F2: &[u8] = include_bytes!("../floors/floor_2.bff");
    pub(crate) const F3: &[u8] = include_bytes!("../floors/floor_3.bff");

    pub(crate) const FLOORS: [&[u8];4] = [F0, F1, F2, F3];


    pub fn tile_to_enum(floortile: &u8) -> TileType {
        match floortile {
            1 => TileType::FLOOR,
            2 => TileType::WALL,
            7 => TileType::EMPTY,
            3 => TileType::DOOR_CLOSED,
            _ => TileType::EMPTY
        }
    }

    pub fn parse_floor(raw: &[u8]) -> Floor {
        Floor {
            size: raw[0..2].into(),
            tiles: raw[2..].iter().map(tile_to_enum).collect(),
        }
    }
}

pub fn show_floor0(gba: &mut agb::Gba) -> ! {
    let (tiled, mut vram) = gba.display.video.tiled0();

    let mut bg = tiled.background(P0);
    let mut obj_controller = gba.display.object.get();

    let (tileset, palette_ids, palette_data) = gfx::load_bgtiles();
    vram.set_background_palettes(palette_data);

    let mut input = agb::input::ButtonController::new();

    let mut gamestate = Box::new(GameState {
        floor: floors::parse_floor(floors::F0),
        player: PlayerState { pos: (0u16, 0u16).into() },
    });
    let initpos = select_random_floor(&gamestate);
    gamestate.player.pos = initpos;

    let vblank = agb::interrupt::VBlank::get();

    let sprite_borrow = gfx::load_sptiles(&obj_controller);

    let mut player = obj_controller.object(sprite_borrow);


    let mut left_x = ((gamestate.player.pos.x as i32 - 15i32).min(gamestate.floor.size.x as i32 - 30).max(0i32)) as u16 * 8u16;
    load_initial_floor_view(&mut vram, bg.borrow_mut(), &tileset, palette_ids, &gamestate.floor, left_x);

    loop {
        input.update();

        if input.is_just_pressed(Button::UP) {
            let tt = gamestate.get_tile(gamestate.player.pos.x as usize, (gamestate.player.pos.y - 1) as usize);
            if tt == FLOOR || tt == DOOR_CLOSED {
                gamestate.player.pos.y -= 1;
            }
        } else if input.is_just_pressed(Button::DOWN) {
            let tt = gamestate.get_tile(gamestate.player.pos.x as usize, (gamestate.player.pos.y + 1) as usize);
            if tt == FLOOR || tt == DOOR_CLOSED {
                gamestate.player.pos.y += 1;
            }
        } else if input.is_just_pressed(Button::RIGHT) {
            let tt = gamestate.get_tile((gamestate.player.pos.x + 1) as usize, (gamestate.player.pos.y) as usize);
            if tt == FLOOR || tt == DOOR_CLOSED {
                gamestate.player.pos.x += 1;
            }
        } else if input.is_just_pressed(Button::LEFT) {
            let tt = gamestate.get_tile((gamestate.player.pos.x - 1) as usize, (gamestate.player.pos.y) as usize);
            if tt == FLOOR || tt == DOOR_CLOSED {
                gamestate.player.pos.x -= 1;
            }
        }

        let target_left_x = ((gamestate.player.pos.x as i32 - 15i32).min(gamestate.floor.size.x as i32 - 30).max(0i32)) as u16 * 8u16;

        if target_left_x != left_x {
            let delta = (target_left_x as i32 - left_x as i32).signum();
            let left_tile = (left_x as i32 / 8i32);
            if delta < 0 && left_tile > 0 {
                let row_to_write = ((left_tile as i32 - 1) % 32).abs() as usize;
                let row_to_read = (left_tile as i32 - 1) as usize;
                overwrite_column(&mut vram, &mut bg, &tileset, palette_ids, row_to_write, row_to_read, &gamestate.floor)
            }
            while target_left_x != left_x {
                let midframe_offset = (target_left_x as i32 - left_x as i32).signum();
                left_x = (left_x as i32 + midframe_offset).clamp(0i32, u16::MAX as i32) as u16;

                bg.set_scroll_pos((left_x, 0 as u16).into());

                bg.commit();
                bg.show();

                let scrpos_x = (gamestate.player.pos.x * 8u16 - left_x);

                player.set_position((scrpos_x, gamestate.player.pos.y * 8u16).into());

                player.commit();
                player.show();

                vblank.wait_for_vblank();
                vblank.wait_for_vblank();
            }
            if delta > 0 {
                let row_to_write = ((left_tile as i32 + 31 ) % 32).abs().clamp(0, 31) as usize;
                let row_to_read = (left_tile as i32 + 31) as usize;
                overwrite_column(&mut vram, &mut bg, &tileset, palette_ids, row_to_write, row_to_read, &gamestate.floor)
            }
        }

        bg.set_scroll_pos((target_left_x, 0 as u16).into());
        bg.commit();
        bg.show();

        let scrpos_x = (gamestate.player.pos.x * 8u16 - left_x);

        player.set_position((scrpos_x, gamestate.player.pos.y * 8u16).into());

        player.commit();
        player.show();

        vblank.wait_for_vblank();
        vblank.wait_for_vblank();
        vblank.wait_for_vblank();
        vblank.wait_for_vblank();
    }
}

fn overwrite_column(vram: &mut VRamManager, bg: &mut MapLoan<RegularMap>,
                    tileset: &TileSet, palette_ids: &[u8],
                    row_to_write: usize, row_to_read: usize,
                    floor: &Floor) {
    for y in 0usize..20usize {
        let tile_setting = get_tilesetting_for_tile(palette_ids, floor, row_to_read, y);
        bg.set_tile(vram, (row_to_write as u16, y as u16).into(),
                    &tileset,
                    tile_setting,
        );
    }
}

fn get_tilesetting_for_tile(palette_ids: &[u8], floor: &Floor, x: usize, y: usize) -> TileSetting {
    let tile_id =
        (if y < floor.size.y as usize {
            floor.tiles[y * floor.size.x as usize + x]
        } else {
            TileType::EMPTY
        }).ordinal() as u16;
    let tile_setting = TileSetting::new(
        tile_id,
        false, false,
        palette_ids[tile_id as usize],
    );
    tile_setting
}

fn select_random_floor(gamestate: &GameState) -> Vector2D<u16> {
    let mut x: u16 = 0u16;
    let mut y: u16 = 0u16;

    loop {
        x = (rng::get_random().abs() as usize % FLOOR_WIDTH) as u16;
        y = (rng::get_random().abs() as usize % FLOOR_HEIGHT) as u16;
        if gamestate.get_tile(x as usize, y as usize) == FLOOR {
            break;
        }
    }

    return (x, y).into();
}


fn load_initial_floor_view(vram: &mut VRamManager, bg: &mut MapLoan<RegularMap>, tileset: &TileSet, palette_ids: &[u8], floor: &Floor, left_x: u16) {
    let l = (left_x / 8u16) as usize;
    let ymax = (FLOOR_HEIGHT).min(floor.size.y as usize);
    let ymin = 0;
    let xmax = (l+FLOOR_WIDTH).min(floor.size.x as usize);
    let xmin = l;
    for y in ymin..ymax {
        for x in xmin..xmax {
            let tile_id =
                (if y < ymax {
                    floor.tiles[y * floor.size.x as usize + x]
                } else {
                    TileType::EMPTY
                }).ordinal() as u16;
            let tile_setting = TileSetting::new(
                tile_id,
                false, false,
                palette_ids[tile_id as usize],
            );
            bg.set_tile(vram, ((x - l) as u16, y as u16).into(),
                        &tileset,
                        tile_setting,
            );
        }
    }
}

