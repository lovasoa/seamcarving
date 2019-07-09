//! ## Content-preserving image resizing in rust
//!
//! The main function of this crate is [resize]:
//! it takes an image, and removes horizontal and vertical seams
//! until it fits a given size.
//!
use image::{GenericImageView, ImageBuffer, Pixel};

pub use crate::carved::Carved;
use crate::energy::energy_fn;
use crate::pos::Pos;
pub use crate::rotated::Rotated;
use crate::seam_finder::SeamFinder;

mod carved;
mod energy;
mod matrix;
mod pos;
mod rotated;
mod seam_finder;

/// Resizes an image to a lower width and height,
/// using seam carving to avoid deforming the contents.
///
/// This works by removing horizontal and then vertical seams
/// until both the width and the height of the image
/// are inferior to the given dimensions.
///
/// If the image is already smaller than the given dimensions,
/// then the returned image is identical to the input.
///
/// ```no_run
/// let img = image::open("./my_image.jpg").unwrap();
/// let resized = seamcarving::resize(&img, 100, 100); // Creates a 100x100 version of the image
/// resized.save("./resized.jpg");
/// ```
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
    let re_rotated = Rotated(&carved_y);
    image_view_to_buffer(&re_rotated)
}

fn max_pos<IMG: GenericImageView>(img: &IMG) -> Pos {
    Pos(img.width(), img.height())
}

/// A structure that allows removing vertical seams of content
/// from an image
pub struct Carvable<'a, IMG: GenericImageView>
where
    <IMG as GenericImageView>::Pixel: 'a,
{
    carved: Carved<'a, IMG>,
    seam_finder: SeamFinder,
}

impl<'a, IMG: GenericImageView> Carvable<'a, IMG> {
    /// Creates a new proxy object that will allow reducing an image width.
    /// Notice that it does not take a mutable pointer.
    /// The underlying image itself is untouched.
    pub fn new(img: &'a IMG) -> Self {
        let carved = Carved::new(img);
        let seam_finder = SeamFinder::new(max_pos(img));
        Carvable {
            carved,
            seam_finder,
        }
    }
    /// Removes a vertical seam from the image,
    /// diminishing its width by 1.
    pub fn remove_seam(&mut self) {
        let img = &self.carved;
        let seam = self.seam_finder.extract_seam(|p| energy_fn(img, p));
        self.carved.remove_seam(&seam);
    }
    /// Get the resulting carved image
    pub fn result(&self) -> &Carved<'a, IMG> {
        &self.carved
    }
}

/// Converts [GenericImageView](GenericImageView)
/// to an [ImageBuffer](ImageBuffer)
pub fn image_view_to_buffer<IMG: GenericImageView>(
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
