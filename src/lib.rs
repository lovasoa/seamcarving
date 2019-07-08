/// Content-preserving image resizing
///
/// See: [resize]

use image::{GenericImageView, ImageBuffer, Pixel};
use pathfinding::prelude::dijkstra;

use crate::energy::energy_fn;
use crate::matrix::Matrix;
use crate::pos::Pos;
use crate::rotated::Rotated;
use smallvec::SmallVec;

mod rotated;
mod matrix;
mod pos;
mod energy;


/// Resize an image to a lower width and height,
/// using seam carving to avoid deforming the contents.
pub fn resize<IMG: GenericImageView>(
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

fn max_pos<IMG: GenericImageView>(img: &IMG) -> Pos {
    Pos(img.width(), img.height())
}

/// An image with some vertical seams carved
struct Carved<'a, IMG: GenericImageView>
    where <IMG as GenericImageView>::Pixel: 'static {
    img: &'a IMG,
    removed: u32,
    // pos_aliases is a matrix such as img[x,y] = self[pos_aliases[x,y],y]
    pos_aliases: Matrix<u32>,
    energy_cache: Matrix<Option<u32>>, // The energy is computed lazily, hence the Option
}

impl<'a, IMG: GenericImageView> Carved<'a, IMG>
    where <IMG as GenericImageView>::Pixel: 'static {
    fn new(img: &'a IMG) -> Self {
        let size = max_pos(img);
        let pos_aliases = Matrix::from_fn(size, |x, _y| x as u32);
        let energy = Matrix::from_fn(size, |_x, _y| None);
        Carved { img, removed: 0, pos_aliases, energy_cache: energy }
    }
    fn remove_seam(&mut self, seam: &[Pos]) {
        let last = max_pos(self);
        seam.iter().for_each(|&pos| { // invalidate the energy cache around the seam
            pos.surrounding().iter()
                .filter(|&p| p.before(last))
                .for_each(|&p| { self.energy_cache[p] = None; })
        });
        self.pos_aliases.remove_seam(seam);
        self.energy_cache.remove_seam(seam);
        self.removed += 1;
    }
    fn energy(&mut self, pos: Pos) -> u32 {
        self.energy_cache[pos].unwrap_or_else(|| {
            let computed = energy_fn(self, pos);
            self.energy_cache[pos] = Some(computed);
            computed
        })
    }
    /// Given a position in the carved image, return a position in the original
    #[inline(always)]
    fn transform_pos(&self, pos: Pos) -> Pos {
        let mut pos = pos;
        pos.0 = self.pos_aliases[pos];
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
    // Avoid checking for the empty image case in the hot path
    let seam: Vec<Pos> = if h == 0 { vec![] } else {
        let (seam, _cost): (Vec<Option<Pos>>, u32) = dijkstra(
            &None,
            |maybe_pos: &Option<Pos>| -> SmallVec<[(Option<Pos>, u32); 3]>{
                match maybe_pos {
                    None => {
                        let mut v = SmallVec::with_capacity(w as usize);
                        v.extend((0..w).map(|x| (
                            Some(Pos(x, 0)),
                            carved.energy(Pos(x, 0))
                        )));
                        v
                    }
                    Some(pos) =>
                        pos.successors(w, h)
                            .map(|pos| (Some(pos), carved.energy(pos)))
                            .collect(),
                }
            },
            |maybe_pos: &Option<Pos>| {
                maybe_pos.map_or(false, |Pos(_x, y)| y + 1 == h)
            },
        ).expect("No seam found. This is a bug in seamcarving");
        seam.into_iter().skip(1).collect::<Option<_>>().expect("empty seam. This is a bug")
    };
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

#[cfg(test)]
mod tests {
    use image::{GrayImage, ImageBuffer, Luma};

    use crate::{energy_fn, Pos};

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
}
