use crate::max_pos;
use crate::pos::Pos;
use image::{GenericImageView, Pixel};
use num_traits::ToPrimitive;

pub(crate) fn energy_fn<IMG: GenericImageView>(img: &IMG, pos: Pos) -> u32 {
    let last_pos = max_pos(img);
    let [top, bottom, left, right] = pos.surrounding(last_pos);
    let top_px = img.get_pixel(top.0, top.1);
    let bottom_px = img.get_pixel(bottom.0, bottom.1);
    let left_px = img.get_pixel(left.0, left.1);
    let right_px = img.get_pixel(right.0, right.1);
    square_diff_px(top_px, bottom_px) +
        square_diff_px(left_px, right_px)
}

fn square_diff_px<P: Pixel>(p1: P, p2: P) -> u32 {
    let (ch1, ch2) = (p1.channels(), p2.channels());
    let count = <P as Pixel>::channel_count() as usize;
    let mut sum = 0;
    for i in 0..count {
        sum += square_diff(ch1[i], ch2[i]);
    }
    sum
}

#[inline]
fn square_diff<T: ToPrimitive>(a: T, b: T) -> u32 {
    let a = a.to_i32().unwrap_or(i32::max_value());
    let b = b.to_i32().unwrap_or(i32::max_value());
    let diff = a - b;
    (diff * diff) as u32
}