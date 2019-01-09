use euclid::{TypedPoint2D, TypedRect, TypedSize2D, TypedVector2D};
use num_traits::clamp;

use crate::fixed::Fixed;
//use fpa::I24F8;

pub type ScreenPixel = u16;
pub type WorldUnit = Fixed;
pub type WorldStorage = i32;
pub type WorldWhole = i16;
pub struct ScreenSpace;
pub struct WorldSpace;

pub type Point = TypedPoint2D<WorldUnit, WorldSpace>;
pub type Rect = TypedRect<WorldUnit, WorldSpace>;
pub type Size = TypedSize2D<WorldUnit, WorldSpace>;
pub type Vector = TypedVector2D<WorldUnit, WorldSpace>;

pub trait RectExt {
    fn touches(&self, other: &Self) -> bool;
}

impl RectExt for Rect {
    /// Like `TypedRect::intersects`, but also returns true if only the edges coincide.
    #[inline]
    fn touches(&self, other: &Self) -> bool {
        self.origin.x <= other.origin.x + other.size.width &&
       other.origin.x <=  self.origin.x + self.size.width &&
        self.origin.y <= other.origin.y + other.size.height &&
       other.origin.y <=  self.origin.y + self.size.height
    }
}

// Versions of the euclid helper functions that also perform fixed conversion
pub fn rect(x: WorldWhole, y: WorldWhole, w: WorldWhole, h: WorldWhole) -> Rect {
    euclid::rect(x.into(), y.into(), w.into(), h.into())
}
pub fn point2(x: WorldWhole, y: WorldWhole) -> Point {
    euclid::point2(x.into(), y.into())
}
pub fn size2(w: WorldWhole, h: WorldWhole) -> Size {
    euclid::size2(w.into(), h.into())
}
pub fn vec2(x: WorldWhole, y: WorldWhole) -> Vector {
    euclid::vec2(x.into(), y.into())
}

#[derive(Clone, Copy)]
pub enum Bounds {
    Empty,
    BBox(Rect),
}

// XXX wait, should the camera be in screenspace?
pub struct Camera {
    pub bounds: Bounds,
    pub size: Size,
    pub position: Point,
    pub margin: Size,
}

impl Camera {
    pub fn new() -> Self {
        Camera{
            bounds: Bounds::Empty,
            size: Size::zero(),
            position: Point::zero(),
            margin: Size::zero(),
        }
    }

    /// Update camera position, moving as little as possible
    pub fn aim_at(&mut self, target: Point) {
        // FIXME would like some more interesting features here like smoothly
        // catching up with the player, platform snapping?

        let x0 = 0 + self.margin.width;
        let x1 = self.size.width - self.margin.width;
        //local minx = self.map.camera_margin_left
        //local maxx = self.map.width - self.map.camera_margin_right - self.width
        let mut newx = self.position.x;
        if target.x - newx < x0 {
            newx = target.x - x0;
        }
        else if target.x - newx > x1 {
            newx = target.x - x1;
        }

        let y0 = 0 + self.margin.height;
        let y1 = self.size.height - self.margin.height;
        //local miny = self.map.camera_margin_top
        //local maxy = self.map.height - self.map.camera_margin_bottom - self.height
        let mut newy = self.position.y;
        if target.y - newy < y0 {
            newy = target.y - y0;
        }
        else if target.y - newy > y1 {
            newy = target.y - y1;
        }
        // FIXME moooove, elsewhere.    only tricky bit is that it still wants to clamp to miny/maxy
        /*
        if self.player.camera_jitter and self.player.camera_jitter > 0 then
                newy = newy + math.sin(self.player.camera_jitter * math.pi) * 3
                newy = math.max(miny, math.min(maxy, newy))
        end
        */

        if let Bounds::BBox(ref bbox) = self.bounds {
            newx = clamp(newx, bbox.min_x(), bbox.max_x() - self.size.width);
            newy = clamp(newy, bbox.min_y(), bbox.max_y() - self.size.height);
        }
        // TODO when i switch to fixed?
        //self.x = math.floor(newx)
        //self.y = math.floor(newy)
        self.position.x = newx;
        self.position.y = newy;
    }
}

/*
function Camera:clone()
    local camera = getmetatable(self)()
    camera:set_size(self.width, self.height)
    camera:set_bounds(self.minx, self.miny, self.maxx, self.maxy)
    camera.margin = self.margin
    camera.x = self.x
    camera.y = self.y
    return camera
end

function Camera:apply()
    love.graphics.translate(-self.x, -self.y)
end

-- Draws the camera parameters, in world coordinates
function Camera:draw()
    love.graphics.rectangle('line',
        self.x + self.width * self.margin,
        self.y + self.height * self.margin,
        self.width * (1 - 2 * self.margin),
        self.height * (1 - 2 * self.margin))
end
*/
