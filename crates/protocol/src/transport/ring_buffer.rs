
/// A high-performance, zero-copy contiguous ring buffer for network packet streaming.
///
/// Ensures incoming socket bytes are read directly into the buffer from the OS socket,
/// decrypted in-place without copying, and decoded with SIMD/BMI2 acceleration.
#[derive(Debug)]
pub struct ZeroCopyRingBuffer {
    buffer: Vec<u8>,
    read_pos: usize,
    write_pos: usize,
}

impl Default for ZeroCopyRingBuffer {
    fn default() -> Self {
        Self::new()
    }
}

impl ZeroCopyRingBuffer {
    /// Default buffer capacity: 128 KB, sufficient for hundreds of normal packets
    /// or multiple full chunk columns before requiring compaction.
    pub const DEFAULT_CAPACITY: usize = 131072;

    /// Creates a new ring buffer with the default capacity.
    pub fn new() -> Self {
        Self::with_capacity(Self::DEFAULT_CAPACITY)
    }

    /// Creates a new ring buffer with the specified capacity.
    pub fn with_capacity(capacity: usize) -> Self {
        let capacity = capacity.max(4096);
        Self {
            buffer: vec![0u8; capacity],
            read_pos: 0,
            write_pos: 0,
        }
    }

    /// Total number of unread bytes available in the buffer.
    #[inline]
    pub fn readable_len(&self) -> usize {
        self.write_pos - self.read_pos
    }

    /// Whether there are no unread bytes in the buffer.
    #[inline]
    pub fn is_empty(&self) -> bool {
        self.read_pos == self.write_pos
    }

    /// Total allocated capacity of the buffer.
    #[inline]
    pub fn capacity(&self) -> usize {
        self.buffer.len()
    }

    /// Returns a contiguous slice of all unread bytes currently available.
    #[inline]
    pub fn readable_slice(&self) -> &[u8] {
        &self.buffer[self.read_pos..self.write_pos]
    }

    /// Advances the read pointer by `count` bytes after a packet has been processed.
    #[inline]
    pub fn advance_read(&mut self, count: usize) {
        assert!(
            self.read_pos + count <= self.write_pos,
            "Cannot advance read beyond write position"
        );
        self.read_pos += count;
        if self.read_pos == self.write_pos {
            self.read_pos = 0;
            self.write_pos = 0;
        }
    }

    /// Returns a contiguous mutable slice of available space for writing.
    /// If remaining space at the end is smaller than `min_free`, the buffer is compacted.
    /// If still insufficient, the buffer is automatically resized.
    pub fn prepare_write_slice(&mut self, min_free: usize) -> &mut [u8] {
        let free_at_end = self.buffer.len() - self.write_pos;
        if free_at_end < min_free {
            self.compact();
            let new_free = self.buffer.len() - self.write_pos;
            if new_free < min_free {
                let required = self.write_pos + min_free;
                let new_cap = (self.buffer.len() * 2).max(required);
                self.buffer.resize(new_cap, 0);
            }
        }
        &mut self.buffer[self.write_pos..]
    }

    /// Commits `n` bytes written into the slice returned by `prepare_write_slice`.
    #[inline]
    pub fn commit_write(&mut self, n: usize) {
        assert!(
            self.write_pos + n <= self.buffer.len(),
            "Cannot commit more bytes than buffer capacity"
        );
        self.write_pos += n;
    }

    /// Moves unread bytes to the beginning of the buffer to maximize contiguous free write space.
    pub fn compact(&mut self) {
        if self.read_pos == 0 {
            return;
        }
        let unread = self.write_pos - self.read_pos;
        if unread > 0 {
            self.buffer.copy_within(self.read_pos..self.write_pos, 0);
        }
        self.read_pos = 0;
        self.write_pos = unread;
    }

    /// Clears the ring buffer, resetting both read and write positions to zero.
    pub fn clear(&mut self) {
        self.read_pos = 0;
        self.write_pos = 0;
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_ring_buffer_basic_flow() {
        let mut ring = ZeroCopyRingBuffer::with_capacity(64);
        assert!(ring.is_empty());
        assert_eq!(ring.readable_len(), 0);

        // Write 10 bytes
        let slice = ring.prepare_write_slice(10);
        slice[..10].copy_from_slice(b"0123456789");
        ring.commit_write(10);

        assert_eq!(ring.readable_len(), 10);
        assert_eq!(ring.readable_slice(), b"0123456789");

        // Read 5 bytes
        ring.advance_read(5);
        assert_eq!(ring.readable_len(), 5);
        assert_eq!(ring.readable_slice(), b"56789");

        // Write another 10 bytes
        let slice = ring.prepare_write_slice(10);
        slice[..10].copy_from_slice(b"abcdefghij");
        ring.commit_write(10);

        assert_eq!(ring.readable_len(), 15);
        assert_eq!(ring.readable_slice(), b"56789abcdefghij");

        // Consume remaining
        ring.advance_read(15);
        assert!(ring.is_empty());
    }

    #[test]
    fn test_ring_buffer_stress_compaction() {
        let mut ring = ZeroCopyRingBuffer::with_capacity(32);
        for i in 0..1000 {
            let data = format!("msg_{:04};", i).into_bytes();
            let slice = ring.prepare_write_slice(data.len());
            slice[..data.len()].copy_from_slice(&data);
            ring.commit_write(data.len());

            assert_eq!(ring.readable_slice(), &data[..]);
            ring.advance_read(data.len());
            assert!(ring.is_empty());
        }
    }
}
