/// Content-preserving image resizing
///
/// See: [resize]
use image::{GenericImageView, ImageBuffer, Pixel};

use crate::energy::energy_fn;
use crate::pos::Pos;
use crate::rotated::Rotated;
use crate::seam_finder::SeamFinder;
use crate::carved::Carved;

mod energy;
mod matrix;
mod pos;
mod rotated;
mod seam_finder;
mod carved;

/// Resize an image to a lower width and height,
/// using seam carving to avoid deforming the contents.
pub fn resize<IMG: GenericImageView>(
    img: &IMG,
    width: u32,
    height: u32,
) -> ImageBuffer<IMG::Pixel, Vec<<<IMG as GenericImageView>::Pixel as Pixel>::Subpixel>>
    where
        <IMG as GenericImageView>::Pixel: 'static,
{
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

struct Carvable<'a, IMG: GenericImageView>
    where
        <IMG as GenericImageView>::Pixel: 'static,
{
    carved: Carved<'a, IMG>,
    seam_finder: SeamFinder,
}

impl<'a, IMG: GenericImageView> Carvable<'a, IMG>
    where
        <IMG as GenericImageView>::Pixel: 'static,
{
    fn new(img: &'a IMG) -> Self {
        let carved = Carved::new(img);
        let seam_finder = SeamFinder::new(max_pos(img));
        Carvable {
            carved,
            seam_finder,
        }
    }
    fn remove_seam(&mut self) {
        let img = &self.carved;
        let seam = self.seam_finder.extract_seam(|p| energy_fn(img, p));
        self.carved.remove_seam(&seam);
    }
}


fn image_view_to_buffer<IMG: GenericImageView>(
    img: &IMG,
) -> ImageBuffer<IMG::Pixel, Vec<<<IMG as GenericImageView>::Pixel as Pixel>::Subpixel>>
    where
        <IMG as GenericImageView>::Pixel: 'static,
{
    let (w, h) = img.dimensions();
    ImageBuffer::from_fn(w, h, |x, y| img.get_pixel(x, y))
}


fn carve<IMG: GenericImageView>(img: &IMG, pixel_count: u32) -> Carved<IMG>
    where
        <IMG as GenericImageView>::Pixel: 'static,
{
    let mut carvable = Carvable::new(img);
    (0..pixel_count).for_each(|_| carvable.remove_seam());
    carvable.carved
}

#[cfg(test)]
mod tests {
    use image::{GrayImage, ImageBuffer, Luma};

    use crate::{energy_fn, Pos};

    #[test]
    fn energy_fn_correct() {
        let img = GrayImage::from_raw(3, 2, vec![3, 1, 4, 1, 5, 9]).unwrap();
        let energy = ImageBuffer::from_fn(3, 2, |x, y| Luma([energy_fn(&img, Pos(x, y))]));
        let expected = vec![
            (2 * 2 + 2 * 2),
            (1 * 1 + 4 * 4),
            (5 * 5 + 3 * 3),
            (2 * 2 + 4 * 4),
            (4 * 4 + 8 * 8),
            (5 * 5 + 4 * 4),
        ];
        assert_eq!(energy.into_raw(), expected);
    }
}
