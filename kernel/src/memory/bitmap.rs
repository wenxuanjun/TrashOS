use bit_field::BitField;

pub struct Bitmap {
    inner: &'static mut [usize],
}

#[allow(dead_code)]
impl Bitmap {
    const BITS: usize = usize::BITS as usize;

    pub fn new(inner: &'static mut [usize]) -> Self {
        inner.fill(0);
        Self { inner }
    }

    #[inline]
    pub fn len(&self) -> usize {
        self.inner.len() * Self::BITS
    }

    #[inline]
    pub fn get(&self, index: usize) -> bool {
        let byte = self.inner[index / Self::BITS];
        byte.get_bit(index % Self::BITS)
    }

    #[inline]
    pub fn set(&mut self, index: usize, value: bool) {
        let byte = &mut self.inner[index / Self::BITS];
        byte.set_bit(index % Self::BITS, value);
    }
}

impl Bitmap {
    pub fn set_range(&mut self, start: usize, end: usize, value: bool) {
        if start >= end || start >= self.len() {
            return;
        }

        let start_byte = start.div_ceil(Self::BITS);
        let end_byte = end / Self::BITS;

        (start..(start_byte * Self::BITS).min(end)).for_each(|i| self.set(i, value));

        if start_byte > end_byte {
            return;
        }

        if start_byte <= end_byte {
            let fill_value = if value { usize::MAX } else { 0 };
            self.inner[start_byte..end_byte].fill(fill_value);
        }

        ((end_byte * Self::BITS).max(start)..end).for_each(|i| self.set(i, value));
    }

    #[rustfmt::skip]
    pub fn find_range(&mut self, length: usize, value: bool) -> Option<usize> {
        let mut count = 0;
        let mut start_index = 0;

        let byte_match = if value { usize::MAX } else { 0 };
        let byte_conflict = !byte_match;

        for (i, &byte) in self.inner.iter().enumerate() {
            if byte == byte_conflict {
                count = 0;
                continue;
            }

            if byte == byte_match && length < Self::BITS {
                return Some(i * Self::BITS);
            }

            if byte == byte_match && length >= Self::BITS {
                start_index = if count == 0 { i * Self::BITS } else { start_index };
                count += Self::BITS;
                if count >= length {
                    return Some(start_index);
                }
                continue;
            }

            for j in 0..Self::BITS {
                if byte.get_bit(j) == value {
                    start_index = if count == 0 { i * Self::BITS + j } else { start_index };
                    count += 1;
                    if count == length {
                        return Some(start_index);
                    }
                } else {
                    count = 0;
                }
            }
        }

        None
    }
}
