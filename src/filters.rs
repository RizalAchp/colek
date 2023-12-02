use std::{
    fmt::Display,
    ops::{BitOr, BitOrAssign},
};

#[rustfmt::skip]
pub static MAGIC_BYTES: [&[u8]; 21] = [
    b"\x89PNG\r\n\x1a\n",      // Png,
    &[0xff, 0xd8, 0xff],       // Jpeg,
    b"GIF89a",                 // Gif
    b"GIF87a",                 // Gif
    b"RIFF",                   // WebP 
    b"MM\x00*",                // Tiff
    b"II*\x00",                // Tiff
    b"DDS ",                   // Dds
    b"#?RADIANCE",             // Hdr
    b"P1",                     // Pnm
    b"P2",                     // Pnm
    b"P3",                     // Pnm
    b"P4",                     // Pnm
    b"P5",                     // Pnm
    b"P6",                     // Pnm
    b"P7",                     // Pnm
    b"farbfeld",               // Farbfeld
    b"\0\0\0 ftypavif",        // Avif
    b"\0\0\0\x1cftypavif",     // Avif
    &[0x76, 0x2f, 0x31, 0x01], // OpenExr // = &exr::meta::magic_number::BYTES
    b"qoif",                   // Qoi
];
pub const MAGIC_BYTE_MAX_LEN: usize = 64;
pub fn contains_magic_bytes(bytes: impl AsRef<[u8]>) -> bool {
    MAGIC_BYTES.iter().any(|x| x.starts_with(bytes.as_ref()))
}

pub static VIDEOS_EXTS: &[&str] = &[
    "mp4", "mkv", "webm", "mov", "m4p", "m4v", "mpg", "mpg", "mp2", "mpeg", "mpe", "mpv", "3gp",
];
pub static IMAGES_EXTS: &[&str] = &[
    "avif", "jpg", "jpeg", "png", "gif", "webp", "tif", "tiff", "tga", "dds", "hdr", "exr", "pbm",
    "pam", "ppm", "pgm", "ff", "farbfeld", "qoi",
];
pub static MUSIC_EXTS: &[&str] = &["mp3", "flac"];

#[repr(u8)]
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, clap::ValueEnum)]
pub enum Filter {
    Image = 2,
    Video = 4,
    Music = 8,
}
impl Filter {
    const fn as_u8(self) -> u8 {
        match self {
            Self::Image => 2,
            Self::Video => 4,
            Self::Music => 8,
        }
    }

    pub fn is_ext(self, ext: &str) -> bool {
        match self {
            Filter::Image => IMAGES_EXTS.contains(&ext),
            Filter::Video => VIDEOS_EXTS.contains(&ext),
            Filter::Music => MUSIC_EXTS.contains(&ext),
        }
    }
}

impl Display for Filter {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Filter::Image => f.write_str("image"),
            Filter::Video => f.write_str("video"),
            Filter::Music => f.write_str("music"),
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub struct Filters(u8);

impl Filters {
    #[inline]
    pub const fn contains(self, f: Filter) -> bool {
        (self.0 & f.as_u8()) != 0
    }
    #[inline]
    pub fn contains_ext(self, f: Filter, ext: &str) -> bool {
        if self.contains(f) {
            f.is_ext(ext)
        } else {
            false
        }
    }

    pub fn matches(self, ext: &str) -> bool {
        self.contains_ext(Filter::Image, ext)
            || self.contains_ext(Filter::Video, ext)
            || self.contains_ext(Filter::Music, ext)
    }
}

impl BitOr<Filter> for Filters {
    type Output = Self;
    fn bitor(self, rhs: Filter) -> Self::Output {
        Self(self.0 | rhs.as_u8())
    }
}

impl BitOrAssign<Filter> for Filters {
    fn bitor_assign(&mut self, rhs: Filter) {
        self.0 |= rhs.as_u8()
    }
}

impl<I: IntoIterator<Item = Filter>> From<I> for Filters {
    fn from(value: I) -> Self {
        let mut s = Self(0);
        for val in value {
            s |= val;
        }
        s
    }
}
