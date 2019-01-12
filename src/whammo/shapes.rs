use crate::fixed::Fixed;
use crate::geom::{Point, Rect, Vector, VectorExt, WorldUnit};

/// Allowed rounding error when comparing whether two shapes are overlapping.
/// If they overlap by only this amount, they'll be considered touching.
const PRECISION: Fixed = Fixed::from_bits(3);

#[inline]
pub fn fudge_to_zero(n: Fixed) -> Fixed {
    if n.abs() <= PRECISION {
        0.into()
    }
    else {
        n
    }
}

/// Type of contact to expect from one body moving towards another.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Contact {
    /// This only happens when the two bodies ALREADY overlap.
    Overlap,
    Touch,
    Collide,
}

// XXX?
/*
-- Aggressively de-dupe these extremely common normals
local XPOS = Vector(1, 0)
local XNEG = Vector(-1, 0)
local YPOS = Vector(0, 1)
local YNEG = Vector(0, -1)
*/

struct Shape {
  offset_x: Fixed,
  offset_y: Fixed,
}

impl Shape {
  fn new() -> Shape {
    // XXX
    // self.blockmaps = setmetatable({}, {__mode = 'k'})
    Shape{
      offset_x: 0.into(),
      offset_y: 0.into(),
    }
  }

  // TODO debug?  ha.

/* XXX
function Shape:remember_blockmap(blockmap)
    self.blockmaps[blockmap] = true
end

function Shape:forget_blockmap(blockmap)
    self.blockmaps[blockmap] = nil
end

function Shape:update_blockmaps()
    for blockmap in pairs(self.blockmaps) do
        blockmap:update(self)
    end
end
*/

}

/* XXX?
function Shape:flipx(axis)
    error("flipx not implemented")
end

function Shape:move(dx, dy)
    error("move not implemented")
end

function Shape:move_to(x, y)
    self:move(x - self.xoff, y - self.yoff)
end

function Shape:draw(mode)
    error("draw not implemented")
end

function Shape:normals()
    error("normals not implemented")
end
*/

// TODO gotta merge all these

/// An arbitrary (CONVEX) polygon
pub struct Polygon {
    points: [Point; 4],
    bbox: Rect,
}


/*
function Polygon:clone()
    -- TODO or do this ridiculous repacking (though the vectors need cloning regardless)
    return Polygon(unpack(self:to_coords()))
end

function Polygon:to_coords()
    local coords = {}
    for _, point in ipairs(self.points) do
        table.insert(coords, point.x)
        table.insert(coords, point.y)
    end
    return coords
end

function Polygon:flipx(axis)
    local reverse_coords = {}
    for n, point in ipairs(self.points) do
        reverse_coords[#self.points * 2 - (n * 2 - 1)] = axis * 2 - point.x
        reverse_coords[#self.points * 2 - (n * 2 - 2)] = point.y
    end
    return Polygon(unpack(reverse_coords))
end

function Polygon:_generate_normals()
    self._normals = {}
    local prev_point = self.points[#self.points]
    for _, point in ipairs(self.points) do
        -- Note that this assumes points are given clockwise
        local normal = (prev_point - point):perpendicular()
        prev_point = point

        if normal == Vector.zero then
            -- Ignore zero vectors (where did you even come from)
        elseif normal.x == 0 then
            if normal.y > 0 then
                self._normals[YPOS] = YPOS
            else
                self._normals[YNEG] = YNEG
            end
        elseif normal.y == 0 then
            if normal.x > 0 then
                self._normals[XPOS] = XPOS
            else
                self._normals[XNEG] = XNEG
            end
        else
            -- What a mouthful
            self._normals[normal] = normal:normalized()
        end
    end
end

function Polygon:bbox()
    return self.x0, self.y0, self.x1, self.y1
end

function Polygon:move(dx, dy)
    self.xoff = self.xoff + dx
    self.yoff = self.yoff + dy
    self.x0 = self.x0 + dx
    self.x1 = self.x1 + dx
    self.y0 = self.y0 + dy
    self.y1 = self.y1 + dy
    for _, point in ipairs(self.points) do
        point.x = point.x + dx
        point.y = point.y + dy
    end
    self:update_blockmaps()
end

function Polygon:center()
    -- TODO uhh
    return (self.x0 + self.x1) / 2, (self.y0 + self.y1) / 2
end

function Polygon:draw(mode)
    love.graphics.polygon(mode, self:to_coords())
end

function Polygon:normals()
    return self._normals
end

-- TODO implement this for other types
function Polygon:intersection_with_ray(start, direction)
    local perp = direction:perpendicular()
    -- TODO could save a little effort by passing these in too, maybe
    local startdot = start * direction
    local startperpdot = start * perp
    local pt0 = self.points[#self.points]
    local dot0 = pt0 * perp
    local minpt = nil
    local mindot = math.huge
    for _, point in ipairs(self.points) do
        local pt, dot
        local pt1 = point
        local dot1 = pt1 * perp
        if dot0 == dot1 then
            -- This edge is parallel to the ray.  If it's also collinear to the
            -- ray, figure out where it hits
            if dot0 == startperpdot then
                local startdot = start * direction
                local ldot0 = pt0 * direction
                local ldot1 = pt1 * direction
                if (ldot0 <= startdot and startdot <= ldot1) or
                    (ldot1 <= startdot and startdot <= ldot0)
                then
                    -- Ray starts somewhere inside this line, so the start
                    -- point must be the closest point
                    return start, 0
                elseif ldot0 < startdot and ldot1 < startdot then
                    -- Ray starts after this line and misses it entirely;
                    -- do nothing
                elseif ldot0 < ldot1 then
                    pt = pt0
                    dot = ldot0
                else
                    pt = pt1
                    dot = ldot1
                end
            end
        elseif (dot0 <= startperpdot and startperpdot <= dot1) or
            (dot1 <= startperpdot and startperpdot <= dot0)
        then
            pt = pt0 + (pt1 - pt0) * (startperpdot - dot0) / (dot1 - dot0)
            dot = pt * direction
        end
        if pt then
            if dot >= startdot and dot < mindot then
                mindot = dot
                minpt = pt
            end
        end
        pt0 = pt1
        dot0 = dot1
    end
    -- TODO i feel like this doesn't really do the right thing if the start
    -- point is inside the poly?  should it, i dunno, return the point instead,
    -- since that's the first point where the ray intersects the polygon itself
    -- rather than an edge?
    return minpt, mindot
end
*/


pub enum CollisionNormal {
    Blocked,
    Constrained(Vector, Fixed),
    Free,
}
#[derive(Debug)]
pub struct Collision {
    pub movement: Vector,
    pub amount: WorldUnit,
    pub touchdist: WorldUnit,
    pub touchtype: Contact,
    pub _slide: bool,

    pub left_normal: Option<Vector>,
    pub right_normal: Option<Vector>,
    pub left_normal_dot: Fixed,
    pub right_normal_dot: Fixed,
}

impl Polygon {
    pub fn new(points: [Point; 4]) -> Polygon {
        // TODO generate_normals?  whoof
        let bbox = Rect::from_points(&points);
        Polygon{
            points,
            bbox,
        }
    }

    /// Extend a bbox along a movement vector (to enclose all space it might cross
    /// along the way)
    pub fn extended_bbox(&self, d: Vector) -> Rect {
        let mut bbox = self.bbox.clone();

        if d.x < 0 {
            bbox.origin.x += d.x;
            bbox.size.width -= d.x;
        }
        else if d.x > 0 {
            bbox.size.width += d.x;
        }

        if d.y < 0 {
            bbox.origin.y += d.y;
            bbox.size.height -= d.y;
        }
        else if d.y > 0 {
            bbox.size.height += d.y;
        }

        bbox
    }

    pub fn center(&self) -> Point {
        self.bbox.origin.add_size(&(self.bbox.size / Fixed::promote(2)))
    }

    pub fn project_onto_axis(&self, axis: Vector) -> (Fixed, Fixed, Point, Point) {
        let mut minpt = self.points[0];
        let mut maxpt = minpt;
        let mut min = axis.dot(minpt.to_vector());
        let mut max = min;
        for &pt in self.points.iter().skip(1) {
            let dot = axis.dot(pt.to_vector());
            if dot < min {
                min = dot;
                minpt = pt;
            }
            else if dot > max {
                max = dot;
                maxpt = pt;
            }
        }
        return (min, max, minpt, maxpt);
    }

    /// If this shape were to move by a given distance, would it collide with the given other
    /// shape?  If no, returns None.  If yes, returns Some(Collision).
    ///
    /// Note that a Collision is returned even if the two shapes would exactly touch without
    /// colliding, or would exactly slide against each other.
    // FIXME couldn't there be a much simpler version of this for two AABBs?
    pub fn slide_towards(&self, other: &Polygon, movement: Vector) -> Option<Collision> {
        // We cannot possibly collide if the bboxes don't overlap
        let our_bbox = self.extended_bbox(movement);
        if ! our_bbox.intersects(&other.bbox) {
            return None;
        }

        // Use the separating axis theorem.
        // 1. Choose a bunch of axes, generally normals of the shapes.
        // 2. Project both shapes along each axis.
        // 3. If the projects overlap along ANY axis, the shapes overlap.  Otherwise, they don't.
        // This code also does a couple other things.
        // b. It uses the direction of movement as an extra axis, in order to find the minimum
        //    possible movement between the two shapes.
        // a. It keeps values around in terms of their original vectors, rather than lengths or
        //    normalized vectors, to avoid precision loss from taking square roots.

        // XXX no subshapes support
        /*
        if other.subshapes then
            return self:_multi_slide_towards(other, movement)
        end
        */

        // Mapping of normal vectors (i.e. projection axes) to their normalized
        // versions (needed for comparing the results of the projection)
        // FIXME is the move normal actually necessary, or was it just covering up
        // my bad math before?
        let movenormal = movement.perpendicular();
        /*
        local movenormal = movement:perpendicular()
        movenormal._is_move_normal = true
        local axes = {}
        if movenormal ~= Vector.zero then
            axes[movenormal] = movenormal:normalized()
        end
        for norm, norm1 in pairs(self:normals()) do
            axes[norm] = norm1
        end
        for norm, norm1 in pairs(other:normals()) do
            axes[norm] = norm1
        end
        */

        let mut left_max_dot = Fixed::min_value();
        let mut left_norm = None;
        let mut right_max_dot = Fixed::min_value();
        let mut right_norm = None;

        // Project both shapes onto each axis and look for the minimum distance
        let mut maxamt = Fixed::min_value();
        let mut maxnumer = 1.into();
        let mut maxdenom = 1.into();
        // XXX this is a weird default
        let mut touchtype = Contact::Overlap;
        let mut slide_axis = None;
        // FIXME i can ditch the normalized axes entirely; just need to make sure
        // no callers are relying on getting them in normals
// XXX what does this loop header actually look like?
        for &(mut fullaxis) in &[
            (self.points[1] - self.points[0]).perpendicular(),
            (self.points[2] - self.points[1]).perpendicular(),
            (self.points[3] - self.points[2]).perpendicular(),
            (self.points[0] - self.points[3]).perpendicular(),
            (other.points[1] - other.points[0]).perpendicular(),
            (other.points[2] - other.points[1]).perpendicular(),
            (other.points[3] - other.points[2]).perpendicular(),
            (other.points[0] - other.points[3]).perpendicular(),
        ] {
            if fullaxis == Vector::zero() {
                continue;
            }

            let (min1, max1, minpt1, maxpt1) = self.project_onto_axis(fullaxis);
            let (min2, max2, minpt2, maxpt2) = other.project_onto_axis(fullaxis);
            let mut axis = fullaxis.normalize();
            let dist;
            let sep;
            if min1 < min2 {
                // 1 appears first, so take the distance from 1 to 2
                // Ignore extremely tiny overlaps, which are likely precision errors
                dist = fudge_to_zero(min2 - max1);
                sep = minpt2 - maxpt1;
            }
            else {
                // Other way around
                dist = fudge_to_zero(min1 - max2);
                // Note that sep is always the vector from us to them
                sep = maxpt2 - minpt1;
                // Likewise, flip the axis so it points towards them
                axis = -axis;
                fullaxis = -fullaxis;
            }

            // Negative distance means the shapes overlap from this perspective, which is
            // inconclusive
            if dist < 0 {
              continue;
            }

            // Update touchtype
            if dist > 0 {
                touchtype = Contact::Collide;
            }
            // XXX excuse me what
            else if touchtype == Contact::Overlap {
                touchtype = Contact::Touch;
            }

            // This dot product is positive if we're moving closer along this
            // axis, negative if we're moving away
            let dot = fudge_to_zero(movement.dot(fullaxis));

            if dot < 0 || (dot == 0 && dist > 0) {
                // Even if the shapes are already touching, they're not moving
                // closer together, so they can't possibly collide.  Stop here.
                // FIXME this means collision detection is not useful for finding touches
                return None;
            }
            else if dist == 0 && dot == 0 {
                // Zero dot and zero distance mean the movement is parallel
                // and the shapes can slide against each other.  But we still
                // need to check other axes to know if they'll actually touch.
                slide_axis = Some(fullaxis);
                continue;
            }

            // Figure out how much movement is allowed, as a fraction.
            // Conceptually, the answer is the movement projected onto the
            // axis, divided by the separation projected onto the same
            // axis.  Stuff cancels, and it turns out to be just the ratio
            // of dot products (which makes sense).  Vectors are neat.
            // Note that slides are meaningless here; a shape could move
            // perpendicular to the axis forever without hitting anything.
            let numer = sep.dot(fullaxis);
            let amount = fudge_to_zero(numer / dot);

            // TODO i think i could avoid this entirely by using a cross
            // product instead?
            // FIXME i had to fix this here, so fix it in LÖVE too.  but also in fact, uh,
            // maybe write some tests and rejigger this code a bit too
            if maxamt > Fixed::min_value() && (amount - maxamt).abs() < PRECISION {
                // Equal, ish
            }
            else if amount > maxamt {
                maxamt = amount;
                maxnumer = numer;
                maxdenom = dot;
                // XXX normals normals = {};
                left_norm = None;
                right_norm = None;
                left_max_dot = Fixed::min_value();
                right_max_dot = Fixed::min_value();
            }
            else {
                continue;
            }

            // XXX continue if this is a move normal
            // Now all that's left to do is merge the collision normal with what we've got so
            // far

            // FIXME these are no longer de-duplicated, hmm
            let normal = -fullaxis;
            // XXX normals normals[normal] = -axis;

            let ourdot = -(movement.dot(axis));
            // Skip normals that face away from us
            // XXX is this right, we could skip two iterations if we flipped it
            if ourdot > 0 {
                continue;
            }

            // Determine if this normal is on our left or right
            let perpdot = movenormal.dot(normal);

            // TODO explain this better, but the idea is: using the greater dot means using the slope that's furthest away from us, which resolves corners nicely because two normals on one side HAVE to be a corner, they can't actually be one in front of the other
            // TODO should these do something on a tie?
            if perpdot <= PRECISION && ourdot > left_max_dot {
                left_norm = Some(normal);
                left_max_dot = ourdot;
            }
            if perpdot >= -PRECISION && ourdot > right_max_dot {
                right_norm = Some(normal);
                right_max_dot = ourdot;
            }
        }

        if touchtype == Contact::Overlap {
            // Shapes are already colliding
            // FIXME should have /some/ kind of gentle rejection here; should be
            // easier now that i have touchdist
            //error("seem to be inside something!!  stopping so you can debug buddy  <3")
            return Some(Collision{
                movement: Vector::zero(),
                amount: 0.into(),
                touchdist: 0.into(),
                touchtype: Contact::Overlap,
                _slide: false,

                left_normal: None,
                right_normal: None,
                left_normal_dot: Fixed::min_value(),
                right_normal_dot: Fixed::min_value(),
            });
        }
        else if maxamt > 1 && touchtype == Contact::Collide {
            // We're allowed to move further than the requested distance, AND we
            // won't end up touching.  (Touching is handled as a slide below!)
            return None;
        }

        if let Some(slide_axis) = slide_axis {
            // This is a slide; we will touch (or are already touching) the other
            // object, but can continue past it.  (If we wouldn't touch, amount
            // would exceed 1, and we would've returned earlier.)
            // touchdist is how far we can move before we touch.  If we're already
            // touching, then the touch axis will be the max distance, the dot
            // products above will be zero, and amount will be nonsense.  If not,
            // amount is correct.
            let touchdist;
            if touchtype == Contact::Collide {
                touchdist = 0.into();
            }
            else {
                touchdist = maxamt;
            }

            // Since we're touching, the slide axis is also a valid normal, along
            // with any collision normals
            //XXX normals normals[-slide_axis] = -slide_axis.normalized();
            if -slide_axis.dot(movenormal) < 0 {
                left_norm = Some(-slide_axis);
                left_max_dot = 0.into();
            }
            else {
                right_norm = Some(-slide_axis);
                right_max_dot = 0.into();
            }

            return Some(Collision{
                movement: movement,
                amount: 1.into(),
                touchdist: touchdist,
                touchtype: Contact::Touch,
                //normals: normals,

                _slide: true,
                left_normal: left_norm,
                right_normal: right_norm,
                left_normal_dot: left_max_dot,
                right_normal_dot: right_max_dot,
            });
        }
        else if maxamt == Fixed::min_value() {
            // We don't hit anything at all!
            return None;
        }

        return Some(Collision{
            // Minimize rounding error by repeating the same division we used to
            // get amount, but multiplying first
            movement: movement * maxnumer / maxdenom,
            amount: maxamt,
            touchdist: maxamt,
            touchtype: Contact::Collide,
            //normals: normals,

            _slide: false,
            left_normal: left_norm,
            right_normal: right_norm,
            left_normal_dot: left_max_dot,
            right_normal_dot: right_max_dot,
        });
    }
}

// XXX no multi shape support
/*
function Polygon:_multi_slide_towards(other, movement)
    local ret
    for _, subshape in ipairs(other.subshapes) do
        local collision = self:slide_towards(subshape, movement)
        if collision == nil then
            -- Do nothing
        elseif ret == nil then
            -- First result; just accept it
            ret = collision
        else
            -- Need to combine
            if collision.amount < ret.amount then
                ret = collision
            elseif collision.amount == ret.amount then
                ret.touchdist = math.min(ret.touchdist, collision.touchdist)
                if ret.touchtype == 0 then
                    ret.touchtype = collision.touchtype
                end
                -- FIXME would be nice to de-dupe here too
                for full, norm in pairs(collision.normals) do
                    ret.normals[full] = norm
                end
                if collision.left_normal_dot > ret.left_normal_dot then
                    ret.left_normal_dot = collision.left_normal_dot
                    ret.left_normal = collision.left_normal
                end
                if collision.right_normal_dot > ret.right_normal_dot then
                    ret.right_normal_dot = collision.right_normal_dot
                    ret.right_normal = collision.right_normal
                end
            end
        end
    end

    return ret
end
*/


/*
-- An AABB, i.e., an unrotated rectangle
local Box = Polygon:extend{
    -- Handily, an AABB only has two normals: the x and y axes
    _normals = { [XPOS] = XPOS, [YPOS] = YPOS },
}

function Box:init(x, y, width, height, _xoff, _yoff)
    Polygon.init(self, x, y, x + width, y, x + width, y + height, x, y + height)
    self.width = width
    self.height = height
    self.xoff = _xoff or 0
    self.yoff = _yoff or 0
end

function Box:clone()
    -- FIXME i don't think most shapes clone xoff/yoff correctly, oops...  ARGH this breaks something though
    return Box(self.x0, self.y0, self.width, self.height)
    --return Box(self.x0, self.y0, self.width, self.height, self.xoff, self.yoff)
end

function Box:__tostring()
    return "<Box>"
end

function Box:flipx(axis)
    return Box(axis * 2 - self.x0 - self.width, self.y0, self.width, self.height)
end

function Box:_generate_normals()
end

function Box:center()
    return self.x0 + self.width / 2, self.y0 + self.height / 2
end
*/
