use std::env;
use std::path::PathBuf;
use std::error::Error;

fn main() -> Result<(), Box<Error>> {
    let args: Vec<String> = env::args().collect();
    if args.len() < 4 {
        return Err("Usage: resize /path/to/image.jpg desired_width desired_height".into());
    }
    let image_path: PathBuf = args[1].parse()?;
    let width: u32 = args[2].parse()?;
    let height: u32 = args[3].parse()?;

    let input_image = image::open(&image_path)?;
    let resized = seamcarving::resize(&input_image, width, height);
    let output_path = image_path.with_file_name(
        format!("{}_resized.{}",
                image_path.file_stem().expect("invalid file name").to_string_lossy(),
                image_path.extension().expect("invalid file extension").to_string_lossy()
        )
    );
    resized.save(&output_path)?;
    println!("Resized image successfully written to {}", output_path.to_string_lossy());
    Ok(())
}