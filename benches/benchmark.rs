use std::path::{Path, PathBuf};
use std::time::Duration;

use criterion::{Criterion, criterion_group, criterion_main, ParameterizedBenchmark};
use criterion::black_box;
use image::{DynamicImage, GenericImageView, GrayImage, Luma};

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

/// Gray image to use in benchmarks. This is neither noise nor
/// similar to natural images - it's just a convenience method
/// to produce an image that's not constant.
pub fn gray_bench_image(width: u32, height: u32) -> GrayImage {
    let mut image = GrayImage::new(width, height);
    for y in 0..image.height() {
        for x in 0..image.width() {
            let intensity = (x % 7 + y % 6) as u8;
            image.put_pixel(x, y, Luma([intensity]));
        }
    }
    image
}


fn criterion_benchmark(c: &mut Criterion) {
    c.bench_function("100x100 to 95x95", |b| {
        let img = black_box(open_image());

        b.iter(||
            seamcarving::resize(&img, 95, 95)
        )
    });

    let (w, h) = (160, 90);
    c.bench(
        &format!("{w}x{h} to ({w}-i)x{h}", w = w, h = h),
        ParameterizedBenchmark::new(
            "seamcarving",
            move |b, &i| {
                let gray_img = black_box(gray_bench_image(w, h));
                b.iter(||
                    seamcarving::resize(&gray_img, w - i, h))
            },
            vec![w / 16, w / 8, w / 6, w / 4, w / 2, 2 * w / 3],
        ).with_function(
            "imageproc",
            move |b, &i| {
                let gray_img = black_box(gray_bench_image(w, h));
                b.iter(||
                    imageproc::seam_carving::shrink_width(&gray_img, w - i))
            },
        ),
    );
}

criterion_group! {
    name = benches;
    config = Criterion::default()
                .sample_size(25)
                .nresamples(25)
                .warm_up_time(Duration::from_secs(1));
    targets = criterion_benchmark
}

criterion_main!(benches);
