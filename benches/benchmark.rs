use criterion::{Criterion, criterion_group, criterion_main};
use criterion::black_box;
use image::{GenericImageView, DynamicImage};
use std::path::{Path, PathBuf};
use std::time::Duration;


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

fn criterion_benchmark(c: &mut Criterion) {
    c.bench_function("100x100 to 95x95", |b| {
        let img = black_box(open_image());

        b.iter(||
            seamcarving::resize(&img, 95, 95)
        )
    });
}

criterion_group! {
    name = benches;
    config = Criterion::default()
                .sample_size(20)
                .nresamples(20)
                .warm_up_time(Duration::from_secs(1));
    targets = criterion_benchmark
}

criterion_main!(benches);
