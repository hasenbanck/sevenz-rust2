#[cfg(feature = "util")]
use std::path::PathBuf;
use std::{fs::File, io::Read, ops::Deref};

use crc32fast::Hasher;

pub(crate) struct SeqReader<R> {
    readers: Vec<R>,
    current: usize,
}

impl<R> Deref for SeqReader<R> {
    type Target = [R];

    fn deref(&self) -> &Self::Target {
        &self.readers
    }
}

impl<R> SeqReader<R> {
    pub(crate) fn new(readers: Vec<R>) -> Self {
        Self {
            readers,
            current: 0,
        }
    }

    pub(crate) fn reader_len(&self) -> usize {
        self.readers.len()
    }
}

impl<R: Read> Read for SeqReader<R> {
    fn read(&mut self, buf: &mut [u8]) -> std::io::Result<usize> {
        let mut i = 0;
        while self.current < self.readers.len() {
            let r = &mut self.readers[self.current];
            i = r.read(buf)?;
            if i == 0 {
                self.current += 1;
            } else {
                break;
            }
        }

        Ok(i)
    }
}

pub struct SourceReader<R> {
    reader: R,
    size: usize,
    crc: Hasher,
    crc_value: u32,
}

impl<R> From<R> for SourceReader<R> {
    fn from(value: R) -> Self {
        Self::new(value)
    }
}

impl<R: Read> Read for SourceReader<R> {
    fn read(&mut self, buf: &mut [u8]) -> std::io::Result<usize> {
        let n = self.reader.read(buf)?;
        if self.crc_value == 0 {
            if n > 0 {
                self.size += n;
                self.crc.update(&buf[..n]);
            } else {
                let crc = std::mem::replace(&mut self.crc, Hasher::new());
                self.crc_value = crc.finalize();
            }
        }
        Ok(n)
    }
}

impl<R> SourceReader<R> {
    pub fn new(reader: R) -> Self {
        Self {
            reader,
            size: 0,
            crc: Hasher::new(),
            crc_value: 0,
        }
    }

    pub fn read_count(&self) -> usize {
        self.size
    }

    pub fn crc_value(&self) -> u32 {
        self.crc_value
    }
}

#[cfg(feature = "util")]
pub(crate) struct LazyFileReader {
    path: PathBuf,
    reader: Option<File>,
    end: bool,
}

#[cfg(feature = "util")]
impl LazyFileReader {
    pub fn new(path: PathBuf) -> Self {
        Self {
            path,
            reader: None,
            end: false,
        }
    }
}

#[cfg(feature = "util")]
impl Read for LazyFileReader {
    fn read(&mut self, buf: &mut [u8]) -> std::io::Result<usize> {
        if self.end {
            return Ok(0);
        }
        if self.reader.is_none() {
            self.reader = Some(File::open(&self.path)?);
        }
        let n = self.reader.as_mut().unwrap().read(buf)?;
        if n == 0 {
            self.end = true;
            self.reader = None;
        }
        Ok(n)
    }
}
