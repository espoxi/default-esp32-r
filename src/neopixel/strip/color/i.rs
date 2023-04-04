use serde::{Deserialize, Serialize};

use std::{ops::{self}, convert::TryInto};
use super::{super::LedColorOrder, ColorBitString};


//TODO: make HSV also integer based !

#[derive(Debug, Copy, Clone, PartialEq, Serialize, Deserialize)]
pub struct Color {
    pub red: u8,
    pub green: u8,
    pub blue: u8,
}

#[allow(dead_code)]
impl Color {
    pub fn from_f32(red: f32, green: f32, blue: f32) -> Self {
        Self {
            red: (red * 255.0) as u8,
            green: (green * 255.0) as u8,
            blue: (blue * 255.0) as u8,
        }
    }

    pub fn to_FColor(&self) -> super::f::Color {
        super::f::Color {
            red: self.red as f32 / 255.0,
            green: self.green as f32 / 255.0,
            blue: self.blue as f32 / 255.0,
        }
    }

    pub fn from_FColor(Color: super::f::Color) -> Self {
        Self::from_f32(Color.red, Color.green, Color.blue)
    }

    /// values should range from 0 to 1
    pub fn new(red: u8, green: u8, blue: u8) -> Self {
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
        let red:u8 = ((hex >> 16) & 0xFF).try_into().unwrap();
        let green:u8 = ((hex >> 8) & 0xFF).try_into().unwrap();
        let blue:u8 = (hex & 0xFF).try_into().unwrap();
        match order {
            LedColorOrder::RGB => Self::new(red, green, blue),
            LedColorOrder::GRB => Self::new(green, red, blue),
        }
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
    //TODO: integer only variant
    // fn to_hsv(&self) -> Hsv {
    //     let r = self.red;
    //     let g = self.green;
    //     let b = self.blue;

    //     let max = fmax!(r, g, b);
    //     let min = fmin!(r, g, b);
    //     let delta = (max - min) as f32;

    //     let hue = if delta == 0.0 {
    //         0.0
    //     } else if max == r {
    //         60.0 * ((g - b) / delta)
    //     } else if max == g {
    //         60.0 * ((b - r) / delta) + 120f32
    //     } else {
    //         60.0 * ((r - g) / delta) + 240f32
    //     }
    //     .rem_euclid(360f32);

    //     let value = max;

    //     let saturation = if max <= 0.1 { 0.0 } else { delta / max };

    //     Hsv {
    //         hue: hue,
    //         saturation: saturation,
    //         value: value,
    //     }
    // }

        /// convert to fp HSV
    fn to_fhsv(&self) -> super::f::Hsv {
        self.to_FColor().to_hsv()
    }

    // /// shift hue
    // /// hue: f32, range from 0 to 360
    // pub fn ishift_hue_deg(&mut self, hue: f32) -> &Self {
    //     let mut hsv = self.to_hsv();
    //     // print!("rgb: {:?}, \t hsv: {:?}", self, hsv);
    //     hsv.hue += hue;
    //     *self = hsv.to_rgb();
    //     // println!("\t-(hue+{})--> rgb: {:?}, \t hsv: {:?}", hue, self, hsv);
    //     self
    // }

    // /// shift saturation
    // /// saturation: f32, range from -1 to 1
    // pub fn ishift_saturation(&mut self, percent: f32) -> &Self {
    //     let mut hsv = self.to_hsv();
    //     hsv.saturation = (hsv.saturation + percent).max(0.0).min(1.0);
    //     *self = hsv.to_rgb();
    //     self
    // }

    // /// shift value
    // /// value: f32, range from -1 to 1
    // pub fn ishift_value(&mut self, percent: f32) -> &Self {
    //     let mut hsv = self.to_hsv();
    //     hsv.value = (hsv.value + percent).max(0.0).min(1.0);
    //     *self = hsv.to_rgb();
    //     self
    // }


    /// shift hue
    /// hue: f32, range from 0 to 360
    pub fn shift_hue_deg(&mut self, hue: f32) -> &Self {
        let mut hsv = self.to_fhsv();
        // print!("rgb: {:?}, \t hsv: {:?}", self, hsv);
        hsv.hue += hue;
        *self = Color::from_FColor(hsv.to_rgb());
        // println!("\t-(hue+{})--> rgb: {:?}, \t hsv: {:?}", hue, self, hsv);
        self
    }

    /// shift saturation
    /// saturation: f32, range from -1 to 1
    pub fn shift_saturation(&mut self, percent: f32) -> &Self {
        let mut hsv = self.to_fhsv();
        hsv.saturation = (hsv.saturation + percent).max(0.0).min(1.0);
        *self = Color::from_FColor(hsv.to_rgb());
        self
    }

    /// shift value
    /// value: f32, range from -1 to 1
    pub fn shift_value(&mut self, percent: f32) -> &Self {
        let mut hsv = self.to_fhsv();
        hsv.value = (hsv.value + percent).max(0.0).min(1.0);
        *self = Color::from_FColor(hsv.to_rgb());
        self
    }

    /// Returns the color as a 24-bit RGB value.
    pub fn to_u32(&self, order: &LedColorOrder) -> u32 {
        let (r, g, b) = match order {
            LedColorOrder::RGB => (self.red, self.green, self.blue),
            LedColorOrder::GRB => (self.green, self.red, self.blue),
        };

        ((r as u32) << 16) | ((g as u32) << 8) | b as u32
    }

    pub fn to_bit_iter(&self, order: &LedColorOrder) -> impl Iterator<Item = bool> + '_ {
        ColorBitString::new(self.to_u32(order))
    }
}

///HSV color space (Hue, Saturation, Value)
#[derive(Debug, Copy, Clone, PartialEq)]
struct Hsv {
    hue: u8,
    saturation: u8,
    value: u8,
}

#[allow(dead_code)]
impl Hsv {
    ///HSV color space (Hue, Saturation, Value)
    
    fn new(hue: u8, saturation: u8, value: u8) -> Self {
        Self {
            hue,
            saturation,
            value,
        }
    }
    //TODO: implement same API as f::Hsv but with integers only
}

impl ops::Add<Color> for Color {
    type Output = Color;

    fn add(self, _rhs: Color) -> Color {
        Color {
            red: self.red.wrapping_add(_rhs.red),
            green: self.green.wrapping_add(_rhs.green),
            blue: self.blue.wrapping_add(_rhs.blue),
        }
    }
}

impl ops::Sub<Color> for Color {
    type Output = Color;

    fn sub(self, _rhs: Color) -> Color {
        Color {
            red: self.red.wrapping_sub(_rhs.red),
            green: self.green.wrapping_sub(_rhs.green),
            blue: self.blue.wrapping_sub(_rhs.blue),
        }
    }
}

impl ops::Mul<Color> for Color {
    type Output = Color;

    fn mul(self, _rhs: Color) -> Color {
        Color {
            red: (self.red as u16 * _rhs.red as u16).clamp(0, 255).try_into().unwrap(),
            green: (self.green as u16 * _rhs.green as u16)
                .clamp(0, 255)
                .try_into()
                .unwrap(),
            blue: (self.blue as u16 * _rhs.blue as u16).clamp(0, 255).try_into().unwrap(),
        }
    }
}

impl ops::Mul<f32> for Color {
    type Output = Color;

    fn mul(self, _rhs: f32) -> Color {
        Color {
            red: (self.red as f32 * _rhs).clamp(0.0, 255.0) as u8,
            green: (self.green as f32 * _rhs).clamp(0.0, 255.0) as u8,
            blue: (self.blue as f32 * _rhs).clamp(0.0, 255.0) as u8,
        }
    }
}