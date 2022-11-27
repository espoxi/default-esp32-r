use std::{
    cmp::{max, min},
    ops,
};

macro_rules! min {
    ($x: expr) => ($x);
    ($x: expr, $($z: expr),+) => (::std::cmp::min($x, min!($($z),*)));
}
macro_rules! max {
    ($x: expr) => ($x);
    ($x: expr, $($z: expr),+) => (::std::cmp::max($x, max!($($z),*)));
}

use super::LedColorOrder;

#[derive(Debug, Copy, Clone, PartialEq, Eq)]
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

    /// convert to HSV
    fn to_hsv(&self) -> Hsv {
        let r = self.red as i16;
        let g = self.green as i16;
        let b = self.blue as i16;

        let max = max!(r, g, b);
        let min = min!(r, g, b);
        let delta = (max - min) as f32;

        let hue = if delta <= 1f32 {
            0
        } else if max == r {
            (42.5f32 * ((g - b)as f32 / delta)%255f32) as u8
        } else if max == g {
            (42.5f32 * ((b - r)as f32 / delta)) as u8 + 85
        } else {
            (42.5f32 * ((r - g)as f32 / delta)) as u8 + 170
        };

        let value = max as u8;

        let saturation = if max == 0 { 0 } else { 255u16 * delta as u16 / max as u16 } as u8;

        Hsv {
            hue: hue,
            saturation: saturation,
            value: value,
        }
    }

    /// shift hue
    /// hue: i16, -360 to 360
    pub fn shift_hue(&mut self, hue: i16) -> &Self {
        let hsv = self.to_hsv();
        let new_hue = hsv.hue as i16 + (hue as i16) *17/24; //255/360
        let new_hsv = Hsv::new((new_hue%255) as u8, hsv.saturation, hsv.value);
        let new = new_hsv.to_rgb();
        println!("{:?}[{:?}] --({}->{})-> [{:?}]{:?}",self, hsv, hue, new_hue%255, new_hsv, new);
        (self.red, self.green, self.blue) = (new.red, new.green, new.blue);
        self
    }

    /// shift saturation
    /// saturation: i8, -100 to 100
    pub fn shift_saturation(&mut self, percent: i8) -> &Self {
        let hsv = self.to_hsv();
        let mut new_saturation = hsv.saturation as i16 + (percent as i16 * 51/20); //255/100
        if new_saturation < 0 {
            new_saturation = 0;
        } else if new_saturation > 255 {
            new_saturation = 255;
        }
        let new = Hsv::new(hsv.hue, new_saturation as u8, hsv.value).to_rgb();
        (self.red, self.green, self.blue) = (new.red, new.green, new.blue);
        self
    }

    /// shift value
    /// value: i16, -100 to 100
    pub fn shift_value(&mut self, percent: i8) -> &Self {
        let hsv = self.to_hsv();
        let mut new_value = hsv.value as i16 + (percent as i16  * 51/20); //255/100
        if new_value < 0 {
            new_value = 0;
        } else if new_value > 255 {
            new_value = 255;
        }
        let new = Hsv::new(hsv.hue, hsv.saturation, new_value as u8).to_rgb();
        (self.red, self.green, self.blue) = (new.red, new.green, new.blue);
        self
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
        let bit = (self.color_u32 >> self.current_bit_pos)&1 == 1;
        Some(bit)
    }
}

///HSV color space values range from 0 to 255
#[derive(Debug, Copy, Clone, PartialEq, Eq)]
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
        let h = self.hue as f32 / 255.0 * 360.0;
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
