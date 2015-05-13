use std::cmp;
use std::io;
use std::slice;

trait IntoBufRead {
    type IntoBuf: io::BufRead + BufReadExt;

    fn into_buf_read(self) -> Self::IntoBuf;
}

trait BufReadExt {
    fn get_buf(&self) -> &[u8];
    fn read_u8(&mut self) -> io::Result<Option<u8>>;
}

struct SliceReader<'a> {
    buf: &'a [u8],
}

impl<'a> io::Read for SliceReader<'a> {
    #[inline]
    fn read(&mut self, buf: &mut [u8]) -> io::Result<usize> {
        let amt = cmp::min(buf.len(), self.buf.len());
        let (a, b) = self.buf.split_at(amt);
        slice::bytes::copy_memory(buf, a);
        *self.buf = b;
        Ok(amt)
    }
}

impl<'a> io::BufRead for SliceReader<'a> {
    fn fill_buf(&mut self) -> io::Result<&[u8]> { Ok(*self) }
    fn consume(&mut self, amt: usize) { *self.buf = &self.buf[amt..]; }
}

impl<'a> BufReadExt for SliceReader<'a> {
    fn get_buf(&self) -> &[u8] { self.buf }
    fn read_u8(&mut self) -> io::Result<Option<u8>> {
        let byte = self.buf.get(0);
        *self.buf = &self.buf[1..];
        byte
    }
}

struct BufReader<R> {
    inner: R,
    buf: io::Cursor<Vec<u8>>,
}

impl<R> BufReader<R> where R: io::Read {
    fn new(inner: R) -> Self {
        BufferedReader::with_capacity(io::DEFAULT_BUF_SIZE, inner)
    }

    fn new(cap: usize, inner: R) -> Self {
        BufferedReader {
            inner: inner,
            buf: io::Cursor::new(Vec::with_capacity(cap)),
        }
    }

    fn into_inner(self) -> R {
        self.inner
    }
}

impl<R> Read for BufReader<R> where R: io::Read {
    fn read(&mut self, buf: &mut [u8]) -> io::Result<usize> {
        // If we don't have any buffered data and we're doing a massive read
        // (larger than our internal buffer), bypass our internal buffer
        // entirely.
        if self.buf.get_ref().len() == self.buf.position() as usize &&
            buf.len() >= self.buf.get_ref().capacity() {
            return self.inner.read(buf);
        }
        try!(self.fill_buf());
        self.buf.read(buf)
    }
}

impl<R> BufReadExt for BufReader<R> {
    fn get_buf(&self) -> &[u8] {
        self.buf.get_ref()
    }

    fn read_u8(&mut self) -> io::Result<Option<u8>> {
        if self.buf.get_ref().len() == self.buf.position() as usize {

        }
        let byte = self.buf.get(0);
        *self.buf = &self.buf[1..];
        byte
    }
}
