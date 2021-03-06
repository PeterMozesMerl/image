//! This module provides a bit reader

use std::old_io;
use std::old_io::*;

/// Bit reader
pub trait BitReader: Reader {
    /// Returns the next `n` bits.
    fn read_bits(&mut self, n: u8) -> IoResult<u16>;
}

/// Bit writer
pub trait BitWriter: Writer {
    /// Writes the next `n` bits.
    fn write_bits(&mut self, v: u16, n: u8) -> IoResult<()>;
}

macro_rules! define_bit_readers {
    {$(
        $name:ident, #[$doc:meta];
    )*} => {

$( // START Structure definitions

#[$doc]
pub struct $name<R> where R: Reader {
    r: R,
    bits: u8,
    acc: u32,
}

impl<R: Reader> $name<R> {

    /// Creates a new bit reader
    pub fn new(reader: R) -> $name<R> {
        $name {
            r: reader,
            bits: 0,
            acc: 0,
        }
    }

    /// Returns true if the reader is aligned to a byte of the underlying byte stream.
    #[inline(always)]
    fn is_aligned(&self) -> bool {
        self.bits == 0
    }


}

impl<R: Reader> Reader for $name<R> {
    fn read(&mut self, buf: &mut [u8]) -> IoResult<usize> {
        if self.is_aligned() {
            self.r.read(buf)
        } else {
            let mut i = 0;
            for (j, byte) in buf.iter_mut().enumerate() {
                *byte = try!(self.read_bits(8)) as u8;
                i = j;
            }
            Ok(i)
        }
    }
}

)* // END Structure definitions

    }
}

define_bit_readers!{
    LsbReader, #[doc = "Reads bits from a byte stream, LSB first."];
    MsbReader, #[doc = "Reads bits from a byte stream, MSB first."];
}

impl<R> BitReader for LsbReader<R> where R: Reader {

    fn read_bits(&mut self, n: u8) -> IoResult<u16> {
        if n > 16 {
            return Err(old_io::IoError {
                kind: old_io::InvalidInput,
                desc: "Cannot read more than 16 bits",
                detail: None
            })
        }
        while self.bits < n {
            self.acc |= (try!(self.r.read_u8()) as u32) << self.bits;
            self.bits += 8;
        }
        let res = self.acc & ((1 << n) - 1);
        self.acc >>= n;
        self.bits -= n;
        Ok(res as u16)
    }

}

impl<R> BitReader for MsbReader<R> where R: Reader {

    fn read_bits(&mut self, n: u8) -> IoResult<u16> {
        if n > 16 {
            return Err(old_io::IoError {
                kind: old_io::InvalidInput,
                desc: "Cannot read more than 16 bits",
                detail: None
            })
        }
        while self.bits < n {
            self.acc |= (try!(self.r.read_u8()) as u32) << (24 - self.bits);
            self.bits += 8;
        }
        let res = self.acc >> (32 - n);
        self.acc <<= n;
        self.bits -= n;
        Ok(res as u16)
    }
}

macro_rules! define_bit_writers {
    {$(
        $name:ident, #[$doc:meta];
    )*} => {

$( // START Structure definitions

#[$doc]
#[allow(dead_code)]
pub struct $name<'a, W> where W: Writer + 'a {
    w: &'a mut W,
    bits: u8,
    acc: u32,
}

impl<'a, W> $name<'a, W> where W: Writer + 'a  {
    /// Creates a new bit reader
    #[allow(dead_code)]
    pub fn new(writer: &'a mut W) -> $name<'a, W> {
        $name {
            w: writer,
            bits: 0,
            acc: 0,
        }
    }
}

impl<'a, W> Writer for $name<'a, W> where W: Writer + 'a  {

    fn write_all(&mut self, buf: &[u8]) -> IoResult<()> {
        if self.acc == 0 {
            self.w.write(buf)
        } else {
            for &byte in buf.iter() {
                try!(self.write_bits(byte as u16, 8))
            }
            Ok(())
        }
    }

    fn flush(&mut self) -> IoResult<()> {
        let missing = 8 - self.bits;
        if missing > 0 {
            try!(self.write_bits(0, missing));
        }
        self.w.flush()
    }
}

)* // END Structure definitions

    }
}

define_bit_writers!{
    LsbWriter, #[doc = "Writes bits to a byte stream, LSB first."];
    MsbWriter, #[doc = "Writes bits to a byte stream, MSB first."];
}

impl<'a, W> BitWriter for LsbWriter<'a, W> where W: Writer + 'a  {

    fn write_bits(&mut self, v: u16, n: u8) -> IoResult<()> {
        self.acc |= (v as u32) << self.bits;
        self.bits += n;
        while self.bits >= 8 {
            try!(self.w.write_u8(self.acc as u8));
            self.acc >>= 8;
            self.bits -= 8

        }
        Ok(())
    }

}

impl<'a, W> BitWriter for MsbWriter<'a, W> where W: Writer + 'a  {

    fn write_bits(&mut self, v: u16, n: u8) -> IoResult<()> {
        self.acc |= (v as u32) << (32 - n - self.bits);
        self.bits += n;
        while self.bits >= 8 {
            try!(self.w.write_u8((self.acc >> 24) as u8));
            self.acc <<= 8;
            self.bits -= 8

        }
        Ok(())
    }

}

#[cfg(test)]
mod test {
    use super::{BitReader, BitWriter};

    #[test]
    fn reader_writer() {
        let data = [255, 20, 40, 120, 128];
        let mut expanded_data = Vec::new();
        let mut reader = super::LsbReader::new(&data[..]);
        while let Ok(b) = reader.read_bits(10) {
            expanded_data.push(b)
        }
        let mut compressed_data = Vec::new();
        {
            let mut writer = super::LsbWriter::new(&mut compressed_data);
            for &datum in expanded_data.iter() {
                let _  = writer.write_bits(datum, 10);
            }
        }
        assert_eq!(&data, &compressed_data)
    }
}
