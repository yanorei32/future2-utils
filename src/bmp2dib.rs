use std::fs::{File, OpenOptions};
use std::io::{BufReader, BufWriter, copy};
use std::path::PathBuf;

use clap::Parser;

#[derive(Parser)]
struct Cli {
    #[arg(short, long)]
    input: PathBuf,

    #[arg(short, long)]
    output: PathBuf,
}

fn main() {
    let cli = Cli::parse();
    let mut input = BufReader::new(File::open(&cli.input).expect("Failed to open input file"));

    const BITMAP_FILE_HEADER_SIZE: i64 = 14;

    input
        .seek_relative(BITMAP_FILE_HEADER_SIZE)
        .expect("Invalid input BMP size");

    let mut output = BufWriter::new(
        OpenOptions::new()
            .write(true)
            .create(true)
            .truncate(true)
            .open(&cli.output)
            .expect("Failed to create output file"),
    );

    copy(&mut input, &mut output).expect("Failed to copy to output");
}
