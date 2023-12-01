use std::{
    fmt::Display,
    ops::{BitOr, BitOrAssign},
};

#[repr(u8)]
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, clap::ValueEnum)]
pub enum Filter {
    Image = 2,
    Video = 4,
    Music = 8,
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
    pub const fn contains(self, f: Filter) -> bool {
        self.0 & (f as u8) != 8
    }
}

impl BitOr<Filter> for Filters {
    type Output = Self;
    fn bitor(self, rhs: Filter) -> Self::Output {
        Self(self.0 | rhs as u8)
    }
}

impl BitOrAssign<Filter> for Filters {
    fn bitor_assign(&mut self, rhs: Filter) {
        self.0 |= rhs as u8
    }
}

impl<I: IntoIterator<Item = Filter>> From<I> for Filters {
    fn from(value: I) -> Self {
        let mut v = 0;
        for val in value {
            v |= val as u8;
        }
        Self(v)
    }
}

#[macro_export]
macro_rules! files_filter {
    ($(($name:ident, $([$($ext:tt)*])?)),* $(,)?) => {
        paste::paste!{$(
            #[inline]
            #[allow(unused)]
            pub fn [<$name _impl>](ext: &str) -> bool {
                $(
                    match ext {
                        $($ext)* => return true,
                        _ => return false,
                    }
                )?
                true
            }
            #[inline]
            #[allow(unused)]
            pub fn $name(dir: &walkdir::DirEntry) -> bool {
                let Some(Some(ext)) = dir.path().extension().map(|ext| ext.to_str().map(|s| s.to_ascii_lowercase())) else {
                    return false;
                };
                [<$name _impl>](ext.as_ref())
            }
        )*}
    };
}
#[rustfmt::skip]
files_filter!( 
    (is_videos, ["mp4"  | "mkv" | "webm" | "mov" | "m4p" | "m4v" | "mpg" | "mpg" | "mp2" | "mpeg" | "mpe" | "mpv" | "3gp"]),
    (is_images, ["avif" | "jpg" | "jpeg" | "png" | "gif" | "webp" | "heic"]),
    (is_music,  ["mp3"  | "flac"]),
);
