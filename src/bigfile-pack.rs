use std::fs::{File, OpenOptions};
use std::io::{BufReader, Read, Write};
use std::path::{Path, PathBuf};

use binrw::BinWrite;
use clap::Parser;

use future2_utils::{BigFile, BigFileDescriptor};

#[derive(Parser)]
struct Cli {
    #[arg(short, long)]
    inputs: Vec<PathBuf>,

    #[arg(short, long, default_value_t = 0x29)]
    encrypt_key: u8,

    #[arg(long, default_value_t = false)]
    raw: bool,

    #[arg(short, long)]
    output: PathBuf,
}

fn read_file(p: &Path, raw: bool) -> Vec<u8> {
    let mut buffer = vec![];
    let mut input = BufReader::new(File::open(p).expect("Failed to open input file"));

    const BITMAP_FILE_HEADER_SIZE: i64 = 14;

    if raw == false {
        input
            .seek_relative(BITMAP_FILE_HEADER_SIZE)
            .expect("Invalid input BMP size");
    }

    input
        .read_to_end(&mut buffer)
        .expect("Failed to read input file");

    buffer
}

fn main() {
    let cli = Cli::parse();
    let mut files: Vec<_> = cli.inputs.iter().map(|path| read_file(path, cli.raw)).collect();

    // encrypt
    files.iter_mut().for_each(|file| {
        file.iter_mut().for_each(|value| {
            *value ^= cli.encrypt_key;
        });
    });

    let header_size = 4 + 1 + files.len() * 8;

    let start_ats: Vec<_> = files
        .iter()
        .scan(header_size, |cum, data| {
            let current = *cum;
            *cum += data.len();
            Some(current as u32)
        })
        .collect();

    let file_descriptors: Vec<_> = files
        .iter()
        .zip(start_ats)
        .map(|(file, start_at)| BigFileDescriptor {
            start_at,
            size: file.len() as u32,
        })
        .collect();

    let bigfile = BigFile {
        encrypt_key: cli.encrypt_key,
        file_descriptors,
    };

    let mut output = OpenOptions::new()
        .write(true)
        .create(true)
        .truncate(true)
        .open(&cli.output)
        .expect("Failed to create output file");

    bigfile
        .write_le(&mut output)
        .expect("Failed to write header");

    for file in files {
        output.write_all(&file).expect("Failed to write body");
    }
}
