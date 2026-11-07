//! Pure physical-pixel geometry and HDR-to-SDR image conversion.

#[derive(Clone, Copy, Debug, PartialEq)]
pub(super) struct SelectionRect {
    pub left: u32,
    pub top: u32,
    pub width: u32,
    pub height: u32,
}

impl SelectionRect {
    pub(super) fn from_points(a: (i32, i32), b: (i32, i32), width: u32, height: u32) -> Self {
        let left = a.0.min(b.0).clamp(0, width as i32) as u32;
        let top = a.1.min(b.1).clamp(0, height as i32) as u32;
        let right = a.0.max(b.0).clamp(0, width as i32) as u32;
        let bottom = a.1.max(b.1).clamp(0, height as i32) as u32;
        Self {
            left,
            top,
            width: right - left,
            height: bottom - top,
        }
    }
    pub(super) fn adjust(&mut self, dx: i32, dy: i32, resize: bool, width: u32, height: u32) {
        if resize {
            self.width =
                (self.width as i32 + dx).clamp(1, (width - self.left).max(1) as i32) as u32;
            self.height =
                (self.height as i32 + dy).clamp(1, (height - self.top).max(1) as i32) as u32;
        } else {
            self.left = (self.left as i32 + dx).clamp(0, (width - self.width) as i32) as u32;
            self.top = (self.top as i32 + dy).clamp(0, (height - self.height) as i32) as u32;
        }
    }
}

pub(super) struct Image {
    pub width: u32,
    pub height: u32,
    pub pixels: Vec<u8>,
}

impl Image {
    pub(super) fn crop(&self, rect: SelectionRect) -> Result<Self, String> {
        if rect.width == 0
            || rect.height == 0
            || rect
                .left
                .checked_add(rect.width)
                .map_or(true, |x| x > self.width)
            || rect
                .top
                .checked_add(rect.height)
                .map_or(true, |y| y > self.height)
        {
            return Err("Select a non-empty area inside the display.".into());
        }
        let mut pixels = Vec::with_capacity(rect.width as usize * rect.height as usize * 4);
        for y in rect.top..rect.top + rect.height {
            let start = (y as usize * self.width as usize + rect.left as usize) * 4;
            pixels.extend_from_slice(&self.pixels[start..start + rect.width as usize * 4]);
        }
        Ok(Self {
            width: rect.width,
            height: rect.height,
            pixels,
        })
    }
}

pub(super) fn half_to_float(value: u16) -> f32 {
    let sign = if value & 0x8000 != 0 { -1.0 } else { 1.0 };
    let exponent = (value >> 10) & 0x1f;
    let mantissa = value & 0x3ff;
    match exponent {
        0 => sign * (mantissa as f32) * 2f32.powi(-24),
        31 => {
            if mantissa == 0 {
                sign * f32::INFINITY
            } else {
                f32::NAN
            }
        }
        _ => sign * (1.0 + mantissa as f32 / 1024.0) * 2f32.powi(exponent as i32 - 15),
    }
}

pub(super) fn linear_to_sdr(value: f32) -> u8 {
    // SDR content is unchanged through 80% linear white. A smooth shoulder maps
    // HDR highlights into SDR clipboard range rather than wrapping/clipping them.
    let value = if value.is_nan() { 0.0 } else { value.max(0.0) };
    let value = if value > 0.8 {
        0.8 + 0.2 * (1.0 - (-(value - 0.8) / 0.2).exp())
    } else {
        value
    };
    let srgb = if value <= 0.0031308 {
        value * 12.92
    } else {
        1.055 * value.powf(1.0 / 2.4) - 0.055
    };
    (srgb.clamp(0.0, 1.0) * 255.0).round() as u8
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn cropping_uses_physical_rows_and_rejects_out_of_bounds() {
        let image = Image {
            width: 3,
            height: 2,
            pixels: (0..24).collect(),
        };
        let cropped = image
            .crop(SelectionRect {
                left: 1,
                top: 0,
                width: 2,
                height: 2,
            })
            .unwrap();
        assert_eq!(
            cropped.pixels,
            [4, 5, 6, 7, 8, 9, 10, 11, 16, 17, 18, 19, 20, 21, 22, 23]
        );
        assert!(image
            .crop(SelectionRect {
                left: 2,
                top: 0,
                width: 2,
                height: 1
            })
            .is_err());
    }
    #[test]
    fn hdr_conversion_is_bounded_and_handles_half_floats() {
        assert_eq!(half_to_float(0x3c00), 1.0);
        assert_eq!(half_to_float(0xc000), -2.0);
        assert_eq!(linear_to_sdr(0.0), 0);
        assert!(linear_to_sdr(0.5) > 127);
        assert_eq!(linear_to_sdr(f32::INFINITY), 255);
        assert_eq!(linear_to_sdr(f32::NAN), 0);
    }
    #[test]
    fn selection_normalizes_drag_direction_and_clamps_to_display() {
        assert_eq!(
            SelectionRect::from_points((100, 80), (-4, 20), 200, 100),
            SelectionRect {
                left: 0,
                top: 20,
                width: 100,
                height: 60
            }
        );
        assert_eq!(
            SelectionRect::from_points((20, 30), (240, 160), 200, 100),
            SelectionRect {
                left: 20,
                top: 30,
                width: 180,
                height: 70
            }
        );
    }
    #[test]
    fn keyboard_moving_and_resizing_stay_within_display() {
        let mut rectangle = SelectionRect {
            left: 10,
            top: 20,
            width: 50,
            height: 40,
        };
        rectangle.adjust(-20, 200, false, 100, 100);
        assert_eq!(
            rectangle,
            SelectionRect {
                left: 0,
                top: 60,
                width: 50,
                height: 40
            }
        );
        rectangle.adjust(200, -100, true, 100, 100);
        assert_eq!(rectangle.width, 100);
        assert_eq!(rectangle.height, 1);
    }
}
