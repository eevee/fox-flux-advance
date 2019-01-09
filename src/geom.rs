use num_traits::{Num, clamp, zero};

use core::cmp::PartialOrd;

pub trait Numberlike: Num + Copy + PartialOrd {}

impl<T: Num + Copy + PartialOrd> Numberlike for T {}

#[derive(Clone, Copy)]
pub struct Point<N: Numberlike> {
    pub x: N,
    pub y: N,
}

#[derive(Clone, Copy)]
pub struct Size<N: Numberlike> {
    pub width: N,
    pub height: N,
}

#[derive(Clone, Copy)]
pub struct AABB<N: Numberlike> {
    pub topleft: Point<N>,
    pub size: Size<N>,
}

#[derive(Clone, Copy)]
pub enum Bounds<N: Numberlike> {
    Empty,
    BBox(AABB<N>),
}

impl<N: Numberlike> AABB<N> {
    pub fn x0(&self) -> N {
        self.topleft.x
    }
    pub fn y0(&self) -> N {
        self.topleft.y
    }
    pub fn x1(&self) -> N {
        self.topleft.x + self.size.width
    }
    pub fn y1(&self) -> N {
        self.topleft.y + self.size.height
    }
}

pub struct Camera<N: Numberlike> {
    pub bounds: Bounds<N>,
    pub size: Size<N>,
    pub position: Point<N>,
    pub margin: Size<N>,
}

impl<N: Numberlike> Camera<N> {
    pub fn new() -> Self {
        Camera{
            bounds: Bounds::Empty,
            size: Size{width: zero(), height: zero()},
            position: Point{x: zero(), y: zero()},
            margin: Size{width: zero(), height: zero()},
        }
    }

    /// Update camera position, moving as little as possible
    pub fn aim_at(&mut self, target: Point<N>) {
        // FIXME would like some more interesting features here like smoothly
        // catching up with the player, platform snapping?

        let x0 = zero::<N>() + self.margin.width;
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

        let y0 = zero::<N>() + self.margin.height;
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
            newx = clamp(newx, bbox.x0(), bbox.x1() - self.size.width);
            newy = clamp(newy, bbox.y0(), bbox.y1() - self.size.height);
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
