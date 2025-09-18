use std::fs::{File, OpenOptions};
use std::io::{Cursor, Read, Write};
use std::path::PathBuf;

use binrw::BinRead;
use clap::Parser;
use utf16string::WString;

use future2_utils::S10StrFile;

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

    let input_filename = cli.input.file_name().unwrap().to_os_string();
    let input_filename = input_filename.to_string_lossy();
    let output_directory = format!("{input_filename}.extract");
    let output_directory = cli.input.with_file_name(output_directory);

    if !std::fs::exists(&output_directory).expect("Failed to check existency of output directory") {
        std::fs::create_dir(&output_directory).expect("Failed to create output dir");
    }

    for (i, file_descriptor) in header.file_descriptors.iter().enumerate() {
        let title = WString::from_utf16le(file_descriptor.title_u16le.to_vec())
            .expect("Invalid UTF-16LE title");

        let title = title.to_string();
        let title = title.trim_end_matches('\0');

        println!(
            " - Size: {} at Addr: 0x{:x} Title: {title}",
            file_descriptor.size, file_descriptor.start_at
        );

        buffer.set_position(file_descriptor.start_at as u64);

        let mut file_content = vec![0; file_descriptor.size as usize];

        buffer
            .read_exact(&mut file_content)
            .expect("Failed to strip file");

        let output_filename = output_directory.join(format!("{}. {title}.mp3", i + 1));

        let mut output = OpenOptions::new()
            .write(true)
            .create(true)
            .truncate(true)
            .open(&output_filename)
            .expect("Failed to create output file");

        output
            .write_all(&file_content)
            .expect("Failed to write output file");
    }
}
