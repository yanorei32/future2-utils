use std::fs::{File, OpenOptions};
use std::io::{BufReader, BufWriter, Seek, SeekFrom, copy};
use std::path::PathBuf;

use binrw::{BinRead, BinWrite};
use clap::Parser;

use future2_utils::{BitmapFileHeader, BitmapInfoHeader};

#[derive(Parser)]
struct Cli {
    #[arg(short, long)]
    input: PathBuf,

    #[arg(short, long)]
    output: PathBuf,
}

fn main() {
    let cli = Cli::parse();
    let dib_filesize = std::fs::metadata(&cli.input)
        .expect("Failed to get input CLI")
        .len();

    let mut input = BufReader::new(File::open(&cli.input).expect("Failed to open input file"));

    const BITMAP_FILE_HEADER_SIZE: u32 = 14;
    const BITMAP_INFO_HEADER_SIZE: u32 = 40;

    let size = BITMAP_FILE_HEADER_SIZE + dib_filesize as u32;

    let header = BitmapInfoHeader::read_le(&mut input).expect("Failed to read as DIB");

    input.seek(SeekFrom::Start(0)).unwrap();

    let off_bits =
        BITMAP_FILE_HEADER_SIZE + BITMAP_INFO_HEADER_SIZE + header.colorpalette.len() as u32 * 4;

    let mut output = BufWriter::new(
        OpenOptions::new()
            .write(true)
            .create(true)
            .truncate(true)
            .open(&cli.output)
            .expect("Failed to create output file"),
    );

    BitmapFileHeader { size, off_bits }
        .write_le(&mut output)
        .expect("Failed to write BMP header");

    copy(&mut input, &mut output).expect("Failed to copy to output");
}
