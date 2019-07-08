use crate::pos::Pos;
use crate::matrix::Matrix;
use std::iter::successors;

pub(crate) struct SeamFinder {
    size: Pos,
    contents: Matrix<Option<SeamElem>>,
}

struct SeamElem {
    predecessor: Pos,
    energy: u32,
}

impl SeamFinder {
    pub fn new(size: Pos) -> Self {
        let contents: Matrix<Option<SeamElem>> = Matrix::from_fn(size, |_, _| None);
        SeamFinder { size, contents }
    }

    pub fn extract_seam<F: FnMut(Pos) -> u32>(&mut self, mut energy: F) -> Vec<Pos> {
        self.fill(energy);
        let mut seam = Vec::with_capacity(self.size.1 as usize);
        // Find the bottom pixel with the lowest energy
        let bottom_y = self.size.1 - 1;
        let init = (0..self.size.0)
            .map(|x| Pos(x, bottom_y))
            .min_by_key(|&p| {
                self.contents[p].take().expect("should have been filled").energy
            });
        seam.extend(successors(init, |&pos| {
            self.clear(pos);
            if pos.1 == 0 { None } else {
                Some(self.contents[pos].take().expect("should be filled").predecessor)
            }
        }));
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