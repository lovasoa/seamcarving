use image::{GrayImage, ImageBuffer, Luma};

use seamcarving::resize;

fn pi_img_8_3() -> ImageBuffer<Luma<u8>, Vec<u8>> {
    GrayImage::from_raw(
        8,
        3,
        vec![
            // 1  2  3  4  5  6  7
            3, 1, 4, 0, 0, 0, 1, 5, // 0
            9, 2, 6, 0, 0, 0, 5, 3, // 1
            5, 8, 0, 0, 0, 9, 7, 9, // 2
        ],
    )
        .unwrap()
}

#[test]
fn removes_the_right_vertical_seam() {
    let resized = resize(&pi_img_8_3(), 7, 3);
    assert_eq!(resized.dimensions(), (7, 3));
    assert_eq!(
        resized.into_raw(),
        vec![
            // 1  2  3  4  5  6
            3, 1, 4, 0, 0, 1, 5, // 0
            9, 2, 6, 0, 0, 5, 3, // 1
            5, 8, 0, 0, 9, 7, 9, // 2
        ]
    );
}

#[test]
fn removes_the_right_horizontal_seam() {
    let rotated = image::imageops::rotate90(&pi_img_8_3());
    let resized_rotated = resize(&rotated, 3, 7);
    assert_eq!(resized_rotated.dimensions(), (3, 7));
    assert_eq!(
        resized_rotated.into_raw(),
        vec![
            // 1  2
            5, 9, 3, // 0
            8, 2, 1, // 1
            0, 6, 4, // 2
            0, 0, 0, // 3
            9, 0, 0, // 4
            7, 5, 1, // 5
            9, 3, 5, // 6
        ]
    );
}

#[test]
fn remove_two_seams() {
    let img = GrayImage::from_raw(
        8,
        3,
        vec![
            // 1  2  3  4  5  5  7
            7, 9, 9, 0, 0, 0, 9, 5, // 0
            8, 9, 9, 0, 0, 0, 9, 3, // 1
            8, 9, 0, 0, 0, 9, 7, 9, // 2
        ],
    ).expect("Unable to create test image");
    let resized = resize(&img, 6, 3);
    assert_eq!(resized.dimensions(), (6, 3));
    assert_eq!(
        resized.into_raw(),
        vec![
            // 1  2  3  4  5
            7, 9, 0, 0, 9, 5, // 0
            8, 9, 0, 0, 9, 3, // 1
            9, 0, 0, 9, 7, 9  // 2
        ]
    )
}

#[test]
fn single_pixel() {
    let img = GrayImage::from_raw(1, 1, vec![42]).unwrap();
    let resized = resize(&img, 0, 0);
    assert_eq!(resized.dimensions(), (0, 0));
    assert_eq!(resized.into_raw(), vec![]);
}
