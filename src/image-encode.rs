use std::fs::OpenOptions;
use std::io::{BufWriter, Write};
use std::path::PathBuf;

use binrw::BinWrite;
use clap::Parser;
use image::ImageReader;

use future2_utils::BitmapInfoHeader;

#[derive(Parser)]
struct Cli {
    #[arg(short, long)]
    input: PathBuf,
}

fn main() {
    let cli = Cli::parse();

    let img = ImageReader::open(&cli.input)
        .expect("Failed to open")
        .decode()
        .expect("Failed to decode");

    let output_filename = &cli.input.with_file_name(format!(
        "{}.data",
        cli.input
            .file_name()
            .unwrap()
            .to_os_string()
            .to_string_lossy()
    ));

    let img = img.flipv().to_rgba8();

    let output = OpenOptions::new()
        .write(true)
        .create(true)
        .truncate(true)
        .open(output_filename)
        .expect("Failed to create output file");

    let mut output = BufWriter::new(output);

    let image = BitmapInfoHeader {
        bit_count: 32,
        compression: 0,
        size_image: img.width() * img.height() * 4,
        colorpalette: vec![],
        x_pels_per_meter: 0,
        y_pels_per_meter: 0,
        width: img.width(),
        height: img.height(),
    };

    image.write_le(&mut output).expect("Failed to write header");

    for pixel in img.pixels() {
        output
            .write_all(&[pixel.0[2], pixel.0[1], pixel.0[0], pixel.0[3]])
            .expect("Failed to write pixel");
    }
}
