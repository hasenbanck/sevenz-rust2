#[cfg(target_os = "macos")]
use std::os::macos::fs::FileTimesExt;
#[cfg(windows)]
use std::os::windows::fs::FileTimesExt;
use std::{
    fs::FileTimes,
    io::{Read, Seek},
    path::{Path, PathBuf},
};

use crate::{Error, Password, *};

/// Decompresses an archive file to a destination directory.
///
/// This is a convenience function for decompressing archive files directly from the filesystem.
///
/// # Arguments
/// * `src_path` - Path to the source archive file
/// * `dest` - Path to the destination directory where files will be extracted
#[cfg_attr(docsrs, doc(cfg(feature = "util")))]
pub fn decompress_file(src_path: impl AsRef<Path>, dest: impl AsRef<Path>) -> Result<(), Error> {
    let file = std::fs::File::open(src_path.as_ref())
        .map_err(|e| Error::file_open(e, src_path.as_ref().to_string_lossy().to_string()))?;
    decompress(file, dest)
}

/// Decompresses an archive file to a destination directory with a custom extraction function.
///
/// The extraction function is called for each entry in the archive, allowing custom handling
/// of individual files and directories during extraction.
///
/// # Arguments
/// * `src_path` - Path to the source archive file
/// * `dest` - Path to the destination directory where files will be extracted
/// * `extract_fn` - Custom function to handle each archive entry during extraction
#[cfg_attr(docsrs, doc(cfg(feature = "util")))]
pub fn decompress_file_with_extract_fn(
    src_path: impl AsRef<Path>,
    dest: impl AsRef<Path>,
    extract_fn: impl FnMut(&ArchiveEntry, &mut dyn Read, &PathBuf) -> Result<bool, Error>,
) -> Result<(), Error> {
    let file = std::fs::File::open(src_path.as_ref())
        .map_err(|e| Error::file_open(e, src_path.as_ref().to_string_lossy().to_string()))?;
    decompress_with_extract_fn(file, dest, extract_fn)
}

/// Decompresses an archive from a reader to a destination directory.
///
/// # Arguments
/// * `src_reader` - Reader containing the archive data
/// * `dest` - Path to the destination directory where files will be extracted
#[cfg_attr(docsrs, doc(cfg(feature = "util")))]
pub fn decompress<R: Read + Seek>(src_reader: R, dest: impl AsRef<Path>) -> Result<(), Error> {
    decompress_with_extract_fn(src_reader, dest, default_entry_extract_fn)
}

/// Decompresses an archive from a reader to a destination directory with a custom extraction function.
///
/// This provides the most flexibility, allowing both custom input sources and custom extraction logic.
///
/// # Arguments
/// * `src_reader` - Reader containing the archive data
/// * `dest` - Path to the destination directory where files will be extracted
/// * `extract_fn` - Custom function to handle each archive entry during extraction
#[cfg(not(target_arch = "wasm32"))]
#[cfg_attr(docsrs, doc(cfg(feature = "util")))]
pub fn decompress_with_extract_fn<R: Read + Seek>(
    src_reader: R,
    dest: impl AsRef<Path>,
    extract_fn: impl FnMut(&ArchiveEntry, &mut dyn Read, &PathBuf) -> Result<bool, Error>,
) -> Result<(), Error> {
    decompress_impl(src_reader, dest, Password::empty(), extract_fn)
}

/// Decompresses an encrypted archive file with the given password.
///
/// # Arguments
/// * `src_path` - Path to the encrypted source archive file
/// * `dest` - Path to the destination directory where files will be extracted
/// * `password` - Password to decrypt the archive
#[cfg(all(feature = "aes256", not(target_arch = "wasm32")))]
#[cfg_attr(docsrs, doc(cfg(all(feature = "aes256", feature = "util"))))]
pub fn decompress_file_with_password(
    src_path: impl AsRef<Path>,
    dest: impl AsRef<Path>,
    password: Password,
) -> Result<(), Error> {
    let file = std::fs::File::open(src_path.as_ref())
        .map_err(|e| Error::file_open(e, src_path.as_ref().to_string_lossy().to_string()))?;
    decompress_with_password(file, dest, password)
}

/// Decompresses an encrypted archive from a reader with the given password.
///
/// # Arguments
/// * `src_reader` - Reader containing the encrypted archive data
/// * `dest` - Path to the destination directory where files will be extracted
/// * `password` - Password to decrypt the archive
#[cfg(all(feature = "aes256", not(target_arch = "wasm32")))]
#[cfg_attr(docsrs, doc(cfg(all(feature = "aes256", feature = "util"))))]
pub fn decompress_with_password<R: Read + Seek>(
    src_reader: R,
    dest: impl AsRef<Path>,
    password: Password,
) -> Result<(), Error> {
    decompress_impl(src_reader, dest, password, default_entry_extract_fn)
}

/// Decompresses an encrypted archive from a reader with a custom extraction function and password.
///
/// This provides maximum flexibility for encrypted archives, allowing custom input sources,
/// custom extraction logic, and password decryption.
///
/// # Arguments
/// * `src_reader` - Reader containing the encrypted archive data
/// * `dest` - Path to the destination directory where files will be extracted
/// * `password` - Password to decrypt the archive
/// * `extract_fn` - Custom function to handle each archive entry during extraction
#[cfg(all(feature = "aes256", not(target_arch = "wasm32")))]
#[cfg_attr(docsrs, doc(cfg(all(feature = "aes256", feature = "util"))))]
pub fn decompress_with_extract_fn_and_password<R: Read + Seek>(
    src_reader: R,
    dest: impl AsRef<Path>,
    password: Password,
    extract_fn: impl FnMut(&ArchiveEntry, &mut dyn Read, &PathBuf) -> Result<bool, Error>,
) -> Result<(), Error> {
    decompress_impl(src_reader, dest, password, extract_fn)
}

#[cfg(not(target_arch = "wasm32"))]
fn decompress_impl<R: Read + Seek>(
    mut src_reader: R,
    dest: impl AsRef<Path>,
    password: Password,
    mut extract_fn: impl FnMut(&ArchiveEntry, &mut dyn Read, &PathBuf) -> Result<bool, Error>,
) -> Result<(), Error> {
    use std::io::SeekFrom;

    let pos = src_reader.stream_position().map_err(Error::io)?;
    src_reader.seek(SeekFrom::Start(pos)).map_err(Error::io)?;
    let mut seven = ArchiveReader::new(src_reader, password)?;
    let dest = PathBuf::from(dest.as_ref());
    if !dest.exists() {
        std::fs::create_dir_all(&dest).map_err(Error::io)?;
    }
    seven.for_each_entries(|entry, reader| {
        let dest_path = dest.join(entry.name());
        extract_fn(entry, reader, &dest_path)
    })?;

    Ok(())
}

/// Default extraction function that handles standard file and directory extraction.
///
/// # Arguments
/// * `entry` - Archive entry being processed
/// * `reader` - Reader for the entry's data
/// * `dest` - Destination path for the entry
#[cfg(not(target_arch = "wasm32"))]
#[cfg_attr(docsrs, doc(cfg(feature = "util")))]
pub fn default_entry_extract_fn(
    entry: &ArchiveEntry,
    reader: &mut dyn Read,
    dest: &PathBuf,
) -> Result<bool, Error> {
    use std::{fs::File, io::BufWriter};

    if entry.is_directory() {
        let dir = dest;
        if !dir.exists() {
            std::fs::create_dir_all(dir).map_err(Error::io)?;
        }
    } else {
        let path = dest;
        path.parent().and_then(|p| {
            if !p.exists() {
                std::fs::create_dir_all(p).ok()
            } else {
                None
            }
        });
        let file = File::create(path)
            .map_err(|e| Error::file_open(e, path.to_string_lossy().to_string()))?;
        if entry.size() > 0 {
            let mut writer = BufWriter::new(file);
            std::io::copy(reader, &mut writer).map_err(Error::io)?;

            let file = writer.get_mut();
            let file_times = FileTimes::new()
                .set_accessed(entry.access_date().into())
                .set_modified(entry.last_modified_date().into());

            #[cfg(any(windows, target_os = "macos"))]
            let file_times = file_times.set_created(entry.creation_date().into());

            let _ = file.set_times(file_times);
        }
    }

    Ok(true)
}
