use std::ops::{Index, IndexMut};

use crate::Pos;

pub(crate) struct Matrix<T> {
    width: usize,
    contents: Vec<T>,
}

impl<T> Matrix<T> {
    pub fn from_fn(size: Pos, f: fn(x: usize, y: usize) -> T) -> Self {
        let (width, height) = (size.0 as usize, size.1 as usize);
        let size = width * height;
        let mut contents = Vec::with_capacity(size);
        contents.extend((0..width * height)
            .map(|i| f(i % width, i / width)));
        Matrix { width, contents }
    }

    #[inline]
    pub fn remove_seam(&mut self, seam: &[Pos]) {
        self.contents.chunks_exact_mut(self.width)
            .zip(seam)
            .for_each(|(aliases, &Pos(x, _y))| {
                let end = &mut aliases[x as usize..];
                if !end.is_empty() { end.rotate_left(1) }
            });
    }
}

impl<T> Index<Pos> for Matrix<T> {
    type Output = T;

    #[inline(always)]
    fn index(&self, pos: Pos) -> &Self::Output {
        let (x, y) = (pos.0 as usize, pos.1 as usize);
        &self.contents[x + y * self.width]
    }
}

impl<T> IndexMut<Pos> for Matrix<T> {
    #[inline(always)]
    fn index_mut(&mut self, pos: Pos) -> &mut T {
        let (x, y) = (pos.0 as usize, pos.1 as usize);
        &mut self.contents[x + y * self.width]
    }
}