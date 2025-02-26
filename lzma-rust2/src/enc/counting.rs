use std::io::{Seek, SeekFrom};
use std::{cell::Cell, io::Write, rc::Rc};

pub struct CountingWriter<W> {
    inner: W,
    counting: Rc<Cell<usize>>,
    written_bytes: usize,
}

impl<W> CountingWriter<W> {
    pub fn new(inner: W) -> Self {
        Self {
            inner,
            counting: Rc::new(Cell::new(0)),
            written_bytes: 0,
        }
    }

    pub fn writed_bytes(&self) -> usize {
        self.written_bytes
    }

    pub fn counting(&self) -> Rc<Cell<usize>> {
        Rc::clone(&self.counting)
    }
}

impl<W: Write> Write for CountingWriter<W> {
    fn write(&mut self, buf: &[u8]) -> std::io::Result<usize> {
        let len = self.inner.write(buf)?;
        self.written_bytes += len;
        self.counting.set(self.written_bytes);
        Ok(len)
    }

    fn flush(&mut self) -> std::io::Result<()> {
        self.inner.flush()
    }
}

impl<W: Write + Seek> Seek for CountingWriter<W> {
    fn seek(&mut self, pos: SeekFrom) -> std::io::Result<u64> {
        self.inner.seek(pos)
    }
}
