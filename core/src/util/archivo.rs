use crate::util::seguridad::md5_file;
use std::fs::{self, File};
use std::io::{self, Read, Write};
use std::path::Path;
use walkdir::WalkDir;
use zip::{write::FileOptions, CompressionMethod, ZipWriter};

#[derive(Debug)]
pub struct ZipMetadata {
    pub size: u64,
    pub md5: String,
    pub comment: Option<String>,
}

/// Comprime un archivo o directorio en ZIP y retorna metadatos (md5, size).
pub fn compress_to_zip(source: &Path, dest: &Path) -> io::Result<ZipMetadata> {
    let file = File::create(dest)?;
    let mut zip = ZipWriter::new(file);

    let options = FileOptions::<()>::default()
        .compression_method(CompressionMethod::Deflated)
        .unix_permissions(0o755);

    let walker = WalkDir::new(source);

    // Convertir source path a absoluto o canonicalizado si es necesario
    // pero walkdir maneja relativos al cwd.

    for entry in walker.into_iter().filter_map(|e| e.ok()) {
        let path = entry.path();

        // Calcular nombre dentro del ZIP (relativo a source)
        let name = if source.is_file() {
            path.file_name().unwrap().to_string_lossy().into_owned()
        } else {
            path.strip_prefix(source.parent().unwrap_or(source))
                .unwrap_or(path)
                .to_string_lossy()
                .into_owned()
        };

        if path.is_file() {
            #[allow(deprecated)]
            zip.start_file(name, options)?;
            let mut f = File::open(path)?;
            let mut buffer = Vec::new();
            f.read_to_end(&mut buffer)?;
            zip.write_all(&buffer)?;
        } else if !name.is_empty() {
            #[allow(deprecated)]
            zip.add_directory(name, options)?;
        }
    }

    zip.finish()?;

    // Obtener stats
    let meta = fs::metadata(dest)?;
    let md5 = md5_file(dest)?;

    Ok(ZipMetadata {
        size: meta.len(),
        md5,
        comment: None,
    })
}

/// Establece el comentario global del archivo ZIP (usado para metadatos como Author).
/// NOTA: Reescribe el archivo temporalmente.
pub fn set_zip_comment(path: &Path, comment: &str) -> io::Result<()> {
    let temp_path = path.with_extension("tmp.zip");

    {
        let file_src = File::open(path)?;
        let mut archive = zip::ZipArchive::new(file_src)?;

        let file_dest = File::create(&temp_path)?;
        let mut zip_writer = ZipWriter::new(file_dest);

        zip_writer.set_comment(comment);

        for i in 0..archive.len() {
            let mut file = archive.by_index(i)?;
            let options = FileOptions::<()>::default()
                .compression_method(file.compression())
                .unix_permissions(file.unix_mode().unwrap_or(0o755));

            if file.is_dir() {
                #[allow(deprecated)]
                zip_writer.add_directory(file.name(), options)?;
            } else {
                #[allow(deprecated)]
                zip_writer.start_file(file.name(), options)?;
                std::io::copy(&mut file, &mut zip_writer)?;
            }
        }

        zip_writer.finish()?;
    }

    fs::rename(temp_path, path)?;
    Ok(())
}

/// Genera un archivo sidecar (.meta.json) para TXT/CSV con metadatos.
pub fn write_sidecar_metadata(path: &Path, author: &str) -> io::Result<()> {
    let mut meta_path = path.to_path_buf();
    if let Some(ext) = path.extension() {
        let mut new_ext = ext.to_os_string();
        new_ext.push(".meta.json");
        meta_path.set_extension(new_ext);
    } else {
        meta_path.set_extension("meta.json");
    }

    let json = format!(
        r#"{{ "author": "{}", "file": "{}" }}"#,
        author,
        path.file_name().unwrap_or_default().to_string_lossy()
    );
    fs::write(meta_path, json)
}
