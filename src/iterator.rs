use std::io;

pub struct LineColIterator<Iter: Iterator<Item=io::Result<u8>>> {
    rdr: Iter,
    line: usize,
    col: usize,
}

impl<Iter: Iterator<Item=io::Result<u8>>> LineColIterator<Iter> {
    pub fn new(iter: Iter) -> LineColIterator<Iter> {
        LineColIterator {
            line: 1,
            col: 0,
            rdr: iter,
        }
    }
    pub fn line(&self) -> usize { self.line }
    pub fn col(&self) -> usize { self.col }
}

impl<Iter: Iterator<Item=io::Result<u8>>> Iterator for LineColIterator<Iter> {
    type Item = io::Result<u8>;
    fn next(&mut self) -> Option<io::Result<u8>> {
        match self.rdr.next() {
            None => None,
            Some(Ok(b'\n')) => {
                self.line += 1;
                self.col = 0;
                Some(Ok(b'\n'))
            },
            Some(Ok(c)) => {
                self.col += 1;
                Some(Ok(c))
            },
            Some(Err(e)) => Some(Err(e)),
        }
    }
}
