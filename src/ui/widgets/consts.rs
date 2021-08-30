use druid::*;

use super::section::SectionDecal;

pub const BG_COLOR: Color = Color::rgb8(221, 206, 187);
pub const SUB_SECTION_PRIMARY_COLOR: Color = Color::rgb8(209, 160, 68);
pub const SUB_SECTION_SECONDARY_COLOR: Color = Color::rgb8(163, 113, 55);
pub const MAIN_SECTION_PRIMARY_COLOR: Color = Color::rgb8(241, 195, 109);
pub const MAIN_SECTION_SECONDARY_COLOR: Color = Color::rgb8(51, 34, 31);
pub const BORDER_COLOR: Color = SUB_SECTION_SECONDARY_COLOR;
pub const BORDER_WIDTH: f64 = 2.;
pub const MAIN_SECTION_FONT_SIZE: f64 = 30.;
pub const SUB_SECTION_FONT_SIZE: f64 = 20.;
pub const SECTION_PADDING: f64 = 5.;
pub const SECTION_RADIUS: f64 = 5.;
pub const INPUT_BG_COLOR: Color = Color::rgb8(58, 54, 55);

pub const MAIN_SECTION_DECAL: SectionDecal = SectionDecal {
    font_size: MAIN_SECTION_FONT_SIZE,
    primary_color: &MAIN_SECTION_PRIMARY_COLOR,
    secondary_color: &MAIN_SECTION_SECONDARY_COLOR,
};

pub const SUB_SECTION_DECAL: SectionDecal = SectionDecal {
    font_size: SUB_SECTION_FONT_SIZE,
    primary_color: &SUB_SECTION_PRIMARY_COLOR,
    secondary_color: &SUB_SECTION_SECONDARY_COLOR,
};