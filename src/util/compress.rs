//! 7z Compressor helper functions

use std::{
    fs::{DirEntry, File},
    io::{ErrorKind, Seek, Write},
    path::{Path, PathBuf},
};

use crate::{ArchiveEntry, ArchiveWriter, Error, writer::LazyFileReader};
#[cfg(feature = "aes256")]
use crate::{EncoderMethod, Password, encoder_options::AesEncoderOptions};

/// Compresses a source file or directory to a destination writer.
///
/// Symbolic links inside a source directory are followed. A link that creates a cycle is
/// an error and a link without a target is skipped.
///
/// # Arguments
/// * `src` - Path to the source file or directory to compress
/// * `dest` - Writer that implements `Write + Seek` to write the compressed archive to
pub fn compress<W: Write + Seek>(src: impl AsRef<Path>, dest: W) -> Result<W, Error> {
    let mut archive_writer = ArchiveWriter::new(dest)?;
    let parent = if src.as_ref().is_dir() {
        src.as_ref()
    } else {
        src.as_ref().parent().unwrap_or(src.as_ref())
    };
    compress_path(src.as_ref(), parent, &mut archive_writer)?;
    Ok(archive_writer.finish()?)
}

/// Compresses a source file or directory to a destination writer with password encryption.
///
/// Symbolic links inside a source directory are followed. A link that creates a cycle is
/// an error and a link without a target is skipped.
///
/// # Arguments
/// * `src` - Path to the source file or directory to compress
/// * `dest` - Writer that implements `Write + Seek` to write the compressed archive to
/// * `password` - Password to encrypt the archive with
#[cfg(feature = "aes256")]
pub fn compress_encrypted<W: Write + Seek>(
    src: impl AsRef<Path>,
    dest: W,
    password: Password,
) -> Result<W, Error> {
    let mut archive_writer = ArchiveWriter::new(dest)?;
    if !password.is_empty() {
        archive_writer.set_content_methods(vec![
            AesEncoderOptions::new(password).into(),
            EncoderMethod::LZMA2.into(),
        ]);
    }
    let parent = if src.as_ref().is_dir() {
        src.as_ref()
    } else {
        src.as_ref().parent().unwrap_or(src.as_ref())
    };
    compress_path(src.as_ref(), parent, &mut archive_writer)?;
    Ok(archive_writer.finish()?)
}

/// Compresses a source file or directory to a destination file path.
///
/// This is a convenience function that handles file creation automatically.
///
/// Symbolic links inside a source directory are followed. A link that creates a cycle is
/// an error and a link without a target is skipped.
///
/// # Arguments
/// * `src` - Path to the source file or directory to compress
/// * `dest` - Path where the compressed archive will be created
pub fn compress_to_path(src: impl AsRef<Path>, dest: impl AsRef<Path>) -> Result<(), Error> {
    if let Some(path) = dest.as_ref().parent()
        && !path.exists()
    {
        std::fs::create_dir_all(path)
            .map_err(|e| Error::io_msg(e, format!("Create dir failed:{:?}", dest.as_ref())))?;
    }
    compress(
        src,
        File::create(dest.as_ref())
            .map_err(|e| Error::file_open(e, dest.as_ref().to_string_lossy().to_string()))?,
    )?;
    Ok(())
}

/// Compresses a source file or directory to a destination file path with password encryption.
///
/// This is a convenience function that handles file creation automatically.
///
/// Symbolic links inside a source directory are followed. A link that creates a cycle is
/// an error and a link without a target is skipped.
///
/// # Arguments
/// * `src` - Path to the source file or directory to compress
/// * `dest` - Path where the encrypted compressed archive will be created
/// * `password` - Password to encrypt the archive with
#[cfg(feature = "aes256")]
pub fn compress_to_path_encrypted(
    src: impl AsRef<Path>,
    dest: impl AsRef<Path>,
    password: Password,
) -> Result<(), Error> {
    if let Some(path) = dest.as_ref().parent()
        && !path.exists()
    {
        std::fs::create_dir_all(path)
            .map_err(|e| Error::io_msg(e, format!("Create dir failed:{:?}", dest.as_ref())))?;
    }
    compress_encrypted(
        src,
        File::create(dest.as_ref())
            .map_err(|e| Error::file_open(e, dest.as_ref().to_string_lossy().to_string()))?,
        password,
    )?;
    Ok(())
}

fn compress_path<W: Write + Seek>(
    src: &Path,
    root: &Path,
    archive_writer: &mut ArchiveWriter<W>,
) -> Result<(), Error> {
    if src.is_dir() {
        let canonical_src = std::fs::canonicalize(src)
            .map_err(|e| Error::file_open(e, src.to_string_lossy().to_string()))?;
        compress_dir(src, &canonical_src, root, &mut Vec::new(), archive_writer)
    } else {
        compress_file(src, root, archive_writer)
    }
}

/// Adds the content of the directory `dir` with the canonical path `canonical_dir` to the
/// archive. `ancestors` holds the canonical paths of the directories above `dir`.
fn compress_dir<W: Write + Seek>(
    dir: &Path,
    canonical_dir: &Path,
    root: &Path,
    ancestors: &mut Vec<PathBuf>,
    archive_writer: &mut ArchiveWriter<W>,
) -> Result<(), Error> {
    for entry in dir
        .read_dir()
        .map_err(|e| Error::io_msg(e, "error read dir"))?
    {
        let entry = entry?;
        let path = entry.path();
        match resolve_entry(&entry, canonical_dir, ancestors)? {
            EntryKind::Directory(canonical_path) => {
                let entry = ArchiveEntry::from_path(&path, entry_name(&path, root)?);
                archive_writer.push_archive_entry::<&[u8]>(entry, None)?;
                ancestors.push(canonical_dir.to_path_buf());
                compress_dir(&path, &canonical_path, root, ancestors, archive_writer)?;
                ancestors.pop();
            }
            EntryKind::File => compress_file(&path, root, archive_writer)?,
            EntryKind::Skip => {}
        }
    }
    Ok(())
}

fn compress_file<W: Write + Seek>(
    path: &Path,
    root: &Path,
    archive_writer: &mut ArchiveWriter<W>,
) -> Result<(), Error> {
    let entry = ArchiveEntry::from_path(path, entry_name(path, root)?);
    let file =
        File::open(path).map_err(|e| Error::file_open(e, path.to_string_lossy().to_string()))?;
    archive_writer.push_archive_entry(entry, Some(file))?;
    Ok(())
}

fn entry_name(path: &Path, root: &Path) -> Result<String, Error> {
    Ok(path
        .strip_prefix(root)
        .map_err(|e| Error::other(e.to_string()))?
        .to_string_lossy()
        .to_string())
}

/// The kind of a directory entry after its symbolic links are resolved.
enum EntryKind {
    /// A directory with its canonical path.
    Directory(PathBuf),
    /// A regular file.
    File,
    /// A link without a target, a FIFO, a socket or a device file.
    Skip,
}

/// Resolves an entry of the directory with the canonical path `canonical_dir`.
///
/// A symbolic link is followed. A link to a directory that contains
/// `canonical_dir` or one of its `ancestors` creates a cycle and is an error.
fn resolve_entry(
    entry: &DirEntry,
    canonical_dir: &Path,
    ancestors: &[PathBuf],
) -> std::io::Result<EntryKind> {
    let file_type = entry.file_type()?;
    if !file_type.is_symlink() {
        return Ok(if file_type.is_dir() {
            EntryKind::Directory(canonical_dir.join(entry.file_name()))
        } else if file_type.is_file() {
            EntryKind::File
        } else {
            EntryKind::Skip
        });
    }

    let path = entry.path();
    let target = match std::fs::canonicalize(&path) {
        Ok(target) => target,
        Err(e) if e.kind() == ErrorKind::NotFound => return Ok(EntryKind::Skip),
        Err(e) => {
            return Err(std::io::Error::new(
                e.kind(),
                format!("can't resolve symbolic link {path:?}: {e}"),
            ));
        }
    };
    let metadata = std::fs::metadata(&target)?;
    if metadata.is_dir() {
        let cycle = canonical_dir.starts_with(&target)
            || ancestors
                .iter()
                .any(|ancestor| ancestor.starts_with(&target));
        if cycle {
            return Err(std::io::Error::new(
                ErrorKind::InvalidInput,
                format!("symbolic link {path:?} creates a cycle"),
            ));
        }
        Ok(EntryKind::Directory(target))
    } else if metadata.is_file() {
        Ok(EntryKind::File)
    } else {
        Ok(EntryKind::Skip)
    }
}

impl<W: Write + Seek> ArchiveWriter<W> {
    /// Adds a source path to the compression builder with a filter function using solid compression.
    ///
    /// The filter function allows selective inclusion of files based on their paths.
    /// Files are compressed using solid compression for better compression ratios.
    ///
    /// Symbolic links inside a source directory are followed. A link that creates a cycle is
    /// an error and a link without a target is skipped.
    ///
    /// # Arguments
    /// * `path` - Path to add to the compression
    /// * `filter` - Function that returns `true` for paths that should be included
    pub fn push_source_path(
        &mut self,
        path: impl AsRef<Path>,
        filter: impl Fn(&Path) -> bool,
    ) -> Result<&mut Self, Error> {
        encode_path(true, &path, self, filter)?;
        Ok(self)
    }

    /// Adds a source path to the compression builder with a filter function using non-solid compression.
    ///
    /// Non-solid compression allows individual file extraction without decompressing the entire archive,
    /// but typically results in larger archive sizes compared to solid compression.
    ///
    /// Symbolic links inside a source directory are followed. A link that creates a cycle is
    /// an error and a link without a target is skipped.
    ///
    /// # Arguments
    /// * `path` - Path to add to the compression
    /// * `filter` - Function that returns `true` for paths that should be included
    pub fn push_source_path_non_solid(
        &mut self,
        path: impl AsRef<Path>,
        filter: impl Fn(&Path) -> bool,
    ) -> Result<&mut Self, Error> {
        encode_path(false, &path, self, filter)?;
        Ok(self)
    }
}

fn collect_file_paths(
    src: impl AsRef<Path>,
    paths: &mut Vec<PathBuf>,
    filter: &dyn Fn(&Path) -> bool,
) -> std::io::Result<()> {
    let path = src.as_ref();
    if !filter(path) {
        return Ok(());
    }
    if path.is_dir() {
        let canonical_path = std::fs::canonicalize(path)?;
        collect_dir_file_paths(path, &canonical_path, &mut Vec::new(), paths, filter)
    } else {
        paths.push(path.to_path_buf());
        Ok(())
    }
}

/// Collects the files inside the directory `dir` with the canonical path `canonical_dir`.
/// `ancestors` holds the canonical paths of the directories above `dir`.
fn collect_dir_file_paths(
    dir: &Path,
    canonical_dir: &Path,
    ancestors: &mut Vec<PathBuf>,
    paths: &mut Vec<PathBuf>,
    filter: &dyn Fn(&Path) -> bool,
) -> std::io::Result<()> {
    for entry in dir.read_dir()? {
        let entry = entry?;
        let path = entry.path();
        if !filter(&path) {
            continue;
        }
        match resolve_entry(&entry, canonical_dir, ancestors)? {
            EntryKind::Directory(canonical_path) => {
                ancestors.push(canonical_dir.to_path_buf());
                collect_dir_file_paths(&path, &canonical_path, ancestors, paths, filter)?;
                ancestors.pop();
            }
            EntryKind::File => paths.push(path),
            EntryKind::Skip => {}
        }
    }
    Ok(())
}

const MAX_BLOCK_SIZE: u64 = 4 * 1024 * 1024 * 1024; // 4 GiB

fn encode_path<W: Write + Seek>(
    solid: bool,
    src: impl AsRef<Path>,
    zip: &mut ArchiveWriter<W>,
    filter: impl Fn(&Path) -> bool,
) -> Result<(), Error> {
    let mut entries = Vec::new();
    let mut paths = Vec::new();
    collect_file_paths(&src, &mut paths, &filter).map_err(|e| {
        Error::io_msg(
            e,
            format!("Failed to collect entries from path:{:?}", src.as_ref()),
        )
    })?;

    if !solid {
        for ele in paths.into_iter() {
            let name = extract_file_name(&src, &ele)?;

            zip.push_archive_entry(
                ArchiveEntry::from_path(ele.as_path(), name),
                Some(File::open(ele.as_path())?),
            )?;
        }
        return Ok(());
    }
    let mut files = Vec::new();
    let mut file_size = 0;
    for ele in paths.into_iter() {
        let size = ele.metadata()?.len();
        let name = extract_file_name(&src, &ele)?;

        if size >= MAX_BLOCK_SIZE {
            zip.push_archive_entry(
                ArchiveEntry::from_path(ele.as_path(), name),
                Some(File::open(ele.as_path())?),
            )?;
            continue;
        }
        if file_size + size >= MAX_BLOCK_SIZE {
            zip.push_archive_entries(entries, files)?;
            entries = Vec::new();
            files = Vec::new();
            file_size = 0;
        }
        file_size += size;
        entries.push(ArchiveEntry::from_path(ele.as_path(), name));
        files.push(LazyFileReader::new(ele).into());
    }
    if !entries.is_empty() {
        zip.push_archive_entries(entries, files)?;
    }

    Ok(())
}

fn extract_file_name(src: &impl AsRef<Path>, ele: &PathBuf) -> Result<String, Error> {
    if ele == src.as_ref() {
        // Single file case: use just the filename.
        Ok(ele
            .file_name()
            .ok_or_else(|| {
                Error::io_msg(
                    std::io::Error::new(std::io::ErrorKind::InvalidInput, "Invalid filename"),
                    format!("Failed to get filename from {ele:?}"),
                )
            })?
            .to_string_lossy()
            .to_string())
    } else {
        // Directory case: remove path.
        Ok(ele.strip_prefix(src).unwrap().to_string_lossy().to_string())
    }
}
