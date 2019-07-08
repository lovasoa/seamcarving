use std::ops::{Sub, Add};
use std::iter::once;

#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug)]
pub(crate) struct Pos(pub u32, pub u32);

impl Pos {
    #[inline(always)]
    pub fn before(self, max: Pos) -> bool {
        self.0 < max.0 && self.1 < max.1
    }
    pub fn successors(self, width: u32, height: u32)
                      -> impl Iterator<Item=Pos> {
        let Pos(x, y) = self;
        once(y + 1)
            .filter(move |&y| y < height)
            .flat_map(move |y| once(x.checked_sub(1))
                .flatten()
                .chain(once(x))
                .chain(once(x + 1)
                    .filter(move |&x| x < width)
                )
                .map(move |x| Pos(x, y)))
    }

    pub fn predecessors(self, size: Pos)
                        -> impl Iterator<Item=Pos> {
        let Pos(x, y) = self;
        once(y)
            .flat_map(|y| y.checked_sub(1))
            .flat_map(move |y|
                once(x.checked_sub(1))
                    .flatten()
                    .chain(once(x))
                    .chain(once(x + 1)
                        .filter(move |&x| x < size.0)
                    )
                    .map(move |x| Pos(x, y)))
    }

    pub fn iter_in_rect(end: Pos) -> impl Iterator<Item=Pos> {
        (0..end.0 * end.1)
            .map(move |i| Pos(i % end.0, i / end.0))
    }

    /// Returns the top,bottom,left and right positions, in this order
    pub fn surrounding(self) -> [Pos; 4] {
        let Pos(x, y) = self;
        [
            Pos(x, y.saturating_sub(1)), Pos(x, y + 1),
            Pos(x.saturating_sub(1), y), Pos(x + 1, y)
        ]
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