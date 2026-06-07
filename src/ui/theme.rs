use crate::model::PeopleKind;
use ratatui::style::Color;

const ARCHIVE_RED: Color = Color::Rgb(0x7a, 0x2e, 0x1d);
const INK: Color = Color::Rgb(0x3a, 0x2a, 0x1a);
const DARK_INK: Color = Color::Rgb(0x6a, 0x5a, 0x4a);
const WARM_BROWN: Color = Color::Rgb(0x8b, 0x73, 0x55);
const DARK_BROWN: Color = Color::Rgb(0x5a, 0x4a, 0x3a);
const PAPER: Color = Color::Rgb(0xef, 0xe9, 0xdd);

const NEED_LOW: Color = Color::Rgb(0x6b, 0x8e, 0x4a);
const NEED_MID: Color = Color::Rgb(0xc2, 0x9a, 0x6b);
const NEED_HIGH: Color = Color::Rgb(0x7a, 0x2e, 0x1d);

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct Theme {
    pub monochrome: bool,
}

impl Theme {
    pub fn people_color(&self, people: PeopleKind) -> Color {
        if self.monochrome {
            return INK;
        }
        match people {
            PeopleKind::Metsik => Color::Rgb(0x3a, 0x5a, 0x2a),
            PeopleKind::Arkit => Color::Rgb(0x5a, 0x6a, 0x8a),
            PeopleKind::Vayla => Color::Rgb(0x4a, 0x7a, 0x9e),
            PeopleKind::Laakso => Color::Rgb(0x8a, 0x8a, 0x7a),
            PeopleKind::Sepat => Color::Rgb(0xb8, 0x73, 0x33),
            PeopleKind::Ahjo => Color::Rgb(0x96, 0x4a, 0x3a),
            PeopleKind::Varhaiset => Color::Rgb(0x5a, 0x7a, 0x4a),
            PeopleKind::Metsareunat => Color::Rgb(0x3a, 0x6a, 0x2a),
            PeopleKind::Porokansa => Color::Rgb(0x4a, 0x6a, 0x3a),
            PeopleKind::Koskimetsa => Color::Rgb(0x2a, 0x5a, 0x5a),
            PeopleKind::Muistikansa => Color::Rgb(0x6a, 0x5a, 0x4a),
            PeopleKind::Taulukansa => Color::Rgb(0x7a, 0x8a, 0x5a),
            PeopleKind::Kirjakansa => Color::Rgb(0x5a, 0x6a, 0x5a),
            PeopleKind::Takovaki => Color::Rgb(0x8a, 0x5a, 0x2a),
            PeopleKind::Rantavaki => Color::Rgb(0x4a, 0x7a, 0x8a),
            PeopleKind::Saarivaki => Color::Rgb(0x3a, 0x6a, 0x9a),
            PeopleKind::Hiekkakavelijat => Color::Rgb(0x9a, 0x8a, 0x5a),
            PeopleKind::Haramaki => Color::Rgb(0x7a, 0x6a, 0x4a),
            PeopleKind::Jamavaki => Color::Rgb(0x5a, 0x5a, 0x4a),
            PeopleKind::Pohjavaki => Color::Rgb(0x4a, 0x5a, 0x4a),
            PeopleKind::Tzakhar => Color::Rgb(0x5a, 0x4a, 0x6a),
            PeopleKind::Merak => Color::Rgb(0x3a, 0x7a, 0x9a),
            PeopleKind::Shear => Color::Rgb(0x9a, 0x8a, 0x6a),
            PeopleKind::Hal => Color::Rgb(0x2a, 0x7a, 0x3a),
            PeopleKind::Khor => Color::Rgb(0x6a, 0x7a, 0x9a),
        }
    }

    pub fn region_color(&self, region_type: &str) -> Color {
        if self.monochrome {
            return DARK_INK;
        }
        match region_type {
            "river_valley" => Color::Rgb(0x4a, 0x7a, 0x4a),
            "coast" => Color::Rgb(0x3a, 0x6a, 0x9a),
            "forest" => Color::Rgb(0x2a, 0x5a, 0x2a),
            "upland" => Color::Rgb(0x7a, 0x6a, 0x4a),
            "steppe" => Color::Rgb(0x9a, 0x8a, 0x5a),
            "delta" => Color::Rgb(0x3a, 0x7a, 0x7a),
            "mountain" => Color::Rgb(0x6a, 0x6a, 0x6a),
            "swamp" => Color::Rgb(0x4a, 0x5a, 0x3a),
            "deep_desert" => Color::Rgb(0x8a, 0x7a, 0x5a),
            "tundra" => Color::Rgb(0x6a, 0x8a, 0x9a),
            "cave" => Color::Rgb(0x4a, 0x3a, 0x5a),
            _ => DARK_BROWN,
        }
    }

    pub fn need_color(&self, val: f64) -> Color {
        if self.monochrome {
            if val >= 0.7 {
                return INK;
            } else if val >= 0.3 {
                return DARK_INK;
            } else {
                return ARCHIVE_RED;
            }
        }
        if val >= 0.7 {
            NEED_LOW
        } else if val >= 0.3 {
            NEED_MID
        } else {
            NEED_HIGH
        }
    }

    pub fn archive_red(&self) -> Color {
        ARCHIVE_RED
    }

    pub fn ink(&self) -> Color {
        INK
    }

    pub fn dark_ink(&self) -> Color {
        DARK_INK
    }

    pub fn warm_brown(&self) -> Color {
        WARM_BROWN
    }

    pub fn dark_brown(&self) -> Color {
        DARK_BROWN
    }

    pub fn paper(&self) -> Color {
        PAPER
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn monochrome_returns_ink_for_all_people() {
        let theme = Theme { monochrome: true };
        for people in [
            PeopleKind::Metsik,
            PeopleKind::Arkit,
            PeopleKind::Vayla,
            PeopleKind::Sepat,
            PeopleKind::Ahjo,
        ] {
            assert_eq!(theme.people_color(people), INK);
        }
    }

    #[test]
    fn monochrome_returns_ink_for_all_regions() {
        let theme = Theme { monochrome: true };
        for rt in ["river_valley", "coast", "forest", "upland"] {
            assert_eq!(theme.region_color(rt), DARK_INK);
        }
    }

    #[test]
    fn monochrome_need_colors_use_only_ink_shades() {
        let theme = Theme { monochrome: true };
        assert_eq!(theme.need_color(0.8), INK);
        assert_eq!(theme.need_color(0.5), DARK_INK);
        assert_eq!(theme.need_color(0.1), ARCHIVE_RED);
    }

    #[test]
    fn color_theme_distinct_for_major_peoples() {
        let theme = Theme::default();
        let metsik = theme.people_color(PeopleKind::Metsik);
        let sepat = theme.people_color(PeopleKind::Sepat);
        let vayla = theme.people_color(PeopleKind::Vayla);
        assert_ne!(metsik, sepat);
        assert_ne!(metsik, vayla);
        assert_ne!(sepat, vayla);
    }

    #[test]
    fn region_colors_distinct_for_major_types() {
        let theme = Theme::default();
        let river = theme.region_color("river_valley");
        let coast = theme.region_color("coast");
        let forest = theme.region_color("forest");
        assert_ne!(river, coast);
        assert_ne!(river, forest);
        assert_ne!(coast, forest);
    }

    #[test]
    fn default_theme_is_not_monochrome() {
        let theme = Theme::default();
        assert!(!theme.monochrome);
    }
}
