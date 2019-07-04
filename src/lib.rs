use image::GenericImage;
use pathfinding::prelude::dijkstra;
use std::ops::{Add, Sub, Mul};

#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug)]
pub enum Direction { X, Y }

impl Direction {
    pub fn other(&self) -> Direction {
        match self {
            Direction::X => Direction::Y,
            Direction::Y => Direction::X,
        }
    }
    pub fn all() -> [Direction; 2] { [Direction::X, Direction::Y] }
}

#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug)]
struct Pos(u32, u32);

impl Pos {
    fn in_image<IMG: GenericImage>(&self, img: &IMG) -> bool {
        self.0 < img.width() && self.1 < img.height()
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
    fn component(&self, d: Direction) -> u32 {
        match d {
            Direction::X => self.0,
            Direction::Y => self.1,
        }
    }
}

impl Add<Pos> for Pos {
    type Output = Pos;

    fn add(self, rhs: Pos) -> Self::Output {
        Pos(self.0 + rhs.0, self.1 + rhs.1)
    }
}

impl Sub<Pos> for Pos {
    type Output = Pos;

    fn sub(self, rhs: Pos) -> Self::Output {
        Pos(self.0.saturating_sub(rhs.0), self.1.saturating_sub(rhs.1))
    }
}

impl Mul<u32> for Pos {
    type Output = Pos;

    fn mul(self, rhs: u32) -> Self::Output {
        Pos(self.0 * rhs, self.1 * rhs)
    }
}

impl Mul<Pos> for Pos {
    type Output = Pos;

    fn mul(self, rhs: Pos) -> Self::Output {
        Pos(self.0 * rhs.0, self.1 * rhs.1)
    }
}

impl From<Direction> for Pos {
    fn from(d: Direction) -> Self {
        match d {
            Direction::X => Pos(1, 0),
            Direction::Y => Pos(0, 1)
        }
    }
}

fn energy_fn<IMG: GenericImage>(img: &IMG, pos: &Pos) -> u32 {
    use image::Pixel;
    use num_traits::cast::ToPrimitive;
    Direction::all().iter()
        .map(|&dir| -> u32{
            let mut next = *pos + dir.into();
            if !next.in_image(img) { next = *pos }
            let prev = *pos - dir.into();
            let p1 = img.get_pixel(next.0, next.1);
            let p2 = img.get_pixel(prev.0, prev.1);
            p1.channels().iter().zip(p2.channels())
                .map(|(&a, &b)| {
                    let a = a.to_i16().unwrap_or(i16::max_value());
                    let b = b.to_i16().unwrap_or(i16::max_value());
                    (a - b).pow(2) as u32
                }).sum()
        }).sum()
}

struct Carved<IMG: GenericImage> {
    img: IMG,
    dir: Direction,
    removed: u32,
}

impl<IMG: GenericImage> Carved<IMG> {
    fn remove_seam(&mut self, seam: &Vec<Pos>, dir: Direction) {
        seam.iter().for_each(|Pos(x, y)| {
            (*x..self.img.width() - 1).for_each(|i| {
                let p = self.img.get_pixel(i + 1, *y);
                self.img.put_pixel(i, *y, p);
            });
        });
        self.removed += 1;
    }
}

pub fn carve<IMG: GenericImage>(img: &mut IMG) {
    let dir = Direction::X;
    let _seam: Option<(Vec<Option<Pos>>, u32)> = dijkstra(
        &None,
        |maybe_pos: &Option<Pos>| -> Vec<_>{
            match maybe_pos {
                None =>
                    (0..img.width())
                        .map(|x| (Some(Pos(x, 0)), 0))
                        .collect(),
                Some(pos) =>
                    pos.successors(dir)
                        .filter(|pos| pos.in_image(img))
                        .map(|pos| (Some(pos), energy_fn(img, &pos)))
                        .collect(),
            }
        },
        |maybe_pos: &Option<Pos>| {
            maybe_pos.map_or(false, |Pos(_x, y)| y == img.height())
        },
    );
}

#[cfg(test)]
mod tests {
    #[test]
    fn it_works() {
        assert_eq!(2 + 2, 4);
    }
}
