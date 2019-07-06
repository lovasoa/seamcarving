use image::{GenericImage, GenericImageView, ImageBuffer, Pixel};
use pathfinding::prelude::dijkstra;
use std::ops::{Add, Sub, Mul};
use crate::rotated::Rotated;

mod rotated;

#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug)]
struct Pos(u32, u32);

impl Pos {
    #[inline(always)]
    fn before(self, max: Pos) -> bool {
        self.0 < max.0 && self.1 < max.1
    }
    fn successors(self) -> impl Iterator<Item=Pos> {
        let Pos(x, y) = self;
        std::iter::once(x.checked_sub(1))
            .flatten()
            .chain(std::iter::once(x))
            .chain(std::iter::once(x + 1))
            .map(move |x| Pos(x, y + 1))
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

impl From<(u32, u32)> for Pos {
    #[inline(always)]
    fn from((x, y): (u32, u32)) -> Self {
        Pos(x, y)
    }
}

fn energy_fn<IMG: GenericImageView>(img: &IMG, pos: Pos) -> u32 {
    use num_traits::cast::ToPrimitive;
    let last_pos = max_pos(img);
    std::iter::once(Pos(0, 1))
        .chain(std::iter::once(Pos(1, 0)))
        .map(|dir| -> u32 {
            let mut next = pos + dir;
            if !next.before(last_pos) { next = pos }
            let prev = pos - dir;
            let p1 = img.get_pixel(next.0, next.1);
            let p2 = img.get_pixel(prev.0, prev.1);
            p1.channels().iter().zip(p2.channels())
                .map(|(&a, &b)| {
                    let a = a.to_u32().unwrap_or(u32::max_value());
                    let b = b.to_u32().unwrap_or(u32::max_value());
                    let diff = if a > b { a - b } else { b - a };
                    diff * diff
                }).sum()
        }).sum()
}

/// An image with some vertical seams carved
struct Carved<'a, IMG: GenericImageView>
    where <IMG as GenericImageView>::Pixel: 'static {
    img: &'a IMG,
    removed: u32,
    pos_aliases: Vec<u32>,
}

impl<'a, IMG: GenericImageView> Carved<'a, IMG>
    where <IMG as GenericImageView>::Pixel: 'static {
    fn new(img: &'a IMG) -> Self {
        let (w, h) = img.dimensions();
        let pos_aliases = (0..w * h).map(|i| i % w).collect();
        Carved { img, removed: 0, pos_aliases }
    }
    fn remove_seam(&mut self, seam: &[Pos]) {
        let (w, _h) = self.img.dimensions();
        self.pos_aliases.chunks_exact_mut(w as usize)
            .zip(seam)
            .for_each(|(aliases, &Pos(x, _y))| {
                let end = &mut aliases[x as usize..];
                if !end.is_empty() { end.rotate_left(1) }
            });
        self.removed += 1;
    }
    /// Given a position in the carved image, return a position in the original
    #[inline(always)]
    fn transform_pos(&self, pos: Pos) -> Pos {
        let mut pos = pos;
        let w = self.img.width();
        pos.0 = self.pos_aliases[(pos.0 + pos.1 * w) as usize];
        pos
    }
}

fn image_view_to_buffer<IMG: GenericImageView>(img: &IMG)
                                               -> ImageBuffer<IMG::Pixel, Vec<<<IMG as GenericImageView>::Pixel as Pixel>::Subpixel>>
    where <IMG as GenericImageView>::Pixel: 'static
{
    let (w, h) = img.dimensions();
    ImageBuffer::from_fn(w, h, |x, y| {
        img.get_pixel(x, y)
    })
}

impl<'a, IMG: GenericImageView> GenericImageView for Carved<'a, IMG> {
    type Pixel = IMG::Pixel;
    type InnerImageView = IMG::InnerImageView;

    #[inline(always)]
    fn dimensions(&self) -> (u32, u32) {
        let (w, h) = self.img.dimensions();
        (w - self.removed, h)
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

/// Carve one vertical seam in the image
fn carve_one<IMG: GenericImageView>(carved: &mut Carved<IMG>) {
    let (w, h) = carved.dimensions();
    let end_coord = h - 1;
    let (seam, _cost): (Vec<Option<Pos>>, u32) = dijkstra(
        &None,
        |maybe_pos: &Option<Pos>| -> Vec<_>{
            match maybe_pos {
                None =>
                    (0..w).map(|x| (
                        Some(Pos(x, 0)),
                        energy_fn(carved, Pos(x, 0))
                    )).collect(),
                Some(pos) =>
                    pos.successors()
                        .filter(|Pos(x, y)| *x < w && *y < h)
                        .map(|pos| (Some(pos), energy_fn(carved, pos)))
                        .collect(),
            }
        },
        |maybe_pos: &Option<Pos>| {
            maybe_pos.map_or(false, |Pos(_x, y)| y == end_coord)
        },
    ).expect("No seam found. This is a bug in seamcarving");
    let seam: Vec<Pos> = seam.into_iter().skip(1).collect::<Option<_>>().unwrap();
    carved.remove_seam(&seam);
}

fn carve<IMG: GenericImageView>(
    img: &IMG,
    pixel_count: u32,
) -> Carved<IMG>
    where <IMG as GenericImageView>::Pixel: 'static {
    let mut carved = Carved::new(img);
    (0..pixel_count).for_each(|_| carve_one(&mut carved));
    carved
}

pub fn resize<IMG: GenericImage>(
    img: &IMG,
    width: u32,
    height: u32,
) -> ImageBuffer<IMG::Pixel, Vec<<<IMG as GenericImageView>::Pixel as Pixel>::Subpixel>>
    where <IMG as GenericImageView>::Pixel: 'static {
    let Pos(to_remove_x, to_remove_y) = max_pos(img) - Pos(width, height);
    let carved_x = carve(img, to_remove_x);
    let rotated = Rotated(&carved_x);
    let carved_y = carve(&rotated, to_remove_y);
    let rerotated = Rotated(&carved_y);
    image_view_to_buffer(&rerotated)
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
            Luma([energy_fn(&img, Pos(x, y))])
        });
        let expected = vec![
            (2 * 2 + 2 * 2), (1 * 1 + 4 * 4), (5 * 5 + 3 * 3),
            (2 * 2 + 4 * 4), (4 * 4 + 8 * 8), (5 * 5 + 4 * 4),
        ];
        assert_eq!(energy.into_raw(), expected);
    }

    fn pi_img_8_3() -> ImageBuffer<Luma<u8>, Vec<u8>> {
        GrayImage::from_raw(8, 3, vec![
            // 1  2  3  4  5  5  7
            3, 1, 4, 0, 0, 0, 1, 5, // 0
            9, 2, 6, 0, 0, 0, 5, 3, // 1
            5, 8, 0, 0, 0, 9, 7, 9, // 2
        ]).unwrap()
    }

    #[test]
    fn removes_the_right_vertical_seam() {
        let resized = resize(&pi_img_8_3(), 7, 3);
        assert_eq!(resized.dimensions(), (7, 3));
        assert_eq!(resized.into_raw(), vec![
            3, 1, 4, 0, 0, 1, 5,
            9, 2, 6, 0, 0, 5, 3,
            5, 8, 0, 0, 9, 7, 9,
        ]);
    }

    #[test]
    fn removes_the_right_horizontal_seam() {
        let rotated = image::imageops::rotate90(&pi_img_8_3());
        let resized_rotated = resize(&rotated, 3, 7);
        assert_eq!(resized_rotated.dimensions(), (3, 7));
        assert_eq!(resized_rotated.into_raw(), vec![
            5, 9, 3,
            8, 2, 1,
            0, 6, 4,
            0, 0, 0,
            9, 0, 0,
            7, 5, 1,
            9, 3, 5
        ]);
    }

    #[test]
    fn updates_energy() {
        let img = GrayImage::from_raw(8, 3, vec![
            7, 9, 9, 0, 0, 0, 9, 5,
            8, 9, 9, 0, 0, 0, 9, 3,
            8, 9, 0, 0, 0, 9, 7, 9,
        ]).unwrap();
        let resized = resize(&img, 6, 3);
        assert_eq!(resized.dimensions(), (6, 3));
        assert_eq!(resized.into_raw(), vec![
            7, 9, 0, 0, 9, 5,
            8, 9, 0, 0, 9, 3,
            9, 0, 0, 9, 7, 9
        ]);
    }
}
