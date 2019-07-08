use crate::max_pos;
use crate::pos::Pos;
use image::{GenericImageView, Pixel};

pub(crate) fn energy_fn<IMG: GenericImageView>(img: &IMG, pos: Pos) -> u32 {
    use num_traits::cast::ToPrimitive;
    let [top, bottom, left, right] = pos.surrounding();
    let last_pos = max_pos(img);
    [(top, bottom), (left, right)]
        .iter()
        .map(|&(prev, next)| -> u32 {
            let next = if next.before(last_pos) { next } else { pos };
            let p1 = img.get_pixel(next.0, next.1);
            let p2 = img.get_pixel(prev.0, prev.1);
            p1.channels()
                .iter()
                .zip(p2.channels())
                .map(|(&a, &b)| {
                    let a = a.to_u32().unwrap_or(u32::max_value());
                    let b = b.to_u32().unwrap_or(u32::max_value());
                    let diff = if a > b { a - b } else { b - a };
                    diff * diff
                })
                .sum()
        })
        .sum()
}
