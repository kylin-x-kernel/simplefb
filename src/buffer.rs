/// Circular buffer (FIFO) for caching log history
pub struct LogBuffer {
    buffer: &'static mut [u8],
    head: usize,  // Write position
    tail: usize,  // Read position
    len: usize,   // Current number of elements
}

impl LogBuffer {
    /// Creates a new log buffer using the provided storage
    ///
    /// # Safety
    ///
    /// The caller must ensure that the `storage` pointer is valid and points to a
    /// mutable array of size `N` that lives for the `'static` lifetime.
    pub fn new<const N: usize>(storage: *mut [u8; N]) -> Self {
        Self {
            buffer: unsafe { &mut *storage },
            head: 0,
            tail: 0,
            len: 0,
        }
    }

    /// Pushes a byte to the buffer, overwriting oldest data if full
    pub fn push(&mut self, byte: u8) {
        if self.buffer.is_empty() {
            return;
        }
        
        self.buffer[self.head] = byte;
        self.head = (self.head + 1) % self.buffer.len();
        
        if self.len == self.buffer.len() {
            // Buffer is full, move tail forward (overwrite oldest)
            self.tail = (self.tail + 1) % self.buffer.len();
        } else {
            self.len += 1;
        }
    }

    /// Pushes multiple bytes to the buffer
    pub fn push_bytes(&mut self, bytes: &[u8]) {
        for &b in bytes {
            self.push(b);
        }
    }

    /// Returns the number of bytes in the buffer
    pub fn len(&self) -> usize {
        self.len
    }

    /// Returns true if the buffer is empty
    pub fn is_empty(&self) -> bool {
        self.len == 0
    }

    /// Iterates over all bytes in the buffer (oldest to newest)
    pub fn iter(&self) -> LogBufferIter {
        LogBufferIter {
            buffer: self.buffer,
            pos: self.tail,
            remaining: self.len,
        }
    }
}

/// Iterator for LogBuffer
pub struct LogBufferIter<'a> {
    buffer: &'a [u8],
    pos: usize,
    remaining: usize,
}

impl<'a> Iterator for LogBufferIter<'a> {
    type Item = u8;

    fn next(&mut self) -> Option<Self::Item> {
        if self.remaining == 0 {
            return None;
        }
        let byte = self.buffer[self.pos];
        self.pos = (self.pos + 1) % self.buffer.len();
        self.remaining -= 1;
        Some(byte)
    }
}
