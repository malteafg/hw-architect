pub type RGBColor = [f32; 3];
pub type RGBAColor = [f32; 4];

pub const DEFAULT_ALPHA: f32 = 0.8;

pub fn rgba(rgb: RGBColor, alpha: f32) -> RGBAColor {
    [rgb[0], rgb[1], rgb[2], alpha]
}

/// Same as rgba but with default alpha.
pub fn rgba_d(rgb: RGBColor) -> RGBAColor {
    [rgb[0], rgb[1], rgb[2], DEFAULT_ALPHA]
}

pub const DEFAULT: RGBAColor = [1., 1., 1., 1.];

pub const ASPHALT_COLOR: RGBColor = [0.12, 0.12, 0.12];
pub const LANE_MARKINGS_COLOR: RGBColor = [0.95, 0.95, 0.95];

pub const RED: RGBColor = [1.0, 0., 0.1];
pub const LIGHT_BLUE: RGBColor = [0.1, 0.1, 0.6];
pub const GREEN: RGBColor = [0.1, 0.9, 0.2];
