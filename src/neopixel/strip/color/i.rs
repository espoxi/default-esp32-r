
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

    pub fn to_Color(&self) -> Color {
        Color {
            red: self.red as f32 / 255.0,
            green: self.green as f32 / 255.0,
            blue: self.blue as f32 / 255.0,
        }
    }

    pub fn from_Color(Color: Color) -> Self {
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
        let red = (hex >> 16) & 0xFF;
        let green = (hex >> 8) & 0xFF;
        let blue = hex & 0xFF;
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
    fn to_hsv(&self) -> Hsv {
        let r = self.red;
        let g = self.green;
        let b = self.blue;

        let max = fmax!(r, g, b);
        let min = fmin!(r, g, b);
        let delta = (max - min) as f32;

        let hue = if delta == 0.0 {
            0.0
        } else if max == r {
            60.0 * ((g - b) / delta)
        } else if max == g {
            60.0 * ((b - r) / delta) + 120f32
        } else {
            60.0 * ((r - g) / delta) + 240f32
        }
        .rem_euclid(360f32);

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
        // print!("rgb: {:?}, \t hsv: {:?}", self, hsv);
        hsv.hue += hue;
        *self = hsv.to_rgb();
        // println!("\t-(hue+{})--> rgb: {:?}, \t hsv: {:?}", hue, self, hsv);
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
        let (r, g, b) = match order {
            LedColorOrder::RGB => (self.red, self.green, self.blue),
            LedColorOrder::GRB => (self.green, self.red, self.blue),
        };

        (((r * 255.0) as u32) << 16) | (((g * 255.0) as u32) << 8) | (b * 255.0) as u32
    }

    pub fn to_bit_iter(&self, order: &LedColorOrder) -> impl Iterator<Item = bool> + '_ {
        ColorBitString::new(self.to_u32(order))
    }
}

///HSV color space (Hue, Saturation, Value)
/// Hue: 0-360
/// Saturation: 0-1
/// Value: 0-1
#[derive(Debug, Copy, Clone, PartialEq)]
struct IHsv {
    hue: f32,
    saturation: f32,
    value: f32,
}

#[allow(dead_code)]
impl IHsv {
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
        let h = self.hue.rem_euclid(360.0);
        let s = self.saturation.min(1.0).max(0.0);
        let v = self.value.min(1.0).max(0.0);

        let c = v * s;
        let x = c * (1.0 - ((h / 60.0).rem_euclid(2.0) - 1.0).abs());
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
            red: (self.red + _rhs.red).rem_euclid(1.0),
            green: (self.green + _rhs.green).rem_euclid(1.0),
            blue: (self.blue + _rhs.blue).rem_euclid(1.0),
        }
    }
}

impl ops::Sub<Color> for Color {
    type Output = Color;

    fn sub(self, _rhs: Color) -> Color {
        Color {
            red: (self.red - _rhs.red).rem_euclid(1.0000001),
            green: (self.green - _rhs.green).rem_euclid(1.0000001),
            blue: (self.blue - _rhs.blue).rem_euclid(1.0000001),
        }
    }
}

impl ops::Mul<Color> for Color {
    type Output = Color;

    fn mul(self, _rhs: Color) -> Color {
        Color {
            red: (self.red * _rhs.red).max(0.0).min(1.0),
            green: (self.green * _rhs.green).max(0.0).min(1.0),
            blue: (self.blue * _rhs.blue).max(0.0).min(1.0),
        }
    }
}

impl ops::Mul<f32> for Color {
    type Output = Color;

    fn mul(self, _rhs: f32) -> Color {
        Color {
            red: (self.red * _rhs).max(0.0).min(1.0),
            green: (self.green * _rhs).max(0.0).min(1.0),
            blue: (self.blue * _rhs).max(0.0).min(1.0),
        }
    }
}