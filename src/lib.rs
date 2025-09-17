use binrw::binrw;

#[binrw]
#[derive(Debug)]
pub struct BigFileDescriptor {
    pub start_at: u32,
    pub size: u32,
}

#[binrw]
#[derive(Debug)]
pub struct BigFile {
    #[bw(try_calc(u32::try_from(file_descriptors.len())))]
    pub file_count: u32,

    pub encrypt_key: u8,

    #[br(count = file_count)]
    pub file_descriptors: Vec<BigFileDescriptor>,
}

#[binrw]
#[derive(Debug)]
pub struct ImageColor {
    pub b: u8,
    pub g: u8,
    pub r: u8,
    pub a: u8,
}

#[binrw]
#[derive(Debug)]
pub struct ImageFile {
    #[bw(calc(0x28))]
    _header_size: u32,

    pub width: u32,
    pub height: u32,

    #[bw(calc(1))]
    _constant_1: u16,

    pub bit_depth: u16,

    #[bw(calc(0))]
    _constant_0: u32,

    pub bitmap_image_size: u32,

    pub constant_2834_if_colorpalette_use_otherwise_0: u32,

    #[bw(calc(*constant_2834_if_colorpalette_use_otherwise_0))]
    _constant_2834_if_colorpalette_use_otherwise_0: u32,

    #[bw(try_calc(u32::try_from(colorpalette.len())))]
    pub colorpalette_size: u32,

    #[bw(try_calc(u32::try_from(colorpalette.len())))]
    _colorpalette_size: u32,

    #[br(count = colorpalette_size)]
    pub colorpalette: Vec<ImageColor>,
}

#[binrw]
#[derive(Debug)]
pub struct S10StrFileDescriptor {
    pub title_u16le: [u8; 520],
    pub start_at: u32,
    pub size: u32,
}

#[binrw]
#[derive(Debug)]
pub struct S10StrFile {
    #[bw(try_calc(u32::try_from(file_descriptors.len())))]
    pub file_count: u32,

    #[br(count = file_count)]
    pub file_descriptors: Vec<S10StrFileDescriptor>,
}
