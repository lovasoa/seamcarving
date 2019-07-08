use image::{DynamicImage, GenericImageView};

use seamcarving::resize;
use std::path::{Path, PathBuf};

fn open_image() -> DynamicImage {
    let path: PathBuf = [
        Path::new(file!()).parent().unwrap(),
        Path::new("input.png")
    ].iter().collect();
    let img = image::open(path)
        .expect("input image not found");
    assert_eq!(img.dimensions(), (100, 100));
    img
}

#[test]
fn resize_100x100_to_1x1() {
    let img = open_image();
    let resized = resize(&img, 1, 1);
    assert_eq!(resized.dimensions(), (1, 1));
    assert_eq!(resized.into_raw(), vec![63, 67, 69, 255]);
}