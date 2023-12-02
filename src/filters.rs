use std::{
    fmt::Display,
    ops::{BitOr, BitOrAssign},
};

#[repr(u8)]
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, clap::ValueEnum)]
pub enum Filter {
    Image,
    Video,
    Music,
    All,
}
utils::impl_filter!(Filter: {
    Image[2] => [
        IMAGES_EXT:
            "avif", "jpg", "jpeg", "png", "gif", "webp",
            "tif", "tiff", "tga", "dds", "hdr", "exr",
            "pbm", "pam", "ppm", "pgm", "ff", "farbfeld",
            "qoi","heic",
    ],
    Video[4] => [
        VIDEOS_EXT:
            "mp4", "mkv", "webm", "mov", "m4p", "m4v",
            "mpg", "mpg", "mp2", "mpeg", "mpe", "mpv", "3gp"
    ],
    Music[8] => [
        MUSIC_EXT:
            "mp3", "flac"
    ]
});

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub struct Filters(u16);

#[allow(unused)]
impl Filters {
    #[inline]
    pub const fn contains(self, f: Filter) -> bool {
        (self.0 & f.as_u16()) != 0
    }

    #[inline]
    pub fn contains_ext(self, f: Filter, ext: &str) -> bool {
        if self.contains(f) {
            f.is_extension(ext)
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
        Self(self.0 | rhs.as_u16())
    }
}

impl BitOrAssign<Filter> for Filters {
    fn bitor_assign(&mut self, rhs: Filter) {
        self.0 |= rhs.as_u16()
    }
}

impl<I: IntoIterator<Item = Filter>> From<I> for Filters {
    fn from(value: I) -> Self {
        const MAX_FILTER: u16 = Filter::all();
        let mut s: u16 = 0;
        for val in value {
            s |= val.as_u16();
        }
        Self(s.min(MAX_FILTER))
    }
}
impl Default for Filters {
    fn default() -> Self {
        Self(Filter::Image.as_u16())
    }
}

pub static MAGIC_BYTES: &[&[u8]] = &[
    /*
     * MAGIC BYTES IMAGES
     */
    b"\x89PNG\r\n\x1a\n",      /* Png */
    &[0xff, 0xd8, 0xff],       /* Jpeg */
    b"GIF89a",                 /* Gif */
    b"GIF87a",                 /* Gif */
    b"RIFF",                   /* WebP  */
    b"MM\x00*",                /* Tiff */
    b"II*\x00",                /* Tiff */
    b"DDS ",                   /* Dds */
    b"#?RADIANCE",             /* Hdr */
    b"ftypheic",               /* Heic */
    b"P1",                     /* Pnm */
    b"P2",                     /* Pnm */
    b"P3",                     /* Pnm */
    b"P4",                     /* Pnm */
    b"P5",                     /* Pnm */
    b"P6",                     /* Pnm */
    b"P7",                     /* Pnm */
    b"farbfeld",               /* Farbfeld */
    b"\0\0\0 ftypavif",        /* Avif */
    b"\0\0\0\x1cftypavif",     /* Avif */
    &[0x76, 0x2f, 0x31, 0x01], /* OpenExr */
    b"qoif",                   /* Qoi */
    /*
     * MAGIC BYTES VIDEOS
     */
    b"ftypisom",               /* ISO Base Media file (MPEG-4) */
    b"ftypMSNV",               /* MPEG-4 */
    &[0x00, 0x00, 0x01, 0xBA], /* MPEG */
    &[0x00, 0x00, 0x01, 0xB3], /* MPEG */
    &[0x1A, 0x45, 0xDF, 0xA3], /* Matroska(MKV), including WebM */
    b"ftyp3g",                 /* 3gp */
    b"FLV",                    /* Flash Video file */
];

pub const MAGIC_BYTE_MAX_LEN: usize = 64;
pub fn contains_magic_bytes(bytes: impl AsRef<[u8]>) -> bool {
    MAGIC_BYTES.iter().any(|x| bytes.as_ref().starts_with(x))
}

mod utils {
    macro_rules! impl_filter {
        ($type:ty: {$(
            $n:ident[$num:literal] => [$name:ident : $($f:literal),* $(,)?]
        ),* $(,)?}) => {
            impl $type {
                $(const $name: &'static [&'static str] = &[$($f),*]);*;

                const fn all() -> u16 {
                    $(Self::$n.as_u16())|*
                }
                const fn as_u16(self) -> u16 {
                    match self {
                        $(Self::$n => $num),*,
                        Self::All => Self::all(),
                    }
                }

                #[allow(unused)]
                pub fn from_extension(ext: impl AsRef<str>) -> Option<Self> {
                    match ext.as_ref() {
                        $($($f)|* => Some(Self::$n)),*,
                        _ => None,
                    }
                }
                #[allow(unused)]
                pub fn is_extension(self, ext: &str) -> bool {
                    match self {
                        Self::All => {
                            $(Self::$name.contains(&ext))||*
                        }
                        $(Self::$n => Self::$name.contains(&ext)),*
                    }
                }
            }

            impl Display for $type {
                fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                    match self {
                        $(Self::$n => f.write_str(stringify!($n))),*,
                        Self::All => f.write_str("All"),
                    }
                }
            }

        };
    }
    pub(super) use impl_filter;
}
