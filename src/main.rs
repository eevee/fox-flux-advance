#![no_std]
#![feature(start)]

extern crate euclid;
extern crate gba;
extern crate num_traits;

mod data;
mod fixed;
mod geom;


#[panic_handler]
fn panic(panic_info: &core::panic::PanicInfo) -> ! {
    use gba::mgba::{MGBADebug, MGBADebugLevel};
    use core::fmt::Write;
    if let Some(mut debug) = MGBADebug::new() {
        write!(debug, "panic: {}", panic_info).unwrap();
        debug.send(MGBADebugLevel::Error);
    }

    loop {}
}


use gba::{
    base::volatile::VolAddress,
    io::{
        background::{BackgroundControlSetting, BG0CNT, BG1CNT, BG0HOFS, BG0VOFS},
        display::{DISPCNT, DisplayControlSetting, DisplayMode, spin_until_vblank, spin_until_vdraw},
        keypad::{read_key_input},
    },
    oam::{write_obj_attributes},
    palram::{index_palram_bg_8bpp, index_palram_obj_8bpp},
    vram::{text::TextScreenblockEntry, Tile4bpp, CHAR_BASE_BLOCKS, SCREEN_BASE_BLOCKS},
    Color,
};
use core::fmt::Write;


// NOTE: these are also defined in gba.rs, but the addresses are wrong in 0.3.0
/// BG1 X-Offset. Write only. Text mode only. 9 bits.
pub const BG1HOFS: VolAddress<u16> = unsafe { VolAddress::new_unchecked(0x400_0014) };
/// BG1 Y-Offset. Write only. Text mode only. 9 bits.
pub const BG1VOFS: VolAddress<u16> = unsafe { VolAddress::new_unchecked(0x400_0016) };


use crate::data::PALETTE;
use crate::data::places::TEST_PLACE;
use crate::geom::{Camera, Point, Rect, Size, point2, rect, size2};

#[start]
fn main(_argc: isize, _argv: *const *const u8) -> isize {
    let disp = DisplayControlSetting::new().with_mode(DisplayMode::Mode2).with_oam_memory_1d(true).with_obj(true);
    DISPCNT.write(disp);

    let tiledata = include_bytes!("../target/assets/terrain.bin");

    // TODO oam_init, blanks out OAM

    // Set up palette
    for (i, &color) in PALETTE.iter().enumerate() {
        index_palram_bg_8bpp(i as u8).write(color);
        index_palram_obj_8bpp(i as u8).write(color);
    }

    unsafe {
        while (0x0400_0006 as *mut u16).read_volatile() >= 160 {}
        while (0x0400_0006 as *mut u16).read_volatile() < 160 {}

        while (0x0400_0006 as *mut u16).read_volatile() >= 160 {}
        while (0x0400_0006 as *mut u16).read_volatile() < 160 {}

        update_lexy_sprite(0);

        //write_obj_attributes(...)
        (0x0700_0000 as *mut u16).write_volatile(64u16 | 0x2000u16 | 0x8000u16);
        (0x0700_0002 as *mut u16).write_volatile(32u16 | 0xc000u16);
        (0x0700_0004 as *mut u16).write_volatile(0u16 | 0x0400u16);
        /*
        (0x0700_0008 as *mut u16).write_volatile(32u16 | 0x40u16);
        (0x0700_000a as *mut u16).write_volatile(32u16);
        (0x0700_000c as *mut u16).write_volatile(4u16);
        */

        for p in 0..(3 * 16 * 8 * 8 / 2) {
            ((0x0600_0100 + p * 2) as *mut u16).write_volatile(tiledata[p * 2] as u16 | (tiledata[p * 2 + 1] as u16) << 8);
        }
    }

    let mut p = unsafe { SCREEN_BASE_BLOCKS.index(8).cast::<TextScreenblockEntry>() };
    for row in &TEST_PLACE.tiles {
        for &tid in row {
            let tse = TextScreenblockEntry::from_tile_id(tid as u16);
            unsafe {
                p.write(tse);
                p = p.offset(1);
            }
        }
    }

    // bg0 control
    BG0CNT.write(BackgroundControlSetting::new().with_screen_base_block(8).with_bg_priority(1).with_is_8bpp(true));
    BG1CNT.write(BackgroundControlSetting::new().with_screen_base_block(16).with_bg_priority(0).with_is_8bpp(true));
    // Display Control
    DISPCNT.write(DisplayControlSetting::new().with_bg0(true).with_bg1(true).with_obj(true).with_oam_memory_1d(true));

    let mut game = Game{ camera: Camera::new() };
    game.camera.bounds = crate::geom::Bounds::BBox(rect(0, 0, 1024, 1024));
    game.camera.size = size2(240, 160);
    game.camera.margin = size2(64, 32);
    let mut lexy = Lexy{
        position: point2(48, 80),
        velocity: point2(0, 2),
        anchor: point2(17, 47),
        bbox: rect(-6, -26, 12, 27),
        facing_left: false,
        sprite_index: 0,
        sprite_timer: 0,
    };
    loop {
        spin_until_vdraw();
        spin_until_vblank();
        step(&mut game, &mut lexy);
        lexy.update(&game);

        // UPDATE CAMERA
        // TODO maybe aim at lexy's eyes or something, atm she can get closer to the top of the
        // screen than the bottom
        game.camera.aim_at(lexy.position);

        let cam_x = game.camera.position.x.to_int_round() as u16;
        let cam_y = game.camera.position.y.to_int_round() as u16;
        BG0HOFS.write(cam_x);
        BG0VOFS.write(cam_y);
        BG1HOFS.write(cam_x);
        BG1VOFS.write(cam_y);
    }
}

static LEXY_SPRITES: [u8; 49152] = *include_bytes!("../target/assets/lexy.bin");
fn update_lexy_sprite(sprite_index: usize) {
    // SPRITE
    let offset = sprite_index * 0x400;
    unsafe {
        let mut ptr = 0x0601_0000 as *mut u16;
        for p in offset .. (offset + 0x400) {
            ptr.write_volatile(LEXY_SPRITES[p * 2] as u16 | (LEXY_SPRITES[p * 2 + 1] as u16) << 8);
            ptr = ptr.offset(1);
        }
    }
}

macro_rules! spew (
    () => {};
    ($($arg:tt)*) => ({
        use gba::mgba::{MGBADebug, MGBADebugLevel};
        use core::fmt::Write;
        if let Some(mut debug) = MGBADebug::new() {
            write!(debug, $($arg)*).unwrap();
            debug.send(MGBADebugLevel::Debug);
        }
    });
);

struct Game {
    camera: Camera,
}

trait Entity {
    fn update(&mut self, game: &Game);
}

struct Lexy {
    position: Point,
    velocity: Point,
    anchor: Point,
    bbox: Rect,
    facing_left: bool,
    sprite_index: usize,
    sprite_timer: usize,
}

impl Entity for Lexy {
    fn update(&mut self, game: &Game) {
        // gravity or whatever
        self.velocity.y += 1;

        let old_sprite_index = self.sprite_index;
        if self.velocity.x == 0 {
            self.sprite_index = 0;
            self.sprite_timer = 0;
        }
        else {
            if self.sprite_timer == 0 {
                self.sprite_index += 1;
                if self.sprite_index > 8 {
                    self.sprite_index = 1;
                }
                self.sprite_timer = 4;
            }
            else {
                self.sprite_timer -= 1;
            }
        }
        if self.sprite_index != old_sprite_index {
            update_lexy_sprite(self.sprite_index);
        }

        let dx = self.velocity.x;
        let mut dy = self.velocity.y;

        // poor man's collision detection
        const TILE_SIZE: i16 = 8;
        // x
        self.position.x += dx;
        // y
        if dy > 0 {
            let mut edge = self.position.y + self.bbox.max_y();
            let mut to_next_tile = TILE_SIZE - edge % TILE_SIZE;
            if to_next_tile == TILE_SIZE {
                to_next_tile = 0.into();
            }
            
            if dy < to_next_tile {
                self.position.y += dy;
            }
            else if dy >= to_next_tile {
                self.position.y += to_next_tile;
                edge += to_next_tile;
                dy -= to_next_tile;
                while dy > 0 {
                    let tid = TEST_PLACE.tiles[edge.to_tile_coord()][self.position.x.to_tile_coord()];
                    if TEST_PLACE.tileset.tiles[tid as usize].solid {
                        self.velocity.y = 0.into();
                        break;
                    }
                    if dy >= TILE_SIZE {
                        dy -= TILE_SIZE;
                        self.position.y += TILE_SIZE;
                        edge += TILE_SIZE;
                    }
                    else {
                        self.position.y += dy;
                        break;
                    }
                }
            }
        }
        else if dy < 0 {
            let mut edge = self.position.y + self.bbox.min_y();
            let to_next_tile = edge % TILE_SIZE;
            dy = -dy;
            if dy < to_next_tile {
                self.position.y -= dy;
            }
            else if dy >= to_next_tile {
                self.position.y -= to_next_tile;
                edge -= to_next_tile;
                dy -= to_next_tile;
                while dy > 0 {
                    let tid = TEST_PLACE.tiles[edge.to_tile_coord()][self.position.x.to_tile_coord()];
                    if TEST_PLACE.tileset.tiles[tid as usize].solid {
                        break;
                    }
                    if dy >= TILE_SIZE {
                        dy -= TILE_SIZE;
                        self.position.y -= TILE_SIZE;
                    }
                    else {
                        self.position.y -= dy;
                        break;
                    }
                }
            }
        }

        // update position i guess?  assumes slot 0!
        let sx = self.position.x - (if self.facing_left { 32 - self.anchor.x } else { self.anchor.x }) - game.camera.position.x;
        let sy = self.position.y - self.anchor.y - game.camera.position.y;
        unsafe {
            (0x0700_0000 as *mut u16).write_volatile(sy.to_sprite_offset_y() | 0x2000u16 | 0x8000u16);
            (0x0700_0002 as *mut u16).write_volatile(sx.to_sprite_offset_x() | 0xc000u16 | if self.facing_left { 0x1000u16 } else { 0u16 });
            (0x0700_0004 as *mut u16).write_volatile(0u16 | 0x0400u16);
        }
    }
}

fn step(game: &mut Game, lexy: &mut Lexy) {
    let input = read_key_input();

    if input.left() {
        lexy.velocity.x = (-3).into();
        lexy.facing_left = true;
    }
    else if input.right() {
        lexy.velocity.x = 3.into();
        lexy.facing_left = false;
    }
    else {
        lexy.velocity.x = 0.into();
    }
    lexy.velocity.x /= 2;

    if input.up() {
        if lexy.velocity.y == 0 {
            lexy.velocity.y -= 16;
        }
    }
    if input.down() {
        //lexy.velocity.y = 1;
    }
}
