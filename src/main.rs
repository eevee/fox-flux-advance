#![no_std]
#![feature(start)]
#![feature(lang_items)]

extern crate gba;

#[panic_handler]
fn panic(_info: &core::panic::PanicInfo) -> ! {
  loop {}
}

#[lang = "eh_personality"] fn eh_personality() {}


use core::intrinsics::transmute;
use gba::io::display::{DisplayMode};
use gba::{
    base::volatile::VolAddress,
    io::{
        background::{BackgroundControlSetting, BG0CNT, BG1CNT, BG0HOFS, BG0VOFS},
        display::{DisplayControlSetting, DISPCNT, spin_until_vblank, spin_until_vdraw},
        keypad::{read_key_input},
    },
    palram::index_palram_bg_4bpp,
    vram::{text::TextScreenblockEntry, Tile4bpp, CHAR_BASE_BLOCKS, SCREEN_BASE_BLOCKS},
    Color,
};

// NOTE: these are also defined in gba.rs, but the addresses are wrong in 0.3.0
/// BG1 X-Offset. Write only. Text mode only. 9 bits.
pub const BG1HOFS: VolAddress<u16> = unsafe { VolAddress::new_unchecked(0x400_0014) };
/// BG1 Y-Offset. Write only. Text mode only. 9 bits.
pub const BG1VOFS: VolAddress<u16> = unsafe { VolAddress::new_unchecked(0x400_0016) };



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

  pub const WHITE: Color = Color::from_rgb(31, 31, 31);
  pub const LIGHT_GRAY: Color = Color::from_rgb(25, 25, 25);
  pub const DARK_GRAY: Color = Color::from_rgb(15, 15, 15);
  // bg palette
  /*
  index_palram_bg_4bpp(0, 1).write(WHITE);
  index_palram_bg_4bpp(0, 2).write(LIGHT_GRAY);
  index_palram_bg_4bpp(0, 3).write(DARK_GRAY);
  */
  // bg tiles
  //set_bg_tile_4bpp(0, 0, ALL_TWOS);
  //set_bg_tile_4bpp(0, 1, ALL_THREES);
  // screenblock
  let light_entry = TextScreenblockEntry::from_tile_id(0);
  let dark_entry = TextScreenblockEntry::from_tile_id(1);

  //checker_screenblock(8, light_entry, dark_entry);
  let mut p = unsafe { SCREEN_BASE_BLOCKS.index(8).cast::<TextScreenblockEntry>() };
  unsafe { p = p.offset(32 * (20 - 8)); }
  for row in 0..4 {
    let a = TextScreenblockEntry::from_tile_id(row * 4 + 20 + 0);
    let b = TextScreenblockEntry::from_tile_id(row * 4 + 20 + 1);
    let c = TextScreenblockEntry::from_tile_id(row * 4 + 20 + 2);
    let d = TextScreenblockEntry::from_tile_id(row * 4 + 20 + 3);
    for _col in 0..32/4 {
      unsafe {
        p.write(a);
        p = p.offset(1);
        p.write(b);
        p = p.offset(1);
        p.write(c);
        p = p.offset(1);
        p.write(d);
        p = p.offset(1);
      }
    }
  }
  for row in 0..4 {
    let a = TextScreenblockEntry::from_tile_id(row * 4 + 4 + 0);
    let b = TextScreenblockEntry::from_tile_id(row * 4 + 4 + 1);
    let c = TextScreenblockEntry::from_tile_id(row * 4 + 4 + 2);
    let d = TextScreenblockEntry::from_tile_id(row * 4 + 4 + 3);
    for _col in 0..32/4 {
      unsafe {
        p.write(a);
        p = p.offset(1);
        p.write(b);
        p = p.offset(1);
        p.write(c);
        p = p.offset(1);
        p.write(d);
        p = p.offset(1);
      }
    }
  }


  let mut p = unsafe { SCREEN_BASE_BLOCKS.index(16).cast::<TextScreenblockEntry>() };
  unsafe { p = p.offset(32 * (20 - 8)); }
  for row in 0..4 {
    let a = TextScreenblockEntry::from_tile_id(row * 4 + 36 + 0);
    let b = TextScreenblockEntry::from_tile_id(row * 4 + 36 + 1);
    let c = TextScreenblockEntry::from_tile_id(row * 4 + 36 + 2);
    let d = TextScreenblockEntry::from_tile_id(row * 4 + 36 + 3);
    for _col in 0..32/4 {
      unsafe {
        p.write(a);
        p = p.offset(1);
        p.write(b);
        p = p.offset(1);
        p.write(c);
        p = p.offset(1);
        p.write(d);
        p = p.offset(1);
      }
    }
  }

  // bg0 control
  BG0CNT.write(BackgroundControlSetting::new().with_screen_base_block(8).with_bg_priority(1).with_is_8bpp(true));
  BG1CNT.write(BackgroundControlSetting::new().with_screen_base_block(16).with_bg_priority(0).with_is_8bpp(true));
  // Display Control
DISPCNT.write(DisplayControlSetting::new().with_bg0(true).with_bg1(true).with_obj(true).with_oam_memory_1d(true));


    let mut game = Game{ camera_x: 0, camera_y: 0 };
    loop {
        spin_until_vdraw();
        spin_until_vblank();
        step(&mut game);
    }
}

struct Game {
    camera_x: u16,
    camera_y: u16,
}

fn step(game: &mut Game) {
    let input = read_key_input();

    if input.left() {
        if game.camera_x == 0 {
            game.camera_x = 255;
        }
        else {
            game.camera_x -= 1;
        }
    }
    if input.right() {
        if game.camera_x >= 255 {
            game.camera_x = 0;
        }
        else {
            game.camera_x += 1;
        }
    }
    if input.up() {
        if game.camera_y == 0 {
            game.camera_y = 255;
        }
        else {
            game.camera_y -= 1;
        }
    }
    if input.down() {
        if game.camera_y >= 255 {
            game.camera_y = 0;
        }
        else {
            game.camera_y += 1;
        }
    }

    BG0HOFS.write(game.camera_x);
    BG0VOFS.write(game.camera_y);
    BG1HOFS.write(game.camera_x);
    BG1VOFS.write(game.camera_y);
}

pub const ALL_TWOS: Tile4bpp = Tile4bpp([
  0x22222222, 0x22222222, 0x22222222, 0x22222222, 0x22222222, 0x22222222, 0x22222222, 0x22222222,
]);

pub const ALL_THREES: Tile4bpp = Tile4bpp([
  0x33333333, 0x33333333, 0x33333333, 0x33333333, 0x33333333, 0x33333333, 0x33333333, 0x33333333,
]);
pub fn set_bg_tile_4bpp(charblock: usize, index: usize, tile: Tile4bpp) {
  assert!(charblock < 4);
  assert!(index < 512);
  unsafe { CHAR_BASE_BLOCKS.index(charblock).cast::<Tile4bpp>().offset(index as isize).write(tile) }
}

pub fn checker_screenblock(slot: usize, a_entry: TextScreenblockEntry, b_entry: TextScreenblockEntry) {
  let mut p = unsafe { SCREEN_BASE_BLOCKS.index(slot).cast::<TextScreenblockEntry>() };
  let mut checker = true;
  for _row in 0..32 {
    for _col in 0..32 {
      unsafe {
        p.write(if checker { a_entry } else { b_entry });
        p = p.offset(1);
      }
      checker = !checker;
    }
    checker = !checker;
  }
}
