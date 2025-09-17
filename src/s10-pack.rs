use std::fs::{File, OpenOptions};
use std::io::{Read, Write};
use std::path::{Path, PathBuf};

use binrw::BinWrite;
use clap::Parser;
use utf16string::{LE, WString};

use future2_util::{S10StrFile, S10StrFileDescriptor};

#[derive(Parser)]
struct Cli {
    #[arg(short, long)]
    inputs: Vec<PathBuf>,

    #[arg(short, long)]
    output: PathBuf,
}

fn read_file(p: &Path) -> Vec<u8> {
    let mut buffer = vec![];
    let mut input = File::open(p).expect("Failed to open input file");
    input
        .read_to_end(&mut buffer)
        .expect("Failed to read input file");
    buffer
}

fn main() {
    let cli = Cli::parse();
    let files: Vec<_> = cli.inputs.iter().map(|path| read_file(path)).collect();

    let header_size = 4 + files.len() * (520 + 4 + 4);

    let start_ats: Vec<_> = files
        .iter()
        .scan(header_size, |cum, data| {
            let current = *cum;
            *cum += data.len();
            Some(current as u32)
        })
        .collect();

    let titles: Vec<_> = cli.inputs.iter().map(|path| {
        let title = path.file_stem().unwrap();
        let title: &str = &title.to_string_lossy();
        let title = WString::<LE>::from(title);
        let title_buffer = title.into_bytes();

        if title_buffer.len() > 520 {
            panic!("Filename too long");
        }

        let mut buffer_with_pad = [0; 520];

        buffer_with_pad.iter_mut().zip(title_buffer).for_each(|(dest, src)| {
            *dest = src;
        });

        buffer_with_pad
    }).collect();

    let file_descriptors: Vec<_> = files
        .iter()
        .zip(start_ats)
        .zip(titles)
        .map(|((file, start_at), title_buffer)| S10StrFileDescriptor {
            title_u16le: title_buffer,
            start_at,
            size: file.len() as u32,
        })
        .collect();

    let s10file = S10StrFile {
        file_descriptors,
    };

    let mut output = OpenOptions::new()
        .write(true)
        .create(true)
        .truncate(true)
        .open(&cli.output)
        .expect("Failed to create output file");

    s10file
        .write_le(&mut output)
        .expect("Failed to write header");

    for file in files {
        output.write_all(&file).expect("Failed to write body");
    }
}
