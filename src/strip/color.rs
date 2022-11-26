use std::{
    cmp::{max, min},
    ops,
};

use super::LedColorOrder;

pub struct Color {
    pub red: u8,
    pub green: u8,
    pub blue: u8,
}

#[allow(dead_code)]
impl Color {
    pub fn new(red: u8, green: u8, blue: u8) -> Self {
        Self { red, green, blue }
    }

    pub fn black() -> Self {
        Self::new(0, 0, 0)
    }

    pub fn white() -> Self {
        Self::new(255, 255, 255)
    }

    pub fn red() -> Self {
        Self::new(255, 0, 0)
    }

    pub fn green() -> Self {
        Self::new(0, 255, 0)
    }

    pub fn blue() -> Self {
        Self::new(0, 0, 255)
    }

    pub fn yellow() -> Self {
        Self::new(255, 255, 0)
    }

    pub fn cyan() -> Self {
        Self::new(0, 255, 255)
    }

    pub fn magenta() -> Self {
        Self::new(255, 0, 255)
    }

    pub fn orange() -> Self {
        Self::new(255, 165, 0)
    }

    pub fn purple() -> Self {
        Self::new(128, 0, 128)
    }

    pub fn pink() -> Self {
        Self::new(255, 192, 203)
    }

    pub fn brown() -> Self {
        Self::new(165, 42, 42)
    }

    pub fn gray() -> Self {
        Self::new(128, 128, 128)
    }

    pub fn silver() -> Self {
        Self::new(192, 192, 192)
    }

    pub fn gold() -> Self {
        Self::new(255, 215, 0)
    }

    /// convert to HSV without using floating point math
    fn to_hsv(&self) -> Hsv {
        let r = self.red as u16;
        let g = self.green as u16;
        let b = self.blue as u16;

        let max = max(r, max(g, b));
        let min = min(r, min(g, b));
        let delta = max - min;

        let hue = if delta == 0 {
            0
        } else if max == r {
            60 * ((g - b) / delta)
        } else if max == g {
            60 * (2 + (b - r) / delta)
        } else {
            60 * (4 + (r - g) / delta)
        };

        let value = max;

        let saturation = if max == 0 { 0 } else { 255 * delta / max };

        Hsv {
            hue: hue as u8,
            saturation: saturation as u8,
            value: value as u8,
        }
    }

    /// shift hue
    /// hue: i16, -255 to 255
    pub fn shift_hue(&self, hue: i16) -> Self {
        let hsv = self.to_hsv();
        let mut new_hue = hsv.hue as i16 + hue;
        if new_hue < 0 {
            new_hue += 256;
        } else if new_hue > 255 {
            new_hue -= 256;
        }
        Hsv::new(new_hue as u8, hsv.saturation, hsv.value).to_rgb()
    }

    /// shift saturation
    /// saturation: i16, -255 to 255
    pub fn shift_saturation(&self, saturation: i16) -> Self {
        let hsv = self.to_hsv();
        let mut new_saturation = hsv.saturation as i16 + saturation;
        if new_saturation < 0 {
            new_saturation = 0;
        } else if new_saturation > 255 {
            new_saturation = 255;
        }
        Hsv::new(hsv.hue, new_saturation as u8, hsv.value).to_rgb()
    }

    /// shift value
    /// value: i16, -255 to 255
    pub fn shift_value(&self, value: i16) -> Self {
        let hsv = self.to_hsv();
        let mut new_value = hsv.value as i16 + value;
        if new_value < 0 {
            new_value = 0;
        } else if new_value > 255 {
            new_value = 255;
        }
        Hsv::new(hsv.hue, hsv.saturation, new_value as u8).to_rgb()
    }

    /// Returns the color as a 24-bit RGB value.
    pub fn rgb(&self, order: &LedColorOrder) -> u32 {
        match order {
            LedColorOrder::RGB => {
                ((self.red as u32) << 16) | ((self.green as u32) << 8) | self.blue as u32
            }
            LedColorOrder::GRB => {
                ((self.green as u32) << 16) | ((self.red as u32) << 8) | self.blue as u32
            }
        }
    }

    pub fn to_bit_iter(&self, order: &LedColorOrder) -> impl Iterator<Item = bool> + '_ {
        ColorBitString::new(self.rgb(order))
    }
}

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
        let bit = (self.color_u32 >> self.current_bit_pos) == 1;
        Some(bit)
    }
}

///HSV color space values range from 0 to 255
struct Hsv {
    hue: u8,
    saturation: u8,
    value: u8,
}

impl Hsv {
    fn new(hue: u8, saturation: u8, value: u8) -> Self {
        Self {
            hue,
            saturation,
            value,
        }
    }
    pub fn to_rgb(&self) -> Color {
        let h = self.hue as f32 / 255.0;
        let s = self.saturation as f32 / 255.0;
        let v = self.value as f32 / 255.0;

        let c = v * s;
        let x = c * (1.0 - ((h / 60.0) % 2.0 - 1.0).abs());
        let m = v - c;

        let (r, g, b) = match h {
            h if h < 60.0 => (c, x, 0.0),
            h if h < 120.0 => (x, c, 0.0),
            h if h < 180.0 => (0.0, c, x),
            h if h < 240.0 => (0.0, x, c),
            h if h < 300.0 => (x, 0.0, c),
            _ => (c, 0.0, x),
        };

        Color {
            red: ((r + m) * 255.0) as u8,
            green: ((g + m) * 255.0) as u8,
            blue: ((b + m) * 255.0) as u8,
        }
    }
}

impl ops::Add<Color> for Color {
    type Output = Color;

    fn add(self, _rhs: Color) -> Color {
        Color {
            red: min(self.red + _rhs.red, 255),
            green: min(self.green + _rhs.green, 255),
            blue: min(self.blue + _rhs.blue, 255),
        }
    }
}

impl ops::Sub<Color> for Color {
    type Output = Color;

    fn sub(self, _rhs: Color) -> Color {
        Color {
            red: max(self.red - _rhs.red, 0),
            green: max(self.green - _rhs.green, 0),
            blue: max(self.blue - _rhs.blue, 0),
        }
    }
}

impl ops::Mul<Color> for Color {
    type Output = Color;

    fn mul(self, _rhs: Color) -> Color {
        Color {
            red: min(self.red * _rhs.red / 255, 255),
            green: min(self.green * _rhs.green / 255, 255),
            blue: min(self.blue * _rhs.blue / 255, 255),
        }
    }
}

impl ops::Mul<f32> for Color {
    type Output = Color;

    fn mul(self, _rhs: f32) -> Color {
        Color {
            red: min((self.red as f32 * _rhs) as u8, 255),
            green: min((self.green as f32 * _rhs) as u8, 255),
            blue: min((self.blue as f32 * _rhs) as u8, 255),
        }
    }
}
