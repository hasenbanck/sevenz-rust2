use std::io;

use lzma_sdk_sys::{
    SRes, SZ_ERROR_ARCHIVE, SZ_ERROR_CRC, SZ_ERROR_DATA, SZ_ERROR_FAIL, SZ_ERROR_INPUT_EOF,
    SZ_ERROR_MEM, SZ_ERROR_NO_ARCHIVE, SZ_ERROR_OUTPUT_EOF, SZ_ERROR_PARAM, SZ_ERROR_PROGRESS,
    SZ_ERROR_READ, SZ_ERROR_THREAD, SZ_ERROR_UNSUPPORTED, SZ_ERROR_WRITE, SZ_OK,
};

pub(crate) fn check_res(res: SRes) -> io::Result<()> {
    if res == SZ_OK as SRes {
        return Ok(());
    }
    Err(res_to_io_error(res))
}

pub(crate) fn res_to_io_error(res: SRes) -> io::Error {
    let res_u32 = res as u32;
    let (kind, msg) = if res_u32 == SZ_ERROR_DATA {
        (io::ErrorKind::InvalidData, "LZMA: data error")
    } else if res_u32 == SZ_ERROR_MEM {
        (io::ErrorKind::OutOfMemory, "LZMA: out of memory")
    } else if res_u32 == SZ_ERROR_CRC {
        (io::ErrorKind::InvalidData, "LZMA: CRC error")
    } else if res_u32 == SZ_ERROR_UNSUPPORTED {
        (io::ErrorKind::Unsupported, "LZMA: unsupported")
    } else if res_u32 == SZ_ERROR_PARAM {
        (io::ErrorKind::InvalidInput, "LZMA: invalid parameter")
    } else if res_u32 == SZ_ERROR_INPUT_EOF {
        (io::ErrorKind::UnexpectedEof, "LZMA: unexpected end of input")
    } else if res_u32 == SZ_ERROR_OUTPUT_EOF {
        (
            io::ErrorKind::WriteZero,
            "LZMA: output buffer not large enough",
        )
    } else if res_u32 == SZ_ERROR_READ {
        (io::ErrorKind::Other, "LZMA: read error")
    } else if res_u32 == SZ_ERROR_WRITE {
        (io::ErrorKind::Other, "LZMA: write error")
    } else if res_u32 == SZ_ERROR_PROGRESS {
        (io::ErrorKind::Other, "LZMA: progress error")
    } else if res_u32 == SZ_ERROR_FAIL {
        (io::ErrorKind::Other, "LZMA: operation failed")
    } else if res_u32 == SZ_ERROR_THREAD {
        (io::ErrorKind::Other, "LZMA: thread error")
    } else if res_u32 == SZ_ERROR_ARCHIVE {
        (io::ErrorKind::InvalidData, "LZMA: archive error")
    } else if res_u32 == SZ_ERROR_NO_ARCHIVE {
        (io::ErrorKind::InvalidData, "LZMA: not an archive")
    } else {
        (io::ErrorKind::Other, "LZMA: unknown error")
    };
    io::Error::new(kind, msg)
}
