pub struct RollingWindow {
    pub block_sum: u32,
    pub all_blocks_sum: u32,
    pub window_size: u32,
}

impl RollingWindow {
    // Select a large prime number to be used as modulus
    // Use one that doesn't overflow u32 from https://primes.utm.edu/curios/index.php?start=5&stop=5
    const LARGE_PRIME_MOD: u32 = 21191;

    pub fn generate() -> Self {
        Self {
            block_sum: 0,
            all_blocks_sum: 0,
            window_size: 0,
        }
    }
}

impl Default for RollingWindow {
    fn default() -> Self {
        Self::generate()
    }
}

impl RollingWindow {
    //Get rolling hash after shift operation
    pub fn sha256_digest(&self) -> u32 {
        // If we used different modulo, we would have here r = r1 + (r2 * MODULO).
        // Because MODULO is 1 << 16 we can left shift bits also here.
        //(self.block_sum % IndexHash::MODULO + (self.r2 * IndexHash::MODULO)) % IndexHash::MODULO
        self.block_sum + (self.all_blocks_sum * RollingWindow::LARGE_PRIME_MOD)
    }

    // Append bytes slices to the current checksum state while doing mod of large prime number at every step
    pub fn add_bytes_at_end(&mut self, byte_buf: &[u8]) {
        let mut block_size: u32 = 0;
        let mut all_blocks_size: u32 = 0;
        let byte_lengh = byte_buf.len() as u32;

        byte_buf.iter().enumerate().for_each(|(index, byte)| {
            block_size += *byte as u32;
            all_blocks_size += (*byte as u32) * (byte_lengh - (index as u32));
        });

        self.block_sum = (self.block_sum.wrapping_add(block_size)) % RollingWindow::LARGE_PRIME_MOD;
        self.all_blocks_sum =
            (self.all_blocks_sum.wrapping_add(all_blocks_size)) % RollingWindow::LARGE_PRIME_MOD;
        self.window_size =
            (self.window_size.wrapping_add(byte_lengh)) % RollingWindow::LARGE_PRIME_MOD;
    }

    // Roll window : Remove one block of byte from the beginning and add one at the end
    pub fn roll_window(&mut self, prev: u8, next: Option<u8>) {
        self.block_sum = (self
            .block_sum
            .wrapping_sub(prev as u32)
            .wrapping_add(next.map_or(0, u32::from)))
            % RollingWindow::LARGE_PRIME_MOD;
        self.all_blocks_sum = (self
            .all_blocks_sum
            .wrapping_sub(self.window_size * (prev as u32))
            .wrapping_add(self.block_sum))
            % RollingWindow::LARGE_PRIME_MOD;
        if next.is_none() {
            self.window_size = self.window_size.wrapping_sub(1);
        }
    }
}

// Calculate hash of rolling window based on index of bytes
pub fn rolling_window_checksum(chunk: &[u8]) -> u32 {
    let mut checksum = RollingWindow::generate();
    checksum.add_bytes_at_end(chunk);
    checksum.sha256_digest()
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    pub fn test_window_generation() {
        let rolling_win = RollingWindow::generate();
        assert_eq!(rolling_win.window_size, 0);
        assert_eq!(rolling_win.block_sum, 0);
        assert_eq!(rolling_win.all_blocks_sum, 0);
        assert_eq!(rolling_win.sha256_digest(), 0);
    }

    #[test]
    pub fn test_rolling_window_checksum() {
        let mut rolling_win = RollingWindow::generate();
        rolling_win.add_bytes_at_end(vec!['a' as u8, 'b' as u8, 'c' as u8, 'd' as u8].as_slice());
        assert_eq!(rolling_win.window_size, 4);
        assert_eq!(20767574, rolling_win.sha256_digest());

        rolling_win.add_bytes_at_end(vec!['e' as u8, 'f' as u8, 'g' as u8, 'h' as u8].as_slice());
        assert_eq!(rolling_win.window_size, 8);
        assert_eq!(42382804, rolling_win.sha256_digest());

        rolling_win.roll_window(1, Some('i' as u8));
        assert_eq!(rolling_win.window_size, 8);
        assert_eq!(61454808, rolling_win.sha256_digest());

        rolling_win.roll_window(2, Some('j' as u8));
        rolling_win.roll_window(3, Some('k' as u8));
        rolling_win.roll_window(4, None);
        assert_eq!(rolling_win.window_size, 7);
        assert_eq!(128588100, rolling_win.sha256_digest());
    }

    #[test]
    pub fn test_rolling_window_shift() {
        let mut rolling_window = RollingWindow::generate();

        let mut rolling_window_bytes: Vec<u8> = Vec::with_capacity(80);
        for i in 0..rolling_window_bytes.capacity() {
            rolling_window_bytes.push(i as u8);
        }
        rolling_window.add_bytes_at_end(rolling_window_bytes.as_slice());
        assert_eq!(11785356, rolling_window.sha256_digest());
    }

    #[test]
    pub fn test_rolling_window_chunk_checksum() {
        let vec: Vec<u8> = vec![5; 20];
        assert_eq!(22250650, rolling_window_checksum(vec.as_slice()));
    }
}
