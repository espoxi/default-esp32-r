
//XXX: mod i needs some work
pub mod i;
pub mod f;
pub use i as default;

struct ColorBitString {
    color_u32: u32,
    current_bit_pos: u8,
}
impl ColorBitString {
    fn new(color_u32: u32) -> Self {
        let current_bit_pos = 24;
        Self {
            color_u32,
            current_bit_pos,
        }
    }
}

impl Iterator for ColorBitString {
    type Item = bool;
    fn next(&mut self) -> Option<Self::Item> {
        if self.current_bit_pos == 0 {
            return None;
        }
        self.current_bit_pos -= 1;
        let bit = (self.color_u32 >> self.current_bit_pos) & 1 == 1;
        Some(bit)
    }
}