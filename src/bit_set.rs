pub struct BitSet {
    content: Vec<u32>,
    size: usize,
}

impl BitSet {
    pub fn with_capacity(bits: usize) -> BitSet {
        if bits % 32 == 0 {
            BitSet {
                content: vec![0u32; bits / 32],
                size: bits,
            }
        } else {
            BitSet {
                content: vec![0u32; bits / 32 + 1],
                size: bits,
            }
        }
    }

    #[inline]
    pub fn get(&self, i: usize) -> Option<bool> {
        if i >= self.size {
            return None;
        }
        let w = i / 32;
        let b = i % 32;
        self.content
            .get(w)
            .map(|&block| (block & (1u32 << b)) != 0u32)
    }

    #[inline]
    pub fn set(&mut self, i: usize, x: bool) {
        assert!(
            i < self.size,
            "index out of bounds: {:?} >= {:?}",
            i,
            self.size
        );
        let w = i / 32;
        let b = i % 32;
        let flag = 1u32 << b;
        let val = if x {
            self.content[w] | flag
        } else {
            self.content[w] & !flag
        };
        self.content[w] = val;
    }

    pub fn first(&self) {}
}
