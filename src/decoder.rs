use std::io::{Read, Result as IoResult};

use ll;

struct DecoderContext {
    c: ll::ZBUFFDecompressionContext,
}

impl DecoderContext {
    fn new() -> Self {
        DecoderContext { c: unsafe { ll::ZBUFF_createDCtx() } }
    }
}

impl Drop for DecoderContext {
    fn drop(&mut self) {
        let code = unsafe { ll::ZBUFF_freeDCtx(self.c) };
        ll::parse_code(code).unwrap();
    }
}


/// A decoder that decompress input data from another `Read`.
pub struct Decoder<R: Read> {
    // input reader (compressed data)
    reader: R,
    // input buffer
    buffer: Vec<u8>,
    // we already read everything in the buffer up to that point
    offset: usize,
    // decompression context
    context: DecoderContext,
}

impl<R: Read> Decoder<R> {
    pub fn new(reader: R) -> IoResult<Self> {

        let context = DecoderContext::new();

        try!(ll::parse_code(unsafe { ll::ZBUFF_decompressInit(context.c) }));

        let buffer_size = unsafe { ll::ZBUFF_recommendedDInSize() };

        Ok(Decoder {
            reader: reader,
            buffer: Vec::with_capacity(buffer_size),
            offset: 0,
            context: context,
        })
    }
}

impl<R: Read> Read for Decoder<R> {
    fn read(&mut self, buf: &mut [u8]) -> IoResult<usize> {

        let mut written = 0;
        while written != buf.len() {

            if self.offset == self.buffer.len() {
                // We need moar data!
                // Make a nice clean buffer
                let buffer_size = self.buffer.capacity();
                unsafe {
                    self.buffer.set_len(buffer_size);
                }

                // And FILL IT!
                self.offset = 0;
                let read = try!(self.reader.read(&mut self.buffer));
                unsafe {
                    self.buffer.set_len(read);
                }
                // If we can't read anything, no need to try and decompress it.
                // Just break the loop.
                if read == 0 {
                    break;
                }

            }

            let mut out_size = buf.len() - written;
            println!("Buffer: {:?}, off: {}", &self.buffer, self.offset);
            let mut in_size = self.buffer.len() - self.offset;

            unsafe {
                let code = ll::ZBUFF_decompressContinue(self.context.c,
                                                        buf[written..].as_mut_ptr(),
                                                        &mut out_size,
                                                        self.buffer[self.offset..].as_ptr(),
                                                        &mut in_size);
                let hint = try!(ll::parse_code(code));
            }

            written += out_size;
            self.offset += in_size;
        }
        Ok(written)
    }
}
