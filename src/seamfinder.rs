use std::iter::successors;

use crate::matrix::Matrix;
use crate::pos::Pos;

#[derive(Debug)]
pub(crate) struct SeamFinder {
    size: Pos,

    // The dependencies and energies
    contents: Matrix<Option<SeamElem>>,

    // Vector used during invalid position clearing
    to_clear: Vec<Pos>,

    // min and max x values that will have to be recomputed
    dirty_bounds: DirtyBounds,
}

#[derive(Debug)]
struct SeamElem {
    predecessor_dx: i8,
    energy: u32,
}

#[derive(Debug)]
struct DirtyBounds(u32, u32);

impl DirtyBounds {
    fn clean(size: Pos) -> Self {
        DirtyBounds(size.0, 0)
    }
    fn dirty(size: Pos) -> Self {
        DirtyBounds(0, size.0)
    }
    #[inline(always)]
    fn update(&mut self, Pos(x, _): Pos) {
        if x < self.0 {
            self.0 = x
        }
        if x >= self.1 {
            self.1 = x + 1
        }
    }
}

impl SeamElem {
    #[inline(always)]
    fn new(energy: u32) -> Self {
        SeamElem { predecessor_dx: 0, energy }
    }

    #[inline(always)]
    fn set_dx(&mut self, Pos(x_current, _): Pos, Pos(x_predecessor, _): Pos) {
        self.predecessor_dx = if x_predecessor > x_current {
            (x_predecessor - x_current) as i8
        } else {
            -((x_current - x_predecessor) as i8)
        }
    }

    fn predecessor(&self, pos: Pos) -> Pos {
        let mut p = pos;
        if self.predecessor_dx > 0 {
            p.0 += self.predecessor_dx as u32;
        } else {
            p.0 -= (-self.predecessor_dx) as u32;
        }
        p.1 -= 1;
        p
    }
}

impl SeamFinder {
    pub fn new(size: Pos) -> Self {
        let contents: Matrix<Option<SeamElem>> = Matrix::from_fn(size, |_, _| None);
        let to_clear = Vec::with_capacity(size.1 as usize);
        let dirty_bounds = DirtyBounds::dirty(size);
        SeamFinder {
            size,
            contents,
            to_clear,
            dirty_bounds,
        }
    }

    pub fn extract_seam<F: FnMut(Pos) -> u32>(&mut self, energy: F) -> Vec<Pos> {
        self.fill(energy);
        let mut seam = Vec::with_capacity(self.size.1 as usize);
        // Find the bottom pixel with the lowest energy
        let bottom_y: Option<u32> = self.size.1.checked_sub(1);
        let init = (0..self.size.0)
            .flat_map(|x| bottom_y.map(|y| Pos(x, y)))
            .min_by_key(|&p|
                self.contents[p].as_ref().expect("should have been filled").energy);
        seam.extend(successors(init, |&pos| {
            let next = if pos.1 == 0 {
                None
            } else {
                Some(self.contents[pos]
                    .as_ref()
                    .expect("should be filled")
                    .predecessor(pos))
            };
            self.clear(pos);
            next
        }));
        self.size.0 -= 1;
        self.contents.remove_seam(&seam);
        seam
    }

    fn fill<F: FnMut(Pos) -> u32>(&mut self, mut energy: F) {
        let start = Pos(self.dirty_bounds.0, 0);
        let end = Pos(self.dirty_bounds.1, self.size.1);
        for pos in Pos::iter_in_rect(start, end) {
            if self.contents[pos].is_some() {
                continue;
            }
            let delta_e = energy(pos);
            let mut best_elem = SeamElem::new(std::u32::MAX);
            for predecessor in pos.predecessors(self.size) {
                if let Some(e) = &self.contents[predecessor] {
                    let energy = e.energy + delta_e;
                    if energy < best_elem.energy {
                        best_elem.energy = energy;
                        best_elem.set_dx(pos, predecessor);
                    }
                }
            }
            if best_elem.energy == std::u32::MAX { // We are on the top row
                best_elem.energy = delta_e;
            }
            self.contents[pos] = Some(best_elem);
        }
        self.dirty_bounds = DirtyBounds::clean(self.size);
    }

    /// Recursively invalidates all cached information about a position
    fn clear(&mut self, p: Pos) {
        self.to_clear.push(p);
        while let Some(pos) = self.to_clear.pop() {
            self.contents[pos] = None;
            self.dirty_bounds.update(pos);
            for s in pos.successors(self.size) {
                if let Some(e) = &self.contents[s] {
                    if e.predecessor(s) == pos {
                        self.to_clear.push(s)
                    }
                }
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::pos::Pos;
    use crate::seamfinder::SeamFinder;

    #[test]
    fn extracts_correct_seam() {
        let mut finder = SeamFinder::new(Pos(3, 2));
        let energy_fn = |Pos(x, _y)| x;
        // energy matrix:
        // 0  1  2
        // | \  \
        // 0  1  2
        let s1 = finder.extract_seam(energy_fn);
        assert_eq!(s1, vec![Pos(0, 1), Pos(0, 0)]);
    }

    #[test]
    fn larger_image_1024x256() {
        let (w, h) = (1024, 256);
        let mut finder = SeamFinder::new(Pos(w, h));
        let energy_fn = |Pos(x, _y)| x;
        let s1 = finder.extract_seam(energy_fn);
        let expected: Vec<_> = (0..h).rev().map(|y| Pos(0, y)).collect();
        assert_eq!(s1, expected);
    }

    #[test]
    fn fills() {
        let mut finder = SeamFinder::new(Pos(10, 10));
        finder.fill(|_| 42);
        Pos::iter_in_rect(Pos(0, 0), finder.size)
            .for_each(|p| assert!(finder.contents[p].is_some()))
    }
}
