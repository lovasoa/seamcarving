use image::{GenericImage, GenericImageView, ImageBuffer, Pixel};
use pathfinding::prelude::dijkstra;
use std::ops::{Add, Sub, Mul};

#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug)]
pub enum Direction { X, Y }

impl Direction {
    #[inline(always)]
    pub fn other(&self) -> Direction {
        match self {
            Direction::X => Direction::Y,
            Direction::Y => Direction::X,
        }
    }
    #[inline(always)]
    pub fn all() -> [Direction; 2] { [Direction::X, Direction::Y] }
}

#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug)]
struct Pos(u32, u32);

impl Pos {
    #[inline(always)]
    fn before(&self, max: &Pos) -> bool {
        self.0 < max.0 && self.1 < max.1
    }
    fn successors<'a>(&'a self, dir: Direction) -> impl Iterator<Item=Pos> + 'a {
        let orth = dir.other();
        let orth_projection = self.component(orth);
        let dir_vec = Pos::from(dir);
        let orth_vec = Pos::from(orth);
        vec![
            orth_projection.checked_sub(1),
            Some(orth_projection),
            orth_projection.checked_add(1)
        ].into_iter().filter_map(move |x| {
            x.map(|x| orth_vec * x + (*self + dir_vec) * dir_vec)
        })
    }
    #[inline(always)]
    fn component(&self, d: Direction) -> u32 {
        match d {
            Direction::X => self.0,
            Direction::Y => self.1,
        }
    }
}

fn max_pos<IMG: GenericImageView>(img: &IMG) -> Pos {
    Pos(img.width(), img.height())
}

impl From<Pos> for (u32, u32) {
    #[inline(always)]
    fn from(Pos(x, y): Pos) -> Self { (x, y) }
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

impl Mul<u32> for Pos {
    type Output = Pos;

    #[inline(always)]
    fn mul(self, rhs: u32) -> Self::Output {
        Pos(self.0 * rhs, self.1 * rhs)
    }
}

impl Mul<Pos> for Pos {
    type Output = Pos;

    #[inline(always)]
    fn mul(self, rhs: Pos) -> Self::Output {
        Pos(self.0 * rhs.0, self.1 * rhs.1)
    }
}

impl From<Direction> for Pos {
    #[inline(always)]
    fn from(d: Direction) -> Self {
        match d {
            Direction::X => Pos(1, 0),
            Direction::Y => Pos(0, 1)
        }
    }
}

impl From<(u32, u32)> for Pos {
    #[inline(always)]
    fn from((x, y): (u32, u32)) -> Self {
        Pos(x, y)
    }
}

fn energy_fn<IMG: GenericImageView>(img: &IMG, pos: &Pos) -> u32 {
    use num_traits::cast::ToPrimitive;
    let last_pos = &max_pos(img);
    Direction::all().iter()
        .map(|&dir| -> u32{
            let mut next = *pos + dir.into();
            if !next.before(last_pos) { next = *pos }
            let prev = *pos - dir.into();
            let p1 = img.get_pixel(next.0, next.1);
            let p2 = img.get_pixel(prev.0, prev.1);
            p1.channels().iter().zip(p2.channels())
                .map(|(&a, &b)| {
                    let a = a.to_i16().unwrap_or(i16::max_value());
                    let b = b.to_i16().unwrap_or(i16::max_value());
                    ((a - b).abs() as u32).pow(2)
                }).sum()
        }).sum()
}

struct Carved<'a, IMG: GenericImageView>
    where <IMG as GenericImageView>::Pixel: 'static {
    img: &'a IMG,
    dir: Direction,
    removed: u32,
    pos_aliases: Vec<u32>,
}

impl<'a, IMG: GenericImageView> Carved<'a, IMG>
    where <IMG as GenericImageView>::Pixel: 'static {
    fn new(img: &'a IMG, dir: Direction) -> Self {
        let last_pos = max_pos(img);
        let max_dir = last_pos.component(dir);
        let max_orth = last_pos.component(dir.other());

        let pos_aliases = (0..max_dir * max_orth)
            .map(|i| i % max_orth)
            .collect();
        Carved { img, dir, removed: 0, pos_aliases }
    }
    fn remove_seam(&mut self, seam: &Vec<Pos>) {
        let orth = self.dir.other();
        let max_orth = max_pos(self.img).component(orth);
        self.pos_aliases.chunks_exact_mut(max_orth as usize)
            .zip(seam)
            .for_each(|(aliases, pos)| {
                let n = pos.component(orth);
                let end = &mut aliases[n as usize..];
                if end.len() > 0 { end.rotate_left(1) }
            });
        self.removed += 1;
    }
    /// Given a position in the carved image, return a position in the original
    #[inline(always)]
    fn transform_pos(&self, pos: Pos) -> Pos {
        let orth = self.dir.other();
        let max_orth = max_pos(self.img).component(orth);
        let i = pos.component(self.dir);
        let j = pos.component(orth);
        let u = Pos::from(self.dir);
        let v = Pos::from(orth);
        let j_alias = self.pos_aliases[(i * max_orth + j) as usize];
        u * i + v * j_alias
    }
    fn finalize(self) -> ImageBuffer<IMG::Pixel, Vec<<<IMG as GenericImageView>::Pixel as Pixel>::Subpixel>> {
        let (w, h) = self.dimensions();
        ImageBuffer::from_fn(w, h, |x, y| {
            self.get_pixel(x, y)
        })
    }
}

impl<'a, IMG: GenericImageView> GenericImageView for Carved<'a, IMG> {
    type Pixel = IMG::Pixel;
    type InnerImageView = IMG::InnerImageView;

    #[inline(always)]
    fn dimensions(&self) -> (u32, u32) {
        let p = max_pos(self.img) - Pos::from(self.dir.other()) * self.removed;
        p.into()
    }

    #[inline(always)]
    fn bounds(&self) -> (u32, u32, u32, u32) {
        let (w, h) = self.dimensions();
        (0, 0, w, h)
    }

    #[inline(always)]
    fn get_pixel(&self, x: u32, y: u32) -> Self::Pixel {
        let Pos(u, v) = self.transform_pos(Pos(x, y));
        self.img.get_pixel(u, v)
    }

    fn inner(&self) -> &Self::InnerImageView {
        self.img.inner()
    }
}

fn carve_one<IMG: GenericImageView>(
    carved: &mut Carved<IMG>,
    dir: Direction,
) {
    let last_pos = &max_pos(carved);
    let end_coord = last_pos.component(dir) - 1;
    let (seam, _cost): (Vec<Option<Pos>>, u32) = dijkstra(
        &None,
        |maybe_pos: &Option<Pos>| -> Vec<_>{
            match maybe_pos {
                None =>
                    (0..Pos::from(carved.dimensions()).component(dir.other()))
                        .map(|x| (Some(Pos::from(dir.other()) * x), 0))
                        .collect(),
                Some(pos) =>
                    pos.successors(dir)
                        .filter(|pos| pos.before(last_pos))
                        .map(|pos| (Some(pos), energy_fn(carved, &pos)))
                        .collect(),
            }
        },
        |maybe_pos: &Option<Pos>| {
            maybe_pos.map_or(false, |p|
                p.component(dir) == end_coord,
            )
        },
    ).expect("No seam found. This is a bug in seamcarving");
    let seam: Vec<Pos> = seam.into_iter().skip(1).collect::<Option<_>>().unwrap();
    carved.remove_seam(&seam);
}

fn carve<IMG: GenericImageView>(
    img: &IMG,
    dir: Direction,
    pixel_count: u32,
) -> Carved<IMG>
    where <IMG as GenericImageView>::Pixel: 'static {
    let mut carved = Carved::new(img, dir);
    (0..pixel_count).for_each(|_| carve_one(&mut carved, dir));
    carved
}

pub fn resize<IMG: GenericImage>(
    img: &IMG,
    width: u32,
    height: u32,
) -> ImageBuffer<IMG::Pixel, Vec<<<IMG as GenericImageView>::Pixel as Pixel>::Subpixel>>
    where <IMG as GenericImageView>::Pixel: 'static {
    let Pos(to_remove_x, to_remove_y) = max_pos(img) - Pos(width, height);
    let mut carved_x = carve(img, Direction::Y, to_remove_x);
    let carved_y = carve(&mut carved_x, Direction::X, to_remove_y);
    carved_y.finalize()
}

#[cfg(test)]
mod tests {
    use image::{GrayImage, ImageBuffer, Luma};
    use crate::{resize, energy_fn, Pos};

    #[test]
    fn energy_fn_correct() {
        let img = GrayImage::from_raw(3, 2, vec![
            3, 1, 4,
            1, 5, 9,
        ]).unwrap();
        let energy = ImageBuffer::from_fn(3, 2, |x, y| {
            Luma([energy_fn(&img, &Pos(x, y))])
        });
        let expected = vec![
            (2 * 2 + 2 * 2), (1 * 1 + 4 * 4), (5 * 5 + 3 * 3),
            (2 * 2 + 4 * 4), (4 * 4 + 8 * 8), (5 * 5 + 4 * 4),
        ];
        assert_eq!(energy.into_raw(), expected);
    }

    #[test]
    fn removes_the_right_seam() {
        let raw = vec![
            3, 1, 4, 0, 0, 0, 1, 5,
            9, 2, 6, 0, 0, 0, 5, 3,
            5, 8, 0, 0, 0, 9, 7, 9,
        ];
        let expected = vec![
            3, 1, 4, 0, 0, 1, 5,
            9, 2, 6, 0, 0, 5, 3,
            5, 8, 0, 0, 9, 7, 9,
        ];
        let img = GrayImage::from_raw(8, 3, raw).unwrap();
        let resized = resize(&img, 7, 3);
        assert_eq!(resized.dimensions(), (7, 3));
        assert_eq!(resized.into_raw(), expected);
    }
}
