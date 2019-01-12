#![no_std]
#![feature(start)]

extern crate arrayvec;
extern crate euclid;
extern crate gba;
extern crate num_traits;

mod data;
mod debug;
mod fixed;
mod geom;
mod whammo;


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
        timers::{TimerControlSetting, TimerTickRate, TM0CNT_L, TM0CNT_H},
    },
    oam::{write_obj_attributes},
    palram::{index_palram_bg_8bpp, index_palram_obj_8bpp},
    vram::{text::TextScreenblockEntry, Tile4bpp, CHAR_BASE_BLOCKS, SCREEN_BASE_BLOCKS},
    Color,
};


// NOTE: these are also defined in gba.rs, but the addresses are wrong in 0.3.0
/// BG1 X-Offset. Write only. Text mode only. 9 bits.
pub const BG1HOFS: VolAddress<u16> = unsafe { VolAddress::new_unchecked(0x400_0014) };
/// BG1 Y-Offset. Write only. Text mode only. 9 bits.
pub const BG1VOFS: VolAddress<u16> = unsafe { VolAddress::new_unchecked(0x400_0016) };


use crate::data::PALETTE;
use crate::data::places::TEST_PLACE;
use crate::fixed::Fixed;
use crate::geom::{Camera, Point, Rect, Vector, VectorExt, point2, rect, size2, vec2};
use crate::whammo::shapes::{Contact, Polygon};
use crate::whammo::CollisionVec;

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
        velocity: vec2(0, 2),
        anchor: point2(17, 47),
        bbox: rect(-6, -26, 12, 27),
        facing_left: false,
        sprite_index: 0,
        sprite_timer: 0,
    };

    let timer_disabled = TimerControlSetting::new().with_tick_rate(TimerTickRate::CPU64);
    let timer_enabled = timer_disabled.with_enabled(true);

    loop {
        spin_until_vdraw();
        spin_until_vblank();

        // Reset the timer by disabling and enabling it
        TM0CNT_H.write(timer_disabled);
        TM0CNT_H.write(timer_enabled);

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

// TODO enforce that the sprite data is aligned; use aligned crate?  then i guess put a nicer
// wrapper on this
use gba::io::dma;
static LEXY_SPRITES: [u8; 49152] = *include_bytes!("../target/assets/lexy.bin");
fn update_lexy_sprite(sprite_index: usize) {
    // SPRITE
    let offset = sprite_index * 0x400;
    unsafe {
        dma::DMA3::set_source((&LEXY_SPRITES as *const u8).offset(sprite_index as isize * 0x800) as *const u32);
        dma::DMA3::set_dest(0x0601_0000 as *mut u32);
        dma::DMA3::set_count(0x800 / 4);
        dma::DMA3::set_control(
            dma::DMAControlSetting::new()
            .with_use_32bit(true)
            .with_enabled(true)
        );
    }
}

struct Game {
    camera: Camera,
}

trait Entity {
    fn update(&mut self, game: &Game);
    fn nudge(&mut self, displacement: Vector) -> Vector;
    fn collider_sweep(&self, shape: &Polygon, attempted: Vector /*, pass_callback */) -> (Vector, CollisionVec);
}

struct Lexy {
    position: Point,
    velocity: Vector,
    anchor: Point,
    bbox: Rect,
    facing_left: bool,
    sprite_index: usize,
    sprite_timer: usize,
}


fn _is_vector_almost_zero(vec: Vector) -> bool {
    vec.x.abs() * 64 < 1 && vec.y.abs() * 64 < 1
}


enum SlideResult {
    Stuck,
    Slid(Vector),
}
fn slide_along_normals(hits: &CollisionVec, direction: Vector) -> SlideResult {
    let perp = direction.perpendicular();
    let mut minleftdot;
    let mut minleftnorm;
    let mut minrightdot;
    let mut minrightnorm;
    let mut right_possible = true;
    let mut left_possible = true;

    let mut iter = hits.iter();
    if let Some(collision) = iter.next() {
        minleftdot = collision.left_normal_dot;
        minleftnorm = collision.left_normal;
        minrightdot = collision.right_normal_dot;
        minrightnorm = collision.right_normal;
    }
    else {
        // No hits at all (which doesn't make much sense), so we're free to move wherever
        return SlideResult::Slid(direction);
    }

    // So, here's the problem.  At first blush, this seems easy enough: just
    // pick the normal that restricts us the most, which is the one that faces
    // most towards us (i.e. has the most negative dot product), and slide
    // along that.  Alas, there are two major problems there.
    // 1. We might be blocked on /both sides/ and thus can't move at all.  To
    // detect this, we have to sort normals into "left" and "right", find the
    // worst normal on each side, and then reconcile at the end.
    // 2. Each hit might be a corner collision and have multiple normals.
    // While hitting more objects and thus encountering more normals will
    // /reduce/ our available slide area, hitting a corner /increases/ it.  So
    // within a single hit, we have to do the same thing in reverse, finding
    // the BEST normal on each side and counting that one.
    // FIXME there are also two problems with the data we get out of whammo
    // atm: (a) a corner collision might produce more than two normals which
    // feels ambiguous (but maybe it isn't; remember those normals are from
    // both us and the thing we hit?  maybe draw a diagram to check on this),
    // and (b) MultiShape blindly crams all the normals into a single table,
    // even though normals from different shapes combine differently.  for the
    // latter problem, maybe we should just return a left_normal and
    // right_normal in each hit?  i mean we do the dot products in whammo
    // itself already, so that'd save us a lot of effort.  only drawback i can
    // think of is that moving by zero would make all those normals kind of
    // meaningless, but i think we could just look at the overall direction of
    // contact...?  whatever that means?
    for collision in iter {
        if collision.touchtype == Contact::Overlap /* || collision.passable */ {
            continue;
        }

        // TODO comment stuff in shapes.lua
        // TODO update comments here, delete dead code
        // TODO explain why i used <= below (oh no i don't remember, but i think it was related to how this is done against the last slide only)
        // FIXME i'm now using normals compared against our /last slide/ on our /velocity/ and it's unclear what ramifications that could have (especially since it already had enough ramifications to need the <=) -- think about this i guess lol

        if left_possible && collision.left_normal.is_some() {
            if collision.left_normal_dot <= minleftdot {
                minleftdot = collision.left_normal_dot;
                minleftnorm = collision.left_normal;
            }
        }
        else {
            left_possible = false;
            minleftnorm = None;
        }

        if right_possible && collision.right_normal.is_some() {
            if collision.right_normal_dot <= minrightdot {
                minrightdot = collision.right_normal_dot;
                minrightnorm = collision.right_normal;
            }
        }
        else {
            right_possible = false;
            minrightnorm = None;
        }
    }

    if ! left_possible && ! right_possible {
        return SlideResult::Stuck;
    }

    let axis;
    if ! left_possible {
        axis = minrightnorm;
    }
    else if ! right_possible {
        axis = minleftnorm;
    }
    else if minleftdot > minrightdot {
        axis = minleftnorm;
    }
    else {
        axis = minrightnorm;
    }
    // FIXME this makes me realize that this function doesn't use direction at all until here

    if let Some(axis) = axis {
        // This dot product check handles an obscure case: if a collision callback
        // overwrites our velocity so that we're moving /away/ from the object we
        // hit, then there's no need to change it any more.  This happens with the
        // moo form's charge in fox flux.
        if direction.dot(axis) < 0 {
            return SlideResult::Slid(direction - direction.project_on(axis));
        }
        else {
            return SlideResult::Slid(direction);
        }
    }
    else {
        return SlideResult::Stuck;
    }
}



impl Entity for Lexy {
    fn update(&mut self, game: &Game) {
        // gravity or whatever
        self.velocity.y += Fixed::promote(16) / 75;

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

        let movement = self.velocity.clone();
        self.nudge(movement);

        // update position i guess?  assumes slot 0!
        // FIXME this should very much be done in shadow oam and copied at next vblank
        let sx = self.position.x - (if self.facing_left { 32 - self.anchor.x } else { self.anchor.x }) - game.camera.position.x;
        let sy = self.position.y - self.anchor.y - game.camera.position.y;
        unsafe {
            (0x0700_0000 as *mut u16).write_volatile(sy.to_sprite_offset_y() | 0x2000u16 | 0x8000u16);
            (0x0700_0002 as *mut u16).write_volatile(sx.to_sprite_offset_x() | 0xc000u16 | if self.facing_left { 0x1000u16 } else { 0u16 });
            (0x0700_0004 as *mut u16).write_volatile(0u16 | 0x0400u16);
        }
    }

    fn collider_sweep(&self, shape: &Polygon, attempted: Vector /*, pass_callback */) -> (Vector, CollisionVec) {
        let xbbox = shape.extended_bbox(attempted);
        let mut collisions = CollisionVec::new();
        // Check out the tilemap
        for ty in xbbox.min_y().to_tile_coord() .. (xbbox.max_y().to_tile_coord() + 1) {
            for tx in xbbox.min_x().to_tile_coord() .. (xbbox.max_x().to_tile_coord() + 1) {
                let tid = TEST_PLACE.tiles[ty][tx];
                // TODO well really this should be...  if there's a /shape/
                if ! TEST_PLACE.tileset.tiles[tid as usize].solid {
                    continue;
                }

                let tile_polygon = Polygon::new([
                    point2(tx as i16 * 8, ty as i16 * 8),
                    point2(tx as i16 * 8 + 8, ty as i16 * 8),
                    point2(tx as i16 * 8 + 8, ty as i16 * 8 + 8),
                    point2(tx as i16 * 8, ty as i16 * 8 + 8),
                ]);
                let maybe_hit = shape.slide_towards(&tile_polygon, self.velocity);
                if let Some(hit) = maybe_hit {
                    collisions.push(hit);
                }
            }
        }

        // FIXME klinklang sorts by touchdist then touchtype, but i thought touchdist was
        // meaningless??
        collisions.as_mut_slice().sort_unstable_by_key(|collision| collision.touchdist);

        // Look through the objects we'll hit, in the order we'll /touch/ them,
        // and stop at the first that blocks us
        let mut allowed_amount = None;
        let mut trim_collisions_to = collisions.len();
        for (i, collision) in collisions.iter().enumerate() {
            // FIXME collision.attempted = attempted

            // If we've already found something that blocks us, and this
            // collision requires moving further, then stop here.  This allows
            // for ties
            if let Some(allowed_amount) = allowed_amount {
                if allowed_amount < collision.amount {
                    trim_collisions_to = i;
                    break;
                }
            }

            // Check if the other shape actually blocks us
            /* XXX no need for this yet; if there's a shape, it blocks
            local passable = pass_callback and pass_callback(collision)
            if passable == 'retry' then
                -- Special case: the other object just moved, so keep moving
                -- and re-evaluate when we hit it again.  Useful for pushing.
                if i > 1 and collisions[i - 1].shape == collision.shape then
                    -- To avoid loops, don't retry a shape twice in a row
                    passable = false
                else
                    local new_collision = shape:slide_towards(collision.shape, attempted)
                    if new_collision then
                        new_collision.shape = collision.shape
                        for j = i + 1, #collisions + 1 do
                            if j > #collisions or not _collision_sort(collisions[j], new_collision) then
                                table.insert(collisions, j, new_collision)
                                break
                            end
                        end
                    end
                end
            end
            */
            let passable = false;

            // If we're hitting the object and it's not passable, stop here
            if allowed_amount.is_none() && ! passable && collision.touchtype == Contact::Collide {
                allowed_amount = Some(collision.amount);
            }

            // Log the last contact with each shape
            // XXX collision.passable = passable
            // XXX hits[collision.shape] = collision
        }

        while collisions.len() > trim_collisions_to {
            collisions.pop();
        }

        match allowed_amount {
            None => {
                // We don't hit anything!  Return the entire movement
                return (attempted, collisions);
            }
            Some(a) if a >= 1 => {
                // We're moving towards something, but don't hit it
                return (attempted, collisions);
            }
            Some(a) => {
                // We do hit something.  Cap our movement proportionally
                return (attempted * a, collisions);
            }
        }
    }

    /// Move this entity through the world by some amount, respecting collision.  Returns the
    /// distance actually travelled.
    fn nudge(&mut self, mut displacement: Vector) -> Vector {
        /*
        pushers = pushers or {}
        pushers[self] = true
        */

        /*
        -- Set up the hit callback, which also tells other actors that we hit them
        local already_hit = {}
        local pass_callback = function(collision)
            return self:_collision_callback(collision, pushers, already_hit)
        end
        */

        let hitbox = self.bbox.translate(&self.position.to_vector());

        // Main movement loop!  Try to slide in the direction of movement; if that
        // fails, then try to project our movement along a surface we hit and
        // continue, until we hit something head-on or run out of movement.
        // TODO rename a LOT of these variables and properties, maybe in LÖVE too
        let mut total_movement = Vector::zero();
        let mut stuck_counter = 0;
        loop {
            let lexy_polygon = Polygon::new([
                hitbox.origin + total_movement,
                hitbox.top_right() + total_movement,
                hitbox.bottom_right() + total_movement,
                hitbox.bottom_left() + total_movement,
            ]);

            // TODO return hits up here?
            let (successful, hits) = self.collider_sweep(&lexy_polygon, displacement /*, pass_callback */);
            //self.shape:move(successful:unpack())
            self.position += successful;
            total_movement += successful;

            /* XXX
            if xxx_no_slide then
                break
            end
            */
            let remaining = displacement - successful;
            // FIXME these values are completely arbitrary and i cannot justify them
            if remaining.x.abs() * 16 < 1 && remaining.y.abs() * 16 < 1 {
                break;
            }

            // FIXME this shouldn't be in here...  or should it?  only for self movement obviously
            // but this seems like the right place?
            match slide_along_normals(&hits, self.velocity) {
                SlideResult::Stuck => {
                    self.velocity = Vector::zero();
                }
                SlideResult::Slid(new_velocity) => {
                    self.velocity = new_velocity;
                }
            }

            // Find the allowed slide direction that's closest to the direction of movement.
            // TODO maybe this should just be Option haha
            match slide_along_normals(&hits, remaining) {
                SlideResult::Stuck => {
                    break;
                }
                SlideResult::Slid(direction) => {
                    displacement = direction;
                }
            }

            // FIXME why am i doing this twice
            if displacement.x.abs() * 16 < 1 && displacement.y.abs() * 16 < 1 {
                break;
            }

            // Automatically break if we don't move for three iterations -- not
            // moving once is okay because we might slide, but three indicates a
            // bad loop somewhere
            // XXX well, wait, aren't we only REALLY stuck if the remaining movement didn't get
            // smaller (or at least, change in some way at all)?  but i don't want to move more
            // than 3 times anyway so maybe it's ok
            if _is_vector_almost_zero(successful) {
                stuck_counter += 1;
                if stuck_counter >= 3 {
                    // FIXME interesting!  i get this when jumping against the crate in a corner in
                    // tech-1; i think because clocks can't handle single angles correctly, so this
                    // is the same problem as walking down a hallway exactly your own height -- is
                    // this still the case?
                    break;
                }
            }
        }

        /* XXX cargo not supported
        -- Move our cargo along with us, independently of their own movement
        -- FIXME this means our momentum isn't part of theirs!!  i think we could
        -- compute effective momentum by comparing position to the last frame, or
        -- by collecting all nudges...?  important for some stuff like glass lexy
        if self.can_carry and self.cargo and not _is_vector_almost_zero(total_movement) then
            for actor in pairs(self.cargo) do
                actor:nudge(total_movement, pushers)
            end
        end

        pushers[self] = nil
        */

        return total_movement//, hits
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
            lexy.velocity.y -= 4;
        }
    }
    if input.down() {
        //lexy.velocity.y = 1;
    }
}
