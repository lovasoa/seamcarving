use std::ops::{Sub, Add};

#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug)]
pub(crate) struct Pos(pub u32, pub u32);

impl Pos {
    pub fn successors(self, size: Pos) -> PosLine {
        let Pos(x0, y0) = self;
        let x_end = (x0 + 1).min(size.0 - 1);
        let y = y0 + 1;
        let (x, y) = if y < size.1 {
            (x0.saturating_sub(1), y)
        } else { (x_end + 1, y0) };
        PosLine { x, y, x_end }
    }

    pub fn predecessors(self, size: Pos) -> PosLine {
        let Pos(x0, y0) = self;
        let x_end = (x0 + 1).min(size.0 - 1);
        let (x, y) = if let Some(y) = y0.checked_sub(1) {
            (x0.saturating_sub(1), y)
        } else { (x_end + 1, 0) };
        PosLine { x, y, x_end }
    }

    pub fn iter_in_rect(start: Pos, end: Pos) -> RectIterator {
        RectIterator { current: start, start, end }
    }

    /// Returns the top,bottom,left and right positions, in this order
    pub fn surrounding(self, size: Pos) -> [Pos; 4] {
        let Pos(x, y) = self;
        [
            Pos(x, y.saturating_sub(1)),
            Pos(x, (y + 1).min(size.1 - 1)),
            Pos(x.saturating_sub(1), y),
            Pos((x + 1).min(size.0 - 1), y),
        ]
    }
}

pub(crate) struct PosLine { x: u32, y: u32, x_end: u32 }

impl Iterator for PosLine {
    type Item = Pos;

    #[inline(always)]
    fn next(&mut self) -> Option<Self::Item> {
        if self.x > self.x_end { None } else {
            let p = Pos(self.x, self.y);
            self.x += 1;
            Some(p)
        }
    }
}

pub(crate) struct RectIterator {
    current: Pos,
    start: Pos,
    end: Pos,
}

impl Iterator for RectIterator {
    type Item = Pos;

    #[inline(always)]
    fn next(&mut self) -> Option<Self::Item> {
        if self.current.1 == self.end.1 {
            None
        } else {
            let pos = self.current;
            self.current.0 += 1;
            if self.current.0 == self.end.0 {
                self.current.0 = self.start.0;
                self.current.1 += 1;
            }
            Some(pos)
        }
    }
}

impl Add<Pos> for Pos {
    type Output = Pos;

    #[inline(always)]
    fn add(self, rhs: Pos) -> Self::Output {
        Pos(self.0 + rhs.0, self.1 + rhs.1)
    }
}

impl Sub<Pos> for Pos {
    type Output = Pos;

    #[inline(always)]
    fn sub(self, rhs: Pos) -> Self::Output {
        Pos(self.0.saturating_sub(rhs.0), self.1.saturating_sub(rhs.1))
    }
}
