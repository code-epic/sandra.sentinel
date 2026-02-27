use md5::{Digest, Md5};
use std::fs;
use std::io::{self, Read};
use std::path::Path;

/// Calcula el hash MD5 de una cadena de texto.
pub fn md5_string(input: &str) -> String {
    let mut hasher = Md5::new();
    hasher.update(input.as_bytes());
    format!("{:x}", hasher.finalize())
}

/// Calcula el hash MD5 de un archivo local.
pub fn md5_file(path: &Path) -> io::Result<String> {
    let mut file = fs::File::open(path)?;
    let mut hasher = Md5::new();
    let mut buffer = [0; 1024];

    loop {
        let n = file.read(&mut buffer)?;
        if n == 0 {
            break;
        }
        hasher.update(&buffer[..n]);
    }

    Ok(format!("{:x}", hasher.finalize()))
}
