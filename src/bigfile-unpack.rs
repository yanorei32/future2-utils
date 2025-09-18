use std::fs::{File, OpenOptions};
use std::io::{BufReader, Cursor, Read, Seek, SeekFrom, Write};
use std::path::PathBuf;

use binrw::{BinRead, BinWrite};
use clap::Parser;

use future2_utils::{BigFile, BitmapFileHeader, BitmapInfoHeader};

const BITMAP_FILE_HEADER_SIZE: u32 = 14;
const BITMAP_INFO_HEADER_SIZE: u32 = 40;

#[derive(Parser)]
struct Cli {
    #[arg(short, long)]
    input: PathBuf,
}

fn main() {
    let cli = Cli::parse();

    let mut input = BufReader::new(File::open(&cli.input).expect("Failed to open input file"));

    let bigfile_header = BigFile::read_le(&mut input).expect("Failed to read as BigFile");

    input.seek(SeekFrom::Start(0)).unwrap();

    println!("File Contents:");

    for (i, file_descriptor) in bigfile_header.file_descriptors.iter().enumerate() {
        println!(
            " - Size: {} at Addr: 0x{:x}",
            file_descriptor.size, file_descriptor.start_at
        );

        // Read DIB file
        input
            .seek(SeekFrom::Start(file_descriptor.start_at as u64))
            .unwrap();

        let mut dib_content = vec![0; file_descriptor.size as usize];

        input
            .read_exact(&mut dib_content)
            .expect("Failed to strip file");

        // Decrypt DIB file
        dib_content
            .iter_mut()
            .for_each(|v| *v ^= bigfile_header.encrypt_key);

        // Get colorpalette size
        let colorpalette_size = BitmapInfoHeader::read_le(&mut Cursor::new(&dib_content))
            .expect("Failed to read DIB header")
            .colorpalette
            .len();

        let output_filename = &cli.input.with_file_name(format!(
            "{}.{i}.bmp",
            cli.input
                .file_name()
                .unwrap()
                .to_os_string()
                .to_string_lossy()
        ));

        let mut bmp = OpenOptions::new()
            .write(true)
            .create(true)
            .truncate(true)
            .open(output_filename)
            .expect("Failed to create output file");

        // Create BMP header
        let size = BITMAP_FILE_HEADER_SIZE + file_descriptor.size;

        let off_bits =
            BITMAP_FILE_HEADER_SIZE + BITMAP_INFO_HEADER_SIZE + colorpalette_size as u32 * 4;

        BitmapFileHeader { size, off_bits }
            .write_le(&mut bmp)
            .expect("Failed to write BMP header");

        bmp.write_all(&dib_content)
            .expect("Failed to write output file");
    }
}
