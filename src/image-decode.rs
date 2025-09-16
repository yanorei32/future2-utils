use std::fs::File;
use std::io::{Cursor, Read};
use std::path::PathBuf;

use binrw::BinRead;
use clap::Parser;
use image::{DynamicImage, ImageBuffer, Rgba};

use future_util::ImageFile;

#[derive(Parser)]
struct Cli {
    #[arg(short, long)]
    input: PathBuf,
}

#[derive(Debug, Eq, PartialEq)]
enum ColorMode {
    ColorPalette4bpc,
    ColorPalette8bpc,
    Rgb555_16bpc,
    Bgr888_24bpc,
    Bgra888_32bpc,
}

impl ColorMode {
    fn is_2834(&self) -> bool {
        match self {
            ColorMode::ColorPalette4bpc | ColorMode::ColorPalette8bpc => true,
            ColorMode::Bgr888_24bpc => true,
            _ => false,
        }
    }

    fn has_padding(&self) -> bool {
        match self {
            ColorMode::ColorPalette4bpc | ColorMode::ColorPalette8bpc => true,
            ColorMode::Bgr888_24bpc => true,
            _ => false,
        }
    }

    fn is_colorpalette(&self) -> bool {
        match self {
            ColorMode::ColorPalette4bpc | ColorMode::ColorPalette8bpc => true,
            _ => false,
        }
    }
}

fn main() {
    let cli = Cli::parse();
    let mut input = File::open(&cli.input).expect("Failed to open input file");
    let mut buffer = vec![];

    input
        .read_to_end(&mut buffer)
        .expect("Failed to read input file");

    let filesize = buffer.len();

    let mut buffer = Cursor::new(buffer);

    let header = ImageFile::read_le(&mut buffer).expect("Failed to read as BigFile");

    let color_mode = match header.bit_depth {
        4 => ColorMode::ColorPalette4bpc,
        8 => ColorMode::ColorPalette8bpc,
        16 => ColorMode::Rgb555_16bpc,
        24 => ColorMode::Bgr888_24bpc,
        32 => ColorMode::Bgra888_32bpc,
        _ => panic!("Unsupported bitdepth"),
    };

    if color_mode.is_colorpalette() {
        println!(
            "{}x{} {:?} ({} Colors)",
            header.width,
            header.height,
            color_mode,
            header.colorpalette.len()
        );
    } else {
        println!("{}x{} {:?}", header.width, header.height, color_mode,);
    }

    let estimate_constant_value = if color_mode.is_2834() { 2834 } else { 0 };

    if header.constant_2834_if_colorpalette_use_otherwise_0 != estimate_constant_value {
        println!(
            "WARNING: Unknown constant value is comming: {}",
            header.constant_2834_if_colorpalette_use_otherwise_0
        );
    }

    let image_start_at = 0x28 + 4 * header.colorpalette.len();

    if !color_mode.has_padding() {
        let body_len = match color_mode {
            ColorMode::Rgb555_16bpc => header.width * header.height * 2,
            ColorMode::Bgra888_32bpc => header.width * header.height * 4,
            _ => unreachable!(),
        } as usize;

        if body_len + image_start_at != filesize {
            panic!(
                "Invalid Filesize Expect: (header_len: {image_start_at} + body_len: {body_len}) = {}, Expect: {filesize}",
                body_len + image_start_at
            );
        }
    }

    buffer.set_position(image_start_at as u64);

    let mut rgba_buffer = vec![];

    let estimate_row_length = match color_mode {
        ColorMode::ColorPalette4bpc => (header.width + 1) / 2,
        ColorMode::ColorPalette8bpc => header.width,
        ColorMode::Rgb555_16bpc => header.width * 2,
        ColorMode::Bgr888_24bpc => header.width * 3,
        ColorMode::Bgra888_32bpc => header.width * 4,
    };

    // calculate row alignment (4-bytes)
    let estimate_row_length = ((estimate_row_length + 3) / 4) * 4;

    // Check bitmap image size
    let estimate_bitmap_image_size = match color_mode {
        ColorMode::ColorPalette8bpc | ColorMode::ColorPalette4bpc => 0,
        ColorMode::Bgr888_24bpc => 0,
        ColorMode::Rgb555_16bpc => estimate_row_length * header.height,
        ColorMode::Bgra888_32bpc => estimate_row_length * header.height,
    };

    if header.bitmap_image_size != estimate_bitmap_image_size {
        println!(
            "WARNING: Unknown bitmap image size is comming: {}",
            header.bitmap_image_size
        );
    }

    for _h in 0..header.height {
        let mut line_buffer = vec![0; estimate_row_length as usize];
        buffer.read_exact(&mut line_buffer).unwrap();

        match color_mode {
            ColorMode::ColorPalette4bpc => {
                for _w in 0..header.width {
                    let byte_index = _w / 2;
                    let mut color_index = line_buffer[byte_index as usize];

                    let is_high = _w & 1 == 0;

                    if is_high {
                        color_index >>= 4;
                    } else {
                        color_index &= 0xF;
                    }

                    let color = &header.colorpalette[color_index as usize];
                    rgba_buffer.extend_from_slice(&[color.r, color.g, color.b, 255 - color.a]);
                }
            }
            ColorMode::ColorPalette8bpc => {
                for _w in 0..header.width {
                    let color_index = line_buffer[_w as usize];
                    let color = &header.colorpalette[color_index as usize];
                    rgba_buffer.extend_from_slice(&[color.r, color.g, color.b, 255 - color.a]);
                }
            }
            ColorMode::Bgr888_24bpc => {
                for _w in 0..header.width {
                    let base = (_w * 3) as usize;

                    rgba_buffer.extend_from_slice(&[
                        // R
                        line_buffer[base + 2],
                        // G
                        line_buffer[base + 1],
                        // B
                        line_buffer[base + 0],
                        // A
                        0xFF,
                    ]);
                }
            }
            ColorMode::Rgb555_16bpc => {
                for _w in 0..header.width {
                    let base = (_w * 2) as usize;

                    let mut rgb555 =
                        u16::from_le_bytes([line_buffer[base + 0], line_buffer[base + 1]]);

                    let b = rgb555 & 0b11111;
                    rgb555 >>= 5;

                    let g = rgb555 & 0b11111;
                    rgb555 >>= 5;

                    let r = rgb555 & 0b11111;

                    let r = r * 33 / 4;
                    let g = g * 33 / 4;
                    let b = b * 33 / 4;

                    rgba_buffer.extend_from_slice(&[r as u8, g as u8, b as u8, 255 as u8]);
                }
            }
            ColorMode::Bgra888_32bpc => {
                for _w in 0..header.width {
                    let base = (_w * 4) as usize;

                    rgba_buffer.extend_from_slice(&[
                        // R
                        line_buffer[base + 2],
                        // G
                        line_buffer[base + 1],
                        // B
                        line_buffer[base + 0],
                        // A
                        255 - line_buffer[base + 3],
                    ]);
                }
            }
        }
    }

    let image_buffer =
        ImageBuffer::<Rgba<u8>, _>::from_raw(header.width, header.height, rgba_buffer).unwrap();

    let output_filename = &cli.input.with_file_name(format!(
        "{}.png",
        cli.input
            .file_name()
            .unwrap()
            .to_os_string()
            .to_string_lossy()
    ));

    DynamicImage::ImageRgba8(image_buffer)
        .flipv()
        .save(output_filename)
        .expect("Failed to save image");
}
