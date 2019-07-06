use std::ops::{Sub, Add};

#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug)]
pub(crate) struct Pos(pub u32, pub u32);

impl Pos {
    #[inline(always)]
    pub fn before(self, max: Pos) -> bool {
        self.0 < max.0 && self.1 < max.1
    }
    pub fn successors(self) -> impl Iterator<Item=Pos> {
        let Pos(x, y) = self;
        std::iter::once(x.checked_sub(1))
            .flatten()
            .chain(std::iter::once(x))
            .chain(std::iter::once(x + 1))
            .map(move |x| Pos(x, y + 1))
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