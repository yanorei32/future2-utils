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

#[derive(Eq, PartialEq)]
enum ColorMode {
    ColorPalette4bpc,
    ColorPalette8bpc,
    Rgb555_16bpc,
    Rgb888_24bpc,
    Rgb888_32bpc,
}

impl ColorMode {
    fn is_2834(&self) -> bool {
        match self {
            ColorMode::ColorPalette4bpc | ColorMode::ColorPalette8bpc => true,
            ColorMode::Rgb888_24bpc => true,
            _ => false,
        }
    }

    fn has_padding(&self) -> bool {
        match self {
            ColorMode::ColorPalette4bpc | ColorMode::ColorPalette8bpc => true,
            ColorMode::Rgb888_24bpc => true,
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
        24 => ColorMode::Rgb888_24bpc,
        32 => ColorMode::Rgb888_32bpc,
        _ => panic!("Unsupported bitdepth"),
    };

    println!("Width: {}", header.width);
    println!("Height: {}", header.height);

    let estimate_constant_value = if color_mode.is_2834() { 2834 } else { 0 };

    if header.constant_2834_if_colorpalette_use_otherwise_0 != estimate_constant_value {
        println!(
            "WARNING: Unknown constant value is comming: {}",
            header.constant_2834_if_colorpalette_use_otherwise_0
        );
    }

    let estimate_bitmap_image_size = match color_mode {
        ColorMode::ColorPalette8bpc | ColorMode::ColorPalette4bpc => 0,
        ColorMode::Rgb888_24bpc => 0,
        ColorMode::Rgb555_16bpc => header.width * header.height * 2,
        ColorMode::Rgb888_32bpc => header.width * header.height * 4,
    };

    if header.bitmap_image_size != estimate_bitmap_image_size {
        println!(
            "WARNING: Unknown bitmap image size is comming: {}",
            header.bitmap_image_size
        );
    }

    let image_start_at = 0x28 + 4 * header.colorpalette.len();

    if !color_mode.has_padding() {
        let body_len = match color_mode {
            ColorMode::Rgb555_16bpc => header.width * header.height * 2,
            ColorMode::Rgb888_32bpc => header.width * header.height * 4,
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

    let consume_count = match color_mode {
        ColorMode::ColorPalette4bpc => (((header.width + 1) * 2) / 2 * header.height + 1) / 2,
        ColorMode::Rgb888_24bpc => (header.width * header.height + 1) / 2,
        _ => header.width * header.height,
    };

    for _n in 0..consume_count {
        match color_mode {
            ColorMode::ColorPalette4bpc => {
                let mut color_indexes = [0];
                buffer.read_exact(&mut color_indexes).unwrap();

                let color = &header.colorpalette[(color_indexes[0] >> 4) as usize];
                rgba_buffer.extend_from_slice(&[color.r, color.g, color.b, 255 - color.a]);

                let is_eol = _n != 0 && ((_n + 1) % ((header.width + 1) / 2)) == 0;

                if !is_eol || header.width % 2 == 0 {
                    let color = &header.colorpalette[(color_indexes[0] & 0x0F) as usize];
                    rgba_buffer.extend_from_slice(&[color.r, color.g, color.b, 255 - color.a]);
                }

                if is_eol {
                    println!("pos: {:x}", buffer.position());

                    if buffer.position() & 0b11 != 0 {
                        println!("newpos: {:x}", ((buffer.position() & !0b11) + 4));
                        buffer.set_position((buffer.position() & !0b11) + 4);
                    }
                }
            }
            ColorMode::Rgb888_24bpc => {
                let mut rgbrgbaa = [0; 8];
                buffer.read_exact(&mut rgbrgbaa).unwrap();
                rgba_buffer.extend_from_slice(&[rgbrgbaa[0], rgbrgbaa[1], rgbrgbaa[2], 0xFF]);
                rgba_buffer.extend_from_slice(&[rgbrgbaa[3], rgbrgbaa[4], rgbrgbaa[5], 0xFF]);
            }
            ColorMode::ColorPalette8bpc => {
                let mut color_index = [0];

                buffer.read_exact(&mut color_index).unwrap();

                let color = &header.colorpalette[color_index[0] as usize];
                rgba_buffer.extend_from_slice(&[color.r, color.g, color.b, 255 - color.a]);

                if _n != 0 && (_n + 1) % header.width == 0 {
                    println!("pos: {:x}", buffer.position());

                    if buffer.position() & 0b11 != 0 {
                        println!("newpos: {:x}", ((buffer.position() & !0b11) + 4));
                        buffer.set_position((buffer.position() & !0b11) + 4);
                    }
                }
            }
            ColorMode::Rgb555_16bpc => {
                let mut rgb555 = [0; 2];
                buffer.read_exact(&mut rgb555).unwrap();

                let mut rgb555 = u16::from_le_bytes(rgb555);
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
            ColorMode::Rgb888_32bpc => {
                let mut rgba = [0; 4];
                buffer.read_exact(&mut rgba).unwrap();
                rgba_buffer.extend_from_slice(&rgba);
            }
        }
    }

    println!("{}", rgba_buffer.len());

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
