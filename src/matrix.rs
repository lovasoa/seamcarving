use std::fmt::{Debug, Formatter};
use std::ops::{Index, IndexMut};

use crate::Pos;

pub(crate) struct Matrix<T> {
    original_width: usize,
    current_width: usize,
    contents: Vec<T>,
}

impl<T: Debug> Debug for Matrix<T> {
    fn fmt(&self, f: &mut Formatter) -> std::fmt::Result {
        writeln!(f, "Matrix {{")?;
        for line in self.contents.chunks_exact(self.original_width) {
            writeln!(f, "  {:?}", line)?;
        }
        writeln!(f, "}}")
    }
}

impl<T> Matrix<T> {
    pub fn from_fn(size: Pos, mut f: impl FnMut(usize, usize) -> T) -> Self {
        let (width, height) = (size.0 as usize, size.1 as usize);
        let size = width * height;
        let mut contents = Vec::with_capacity(size);
        contents.extend((0..width * height).map(|i| f(i % width, i / width)));
        Matrix {
            original_width: width,
            current_width: width,
            contents,
        }
    }

    #[inline]
    pub fn remove_seam(&mut self, seam: &[Pos]) {
        let current_width = self.current_width;
        self.current_width -= 1;
        self.contents
            .chunks_exact_mut(self.original_width)
            .zip(seam.iter().rev())
            .for_each(|(aliases, &Pos(x, _y))| {
                let end = &mut aliases[x as usize..current_width];
                if !end.is_empty() {
                    end.rotate_left(1)
                }
            });
    }
}

impl<T> Index<Pos> for Matrix<T> {
    type Output = T;

    #[inline(always)]
    fn index(&self, pos: Pos) -> &Self::Output {
        let (x, y) = (pos.0 as usize, pos.1 as usize);
        &self.contents[x + y * self.original_width]
    }
}

impl<T> IndexMut<Pos> for Matrix<T> {
    #[inline(always)]
    fn index_mut(&mut self, pos: Pos) -> &mut T {
        let (x, y) = (pos.0 as usize, pos.1 as usize);
        &mut self.contents[x + y * self.original_width]
    }
}
