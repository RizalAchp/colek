use walkdir::DirEntry;

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
        #[inline]
        #[allow(unused)]
        pub fn $name(dir: &walkdir::DirEntry) -> bool {
            let Some(Some(ext)) = dir.path().extension().map(|ext| ext.to_str().map(|s| s.to_ascii_lowercase())) else {
                return false;
            };
            $(
            match ext.as_ref() {
                $($ext)* => return true,
                _ => return false,
            }
            )?
            true
        }
    };
}

pub type FilterFn = dyn Fn(&DirEntry) -> bool + Send + Sync;

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
