use std::fs::{File, OpenOptions};
use std::io::{Cursor, Read, Write};
use std::path::PathBuf;

use binrw::BinRead;
use clap::Parser;

use future2_util::S10StrFile;

#[derive(Parser)]
struct Cli {
    #[arg(short, long)]
    input: PathBuf,
}

fn main() {
    let cli = Cli::parse();
    let mut input = File::open(&cli.input).expect("Failed to open input file");

    let mut buffer = vec![];

    input
        .read_to_end(&mut buffer)
        .expect("Failed to read input file");

    let mut buffer = Cursor::new(buffer);

    let header = S10StrFile::read_le(&mut buffer).expect("Failed to read as BigFile");

    println!("File Contents:");

    for (i, file_descriptor) in header.file_descriptors.iter().enumerate() {
        println!(
            " - Size: {} at Addr: 0x{:x}",
            file_descriptor.size, file_descriptor.start_at
        );

        buffer.set_position(file_descriptor.start_at as u64);

        let mut file_content = vec![0; file_descriptor.size as usize];

        buffer
            .read_exact(&mut file_content)
            .expect("Failed to strip file");

        let output_filename = &cli.input.with_file_name(format!(
            "{}.{i}.mp3",
            cli.input
                .file_name()
                .unwrap()
                .to_os_string()
                .to_string_lossy()
        ));

        let mut output = OpenOptions::new()
            .write(true)
            .create(true)
            .truncate(true)
            .open(output_filename)
            .expect("Failed to create output file");

        output
            .write_all(&file_content)
            .expect("Failed to write output file");
    }
}
