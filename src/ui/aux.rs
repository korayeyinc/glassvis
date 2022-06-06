//! Auxiliary module for Glassvis.

use std::env;
use std::fmt;
use std::path::{Path, PathBuf};
use std::string::String;

pub fn get_env(key: &String) -> String {
    let val = env::var(key).unwrap();
    return val;
}

pub fn set_env(key: &String, val: &String) {
    env::set_var(key, val);
}

/// Returns "data" directory path (as path buffer).
pub fn get_path() -> PathBuf {
    let path = PathBuf::from("data");
    return path;
}

/// Returns "data" directory path (as path buffer).
pub fn set_path(prefix: &str, dst: &str) -> PathBuf {
    let mut path = PathBuf::from(dst);
    let file_name = prefix.to_owned() + path.file_name().unwrap().to_str().unwrap();

    path = get_path();
    path.push("output");
    path.push(file_name);

    return path;
}

/// Converts given 'String' to 'Path'.
pub fn to_path(input: &String) -> &Path {
    let output = Path::new(input);

    return output;
}

/// Extracts the filename (as String) from a given path.
pub fn get_filename(file_path: &String) -> String {
    let file_name = to_path(&file_path).file_name().unwrap().to_str().unwrap();

    return file_name.to_string();
}

/// Checks if input file is an image.  
pub fn is_image_file(input: &str) -> bool {
    let extensions = [
        ".bmp", ".gif", ".jpg", ".jpeg", ".png", ".pnm", ".tga", ".tiff", ".webp",
    ];

    for ext in extensions.iter() {
        if input.contains(ext) {
            return true;
        }
    }

    return false;
}
