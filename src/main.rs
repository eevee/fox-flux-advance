#![no_std]
#![feature(start)]

extern crate euclid;
extern crate gba;
extern crate num_traits;

pub mod data;
pub mod geom;


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
    palram::index_palram_bg_4bpp,
    vram::{text::TextScreenblockEntry, Tile4bpp, CHAR_BASE_BLOCKS, SCREEN_BASE_BLOCKS},
    Color,
};
use core::fmt::Write;


// NOTE: these are also defined in gba.rs, but the addresses are wrong in 0.3.0
/// BG1 X-Offset. Write only. Text mode only. 9 bits.
pub const BG1HOFS: VolAddress<u16> = unsafe { VolAddress::new_unchecked(0x400_0014) };
/// BG1 Y-Offset. Write only. Text mode only. 9 bits.
pub const BG1VOFS: VolAddress<u16> = unsafe { VolAddress::new_unchecked(0x400_0016) };


use euclid::{point2, rect, size2};

use crate::data::places::TEST_PLACE;
use crate::geom::{Camera, Point, Rect, Size};

#[start]
fn main(_argc: isize, _argv: *const *const u8) -> isize {
    let disp = DisplayControlSetting::new().with_mode(DisplayMode::Mode2).with_oam_memory_1d(true).with_obj(true);
    DISPCNT.write(disp);

    let lexydata = include_bytes!("../lexy.bin");
    let tiledata = include_bytes!("../terrain.bin");

    // TODO oam_init, blanks out OAM

    unsafe {
        while (0x0400_0006 as *mut u16).read_volatile() >= 160 {}
        while (0x0400_0006 as *mut u16).read_volatile() < 160 {}

        // PALETTE
        for p in 0..64 {
            ((0x0500_0000 + p * 2) as *mut u16).write_volatile(lexydata[p * 2] as u16 | (lexydata[p * 2 + 1] as u16) << 8);
            ((0x0500_0200 + p * 2) as *mut u16).write_volatile(lexydata[p * 2] as u16 | (lexydata[p * 2 + 1] as u16) << 8);
        }

        while (0x0400_0006 as *mut u16).read_volatile() >= 160 {}
        while (0x0400_0006 as *mut u16).read_volatile() < 160 {}

        // SPRITE
        for p in 0usize..0x400usize {
            ((0x0200_0000) as *mut u16).write_volatile(p as u16);
            ((0x0200_0002 + p * 2) as *mut u16).write_volatile(lexydata[p * 2 + 0x80] as u16 | (lexydata[p * 2 + 0x80 + 1] as u16) << 8);
            ((0x0601_0000 + p * 2) as *mut u16).write_volatile(lexydata[p * 2 + 0x80] as u16 | (lexydata[p * 2 + 0x80 + 1] as u16) << 8);
        }

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

        BG0HOFS.write(game.camera.position.x as u16);
        BG0VOFS.write(game.camera.position.y as u16);
        BG1HOFS.write(game.camera.position.x as u16);
        BG1VOFS.write(game.camera.position.y as u16);
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
}

impl Entity for Lexy {
    fn update(&mut self, game: &Game) {
        // gravity or whatever
        self.velocity.y += 1;

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
                to_next_tile = 0;
            }
            
            if dy < to_next_tile {
                self.position.y += dy;
            }
            else if dy >= to_next_tile {
                self.position.y += to_next_tile;
                edge += to_next_tile;
                dy -= to_next_tile;
                while dy > 0 {
                    let tid = TEST_PLACE.tiles[(edge / TILE_SIZE) as usize][(self.position.x / TILE_SIZE) as usize];
                    if TEST_PLACE.tileset.tiles[tid as usize].solid {
                        self.velocity.y = 0;
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
                    let tid = TEST_PLACE.tiles[(edge / TILE_SIZE) as usize][(self.position.x / TILE_SIZE) as usize];
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
            (0x0700_0000 as *mut u16).write_volatile((sy + 256) as u16 & 0xffu16 | 0x2000u16 | 0x8000u16);
            (0x0700_0002 as *mut u16).write_volatile((sx + 512) as u16 & 0x1ffu16 | 0xc000u16 | if self.facing_left { 0x1000u16 } else { 0u16 });
            (0x0700_0004 as *mut u16).write_volatile(0u16 | 0x0400u16);
        }
    }
}

fn step(game: &mut Game, lexy: &mut Lexy) {
    let input = read_key_input();

    if input.left() {
        lexy.velocity.x = -3;
        lexy.facing_left = true;
    }
    else if input.right() {
        lexy.velocity.x = 3;
        lexy.facing_left = false;
    }
    else {
        lexy.velocity.x = 0;
    }

    if input.up() {
        if lexy.velocity.y == 0 {
            lexy.velocity.y -= 16;
        }
    }
    if input.down() {
        //lexy.velocity.y = 1;
    }
}
