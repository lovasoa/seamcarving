use image::GenericImageView;

/// Represents an image with the x and y coordinates reversed
/// In geometric terms, the operation is the combination of a 90° rotation and a symmetry along the Y axis
/// This operation is idempotent: Rotated(Rotated(img)) = img
pub(crate) struct Rotated<'a, IMG: GenericImageView>(pub &'a IMG);

impl<'a, IMG: GenericImageView> GenericImageView for Rotated<'a, IMG> {
    type Pixel = IMG::Pixel;
    type InnerImageView = IMG::InnerImageView;

    #[inline(always)]
    fn dimensions(&self) -> (u32, u32) {
        let (h, w) = self.0.dimensions();
        (w, h)
    }

    #[inline(always)]
    fn bounds(&self) -> (u32, u32, u32, u32) {
        let (w, h) = self.dimensions();
        (0, 0, w, h)
    }

    #[inline(always)]
    fn get_pixel(&self, x: u32, y: u32) -> Self::Pixel {
        self.0.get_pixel(y, x)
    }

    fn inner(&self) -> &Self::InnerImageView {
        self.0.inner()
    }
}
