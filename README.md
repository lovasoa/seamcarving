# seamcarving

A rust library for
**content-aware image resizing** using [seam carving](https://en.wikipedia.org/wiki/Seam_carving).

## How to use

Open an image using the [**image** crate](https://crates.io/crates/image),
then use the `resize` function from this create to create a smaller version
of the image, while preserving its contents.

```rust
let img = image::open("input.jpg")?;
let (width, height) = img.dimensions();
let resized = seamcarving::resize(&img, width/2, height);
resized.save("resized.jpg")?;
```

#### Detailed code example
 - [resize.rs](./examples/resize.rs) : command-line image resizing
 
## Results

Original | Resized
--- | ---
![waterfall original](./examples/waterfall.png) | ![waterfall resized with liquid rescaling](./examples/waterfall_resized.png) 
![butterfly original](./examples/butterfly.png) | ![butterfly resized with liquid rescaling](./examples/butterfly_resized.png) 