use crate::matrix::Matrix;
use crate::max_pos;
use crate::pos::Pos;
use image::GenericImageView;

/// An image with some vertical seams carved.
/// If you want to save this image or otherwise manipulate it,
/// you can convert it to a [ImageBuffer](image::ImageBuffer)
/// using [image_view_to_buffer](crate::image_view_to_buffer)
pub struct Carved<'a, IMG: GenericImageView> {
    img: &'a IMG,
    removed: u32,
    // pos_aliases is a matrix such as img[x,y] = self[pos_aliases[x,y],y]
    pos_aliases: Matrix<u32>,
}

impl<'a, IMG: GenericImageView> Carved<'a, IMG> {
    pub(crate) fn new(img: &'a IMG) -> Self {
        let size = max_pos(img);
        let pos_aliases = Matrix::from_fn(size, |x, _y| x as u32);
        Carved {
            img,
            removed: 0,
            pos_aliases,
        }
    }
    pub(crate) fn remove_seam(&mut self, seam: &[Pos]) {
        self.pos_aliases.remove_seam(seam);
        self.removed += 1;
    }
    /// Given a position in the carved image, return a position in the original
    #[inline(always)]
    fn transform_pos(&self, pos: Pos) -> Pos {
        let mut pos = pos;
        pos.0 = self.pos_aliases[pos];
        pos
    }
}

impl<'a, IMG: GenericImageView> GenericImageView for Carved<'a, IMG>
where
    <IMG as GenericImageView>::Pixel: 'a,
{
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
