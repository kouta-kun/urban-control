use agb::display::tiled::{VRamManager, RegularMap, TileSet, TileSetting, MapLoan, TileFormat};
use agb::display::object::{Sprite, Size, ObjectController, SpriteBorrow};

agb::include_gfx!("gfx/tiles.toml");
agb::include_gfx!("gfx/sprites.toml");
agb::include_gfx!("gfx/splashscrn.toml");

pub fn load_bgtiles() -> (TileSet<'static>, &'static [u8], &'static [agb::display::palette16::Palette16]) {
    (TileSet::new(tiles::background.tiles, TileFormat::FourBpp), tiles::background.palette_assignments, tiles::background.palettes)
}

pub fn load_splashtiles() -> (TileSet<'static>, &'static [u8], &'static [agb::display::palette16::Palette16]) {
    (TileSet::new(splashscrn::background.tiles, TileFormat::FourBpp), splashscrn::background.palette_assignments, splashscrn::background.palettes)
}

const player: Sprite = Sprite::new(&sprites::background.palettes[0], sprites::background.tiles, Size::S8x8);

pub fn load_sptiles(object_controller: &ObjectController) -> SpriteBorrow {
    object_controller.sprite(&player)
}

fn video_test(mut vram: &mut VRamManager, bg: &mut MapLoan<RegularMap>, tileset: &TileSet, palette_ids: &[u8]) -> ! {
    for y in 0..32 {
        for x in 0..32 {
            let tile_id = (y % 2) * 3;
            let tile_setting = TileSetting::new(
                tile_id,
                false, false,
                palette_ids[tile_id as usize],
            );
            bg.set_tile(&mut vram, (x as u16, y as u16).into(),
                        &tileset,
                        tile_setting,
            );
        }
    }

    let vblank = agb::interrupt::VBlank::get();
    bg.set_scroll_pos((0 as u16, 0 as u16).into());

    loop {
        bg.commit();
        bg.show();

        vblank.wait_for_vblank();
        vblank.wait_for_vblank();
        vblank.wait_for_vblank();
        vblank.wait_for_vblank();
    }
}

