use std::ops::{BitAnd, BitOr};

#[macro_export]
macro_rules! files_filter {

    ([$($ext:tt)*]) => { {
            let Some(Some(ext)) = dir.path().extension().map(|ext| ext.to_str().map(|s| s.to_ascii_lowercase())) else {
                return false;
            };
            match ext.as_ref() {
                $($ext)* => return true,
                _ => return false,
            }
    }};
    ($name:ident, $([$($ext:tt)*])?) => {
        paste::paste!{
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
        }
    };
}

files_filter!(
    is_images,
    ["avif" | "jpg" | "jpeg" | "png" | "gif" | "webp" | "heic"]
);

files_filter!(is_music, ["mp3" | "flac"]);

files_filter!(
    is_videos,
    ["mp4"
        | "mkv"
        | "webm"
        | "mov"
        | "m4p"
        | "m4v"
        | "mpg"
        | "mpg"
        | "mp2"
        | "mpeg"
        | "mpe"
        | "mpv"
        | "3gp"]
);

files_filter!(
    is_images_and_videos,
    ["avif"
        | "jpg"
        | "jpeg"
        | "png"
        | "gif"
        | "webp"
        | "heic"
        | "mp4"
        | "mkv"
        | "webm"
        | "mov"
        | "m4p"
        | "m4v"
        | "mpg"
        | "mpg"
        | "mp2"
        | "mpeg"
        | "mpe"
        | "mpv"
        | "3gp"]
);

#[repr(u32)]
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, clap::ValueEnum)]
pub enum Filter {
    Image = 2,
    Video = 4,
    Music = 8,
}
impl BitOr for Filter {
    type Output = u32;
    fn bitor(self, rhs: Self) -> Self::Output {
        self as u32 | rhs as u32
    }
}

impl BitAnd for Filter {
    type Output = u32;
    fn bitand(self, rhs: Self) -> Self::Output {
        self as u32 & rhs as u32
    }
}
