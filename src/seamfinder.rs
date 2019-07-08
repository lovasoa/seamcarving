use crate::pos::Pos;
use crate::matrix::Matrix;
use std::iter::successors;

#[derive(Debug)]
pub(crate) struct SeamFinder {
    size: Pos,
    contents: Matrix<Option<SeamElem>>,
}

#[derive(Debug)]
struct SeamElem {
    predecessor: Pos,
    energy: u32,
}

impl SeamFinder {
    pub fn new(size: Pos) -> Self {
        let contents: Matrix<Option<SeamElem>> = Matrix::from_fn(size, |_, _| None);
        SeamFinder { size, contents }
    }

    pub fn extract_seam<F: FnMut(Pos) -> u32>(&mut self, energy: F) -> Vec<Pos> {
        self.fill(energy);
        let mut seam = Vec::with_capacity(self.size.1 as usize);
        // Find the bottom pixel with the lowest energy
        let bottom_y: Option<u32> = self.size.1.checked_sub(1);
        let init = (0..self.size.0)
            .flat_map(|x| bottom_y.map(|y| Pos(x, y)))
            .min_by_key(|&p| {
                self.contents[p].as_ref().expect("should have been filled").energy
            });
        seam.extend(successors(init, |&pos| {
            dbg!(&self.contents);
            dbg!(pos);
            let next = if pos.1 == 0 { None } else {
                Some(self.contents[pos]
                    .as_ref()
                    .expect("should be filled")
                    .predecessor)
            };
            self.clear(pos);
            next
        }));
        self.size.0 -= 1;
        self.contents.remove_seam(&seam);
        seam
    }

    fn fill<F: FnMut(Pos) -> u32>(&mut self, mut energy: F) {
        for pos in Pos::iter_in_rect(self.size) {
            if self.contents[pos].is_some() { continue; }
            let delta_e = energy(pos);
            let elem = pos.predecessors(self.size)
                .flat_map(|predecessor| {
                    if let Some(e) = &self.contents[predecessor] {
                        Some(SeamElem { predecessor, energy: e.energy + delta_e })
                    } else { None }
                })
                .min_by_key(|e| e.energy)
                .unwrap_or(SeamElem { predecessor: self.size, energy: 0 });
            self.contents[pos] = Some(elem);
        }
    }

    /// Recursively invalidates all cached information about a position
    fn clear(&mut self, p: Pos) {
        let (w, h) = (self.size.0 as u32, self.size.1 as u32);
        self.contents[p] = None;
        for s in p.successors(w, h) {
            if let Some(SeamElem { predecessor, .. }) = &self.contents[s] {
                if *predecessor == p {
                    self.clear(s)
                }
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::seamfinder::SeamFinder;
    use crate::pos::Pos;

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
    fn fills() {
        let mut finder = SeamFinder::new(Pos(10, 10));
        finder.fill(|_| 42);
        Pos::iter_in_rect(finder.size)
            .for_each(|p|
                assert!(finder.contents[p].is_some())
            )
    }
}
