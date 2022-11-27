use std::{
    ops,
};

#[allow(unused_macros)]
macro_rules! min {
    ($x: expr) => ($x);
    ($x: expr, $($z: expr),+) => (::std::cmp::min($x, min!($($z),*)));
}

#[allow(unused_macros)]
macro_rules! max {
    ($x: expr) => ($x);
    ($x: expr, $($z: expr),+) => (::std::cmp::max($x, max!($($z),*)));
}

macro_rules! fmin {
    ($x: expr) => ($x);
    ($x: expr, $($z: expr),+) => ($x.min(fmin!($($z),*)));
}
macro_rules! fmax {
    ($x: expr) => ($x);
    ($x: expr, $($z: expr),+) => ($x.max(fmax!($($z),*)));
}

use super::LedColorOrder;

#[derive(Debug, Copy, Clone, PartialEq)]
pub struct Color {
    pub red: f32,
    pub green: f32,
    pub blue: f32,
}

#[allow(dead_code)]
impl Color {
    pub fn from_u8(red: u8, green: u8, blue: u8) -> Self {
        Self {
            red: red as f32 / 255.0,
            green: green as f32 / 255.0,
            blue: blue as f32 / 255.0,
        }
    }

    /// values should range from 0 to 1
    pub fn new(red: f32, green: f32, blue: f32) -> Self {
        // assert!(red >= 0.0 && red <= 1.0);
        // assert!(green >= 0.0 && green <= 1.0);
        // assert!(blue >= 0.0 && blue <= 1.0);
        Self {
            red: red,
            green: green,
            blue: blue,
        }
    }

    pub fn from_hex(hex: u32, order: &LedColorOrder) -> Self {
        let red = ((hex >> 16) & 0xFF) as f32 / 255.0;
        let green = ((hex >> 8) & 0xFF) as f32 / 255.0;
        let blue = (hex & 0xFF) as f32 / 255.0;
        match order {
            LedColorOrder::RGB => Self::new(red, green, blue),
            LedColorOrder::GRB => Self::new(green, red, blue),
        }
    }

    pub fn black() -> Self {
        Self::from_u8(0, 0, 0)
    }

    pub fn white() -> Self {
        Self::from_u8(255, 255, 255)
    }

    pub fn red() -> Self {
        Self::from_u8(255, 0, 0)
    }

    pub fn green() -> Self {
        Self::from_u8(0, 255, 0)
    }

    pub fn blue() -> Self {
        Self::from_u8(0, 0, 255)
    }

    pub fn yellow() -> Self {
        Self::from_u8(255, 255, 0)
    }

    pub fn cyan() -> Self {
        Self::from_u8(0, 255, 255)
    }

    pub fn magenta() -> Self {
        Self::from_u8(255, 0, 255)
    }

    pub fn orange() -> Self {
        Self::from_u8(255, 165, 0)
    }

    pub fn purple() -> Self {
        Self::from_u8(128, 0, 128)
    }

    pub fn pink() -> Self {
        Self::from_u8(255, 192, 203)
    }

    pub fn brown() -> Self {
        Self::from_u8(165, 42, 42)
    }

    pub fn gray() -> Self {
        Self::from_u8(128, 128, 128)
    }

    pub fn silver() -> Self {
        Self::from_u8(192, 192, 192)
    }

    pub fn gold() -> Self {
        Self::from_u8(255, 215, 0)
    }

    /// convert to HSV
    fn to_hsv(&self) -> Hsv {
        let r = self.red;
        let g = self.green;
        let b = self.blue;

        let max = fmax!(r, g, b);
        let min = fmin!(r, g, b);
        let delta = (max - min) as f32;

        let hue = if delta <= 1f32 {
            0.0
        } else if max == r {
            60.0 * ((g - b) / delta) % 360f32
        } else if max == g {
            60.0 * ((b - r) / delta) + 120f32
        } else {
            60.0 * ((r - g) / delta) + 240f32
        };

        let value = max;

        let saturation = if max <= 0.1 { 0.0 } else { delta / max };

        Hsv {
            hue: hue,
            saturation: saturation,
            value: value,
        }
    }

    /// shift hue
    /// hue: f32, range from 0 to 360
    pub fn shift_hue_deg(&mut self, hue: f32) -> &Self {
        let mut hsv = self.to_hsv();
        hsv.hue = (hsv.hue + hue) % 360f32;
        *self = hsv.to_rgb();
        self
    }

    /// shift saturation
    /// saturation: f32, range from -1 to 1
    pub fn shift_saturation(&mut self, percent: f32) -> &Self {
        let mut hsv = self.to_hsv();
        hsv.saturation = (hsv.saturation + percent).max(0.0).min(1.0);
        *self = hsv.to_rgb();
        self
    }

    /// shift value
    /// value: f32, range from -1 to 1
    pub fn shift_value(&mut self, percent: f32) -> &Self {
        let mut hsv = self.to_hsv();
        hsv.value = (hsv.value + percent).max(0.0).min(1.0);
        *self = hsv.to_rgb();
        self
    }

    /// Returns the color as a 24-bit RGB value.
    pub fn to_u32(&self, order: &LedColorOrder) -> u32 {
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
        ColorBitString::new(self.to_u32(order))
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
        let bit = (self.color_u32 >> self.current_bit_pos) & 1 == 1;
        Some(bit)
    }
}

///HSV color space (Hue, Saturation, Value)
/// Hue: 0-360
/// Saturation: 0-1
/// Value: 0-1
#[derive(Debug, Copy, Clone, PartialEq)]
struct Hsv {
    hue: f32,
    saturation: f32,
    value: f32,
}

#[allow(dead_code)]
impl Hsv {
    ///HSV color space (Hue, Saturation, Value)
    /// Hue: 0-360
    /// Saturation: 0-1
    /// Value: 0-1
    fn new(hue: f32, saturation: f32, value: f32) -> Self {
        Self {
            hue,
            saturation,
            value,
        }
    }
    pub fn to_rgb(&self) -> Color {
        let h = self.hue;
        let s = self.saturation;
        let v = self.value;

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

        Color::new(r + m, g + m, b + m)
    }
}

impl ops::Add<Color> for Color {
    type Output = Color;

    fn add(self, _rhs: Color) -> Color {
        Color {
            red: (self.red + _rhs.red) % 1.0,
            green: (self.green + _rhs.green) % 1.0,
            blue: (self.blue + _rhs.blue) % 1.0,
        }
    }
}

impl ops::Sub<Color> for Color {
    type Output = Color;

    fn sub(self, _rhs: Color) -> Color {
        Color {
            red: (self.red - _rhs.red) % 1.0,
            green: (self.green - _rhs.green) % 1.0,
            blue: (self.blue - _rhs.blue) % 1.0,
        }
    }
}

impl ops::Mul<Color> for Color {
    type Output = Color;

    fn mul(self, _rhs: Color) -> Color {
        Color {
            red: (self.red * _rhs.red) % 1.0,
            green: (self.green * _rhs.green) % 1.0,
            blue: (self.blue * _rhs.blue) % 1.0,
        }
    }
}

impl ops::Mul<f32> for Color {
    type Output = Color;

    fn mul(self, _rhs: f32) -> Color {
        Color {
            red: (self.red * _rhs) % 1.0,
            green: (self.green * _rhs) % 1.0,
            blue: (self.blue * _rhs) % 1.0,
        }
    }
}
