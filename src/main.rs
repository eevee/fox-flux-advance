#![no_std]
#![feature(start)]
#![feature(lang_items)]

extern crate gba;

#[panic_handler]
fn panic(_info: &core::panic::PanicInfo) -> ! {
  loop {}
}

#[lang = "eh_personality"] fn eh_personality() {}


use gba::io::display::{DisplayControlSetting, DisplayMode, DISPCNT};

#[start]
fn main(_argc: isize, _argv: *const *const u8) -> isize {
    let disp = DisplayControlSetting::new().with_mode(DisplayMode::Mode2).with_oam_memory_1d(true).with_obj(true);
    DISPCNT.write(disp);

    let lexydata = include_bytes!("../lexy.bin");

    // TODO oam_init, blanks out OAM

    unsafe {
        while (0x0400_0006 as *mut u16).read_volatile() >= 160 {}
        while (0x0400_0006 as *mut u16).read_volatile() < 160 {}

        // PALETTE
        for p in 0..64 {
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

        (0x0700_0000 as *mut u16).write_volatile(32u16 | 0x2000u16 | 0x8000u16);
        (0x0700_0002 as *mut u16).write_volatile(32u16 | 0xc000u16);
        (0x0700_0004 as *mut u16).write_volatile(0u16);
        /*
        (0x0700_0008 as *mut u16).write_volatile(32u16 | 0x40u16);
        (0x0700_000a as *mut u16).write_volatile(32u16);
        (0x0700_000c as *mut u16).write_volatile(4u16);
        */
        loop {}
    }
}
