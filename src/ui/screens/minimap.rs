use crate::model::Terrain;
use crate::ui::app::App;
use crate::ui::theme::Theme;
use ratatui::{
    layout::Rect,
    style::{Color, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Paragraph},
    Frame,
};

pub fn render_minimap(f: &mut Frame, app: &App, area: Rect) {
    let theme = Theme {
        monochrome: app.monochrome,
        high_contrast: app.high_contrast,
    };
    let region = match app.sim.as_ref().and_then(|s| s.world.regions.first()) {
        Some(r) => r,
        None => return,
    };

    let width = area.width.saturating_sub(2) as usize;
    let height = area.height.saturating_sub(2) as usize;
    let map_w = region.terrain.width.min(width);
    let map_h = region.terrain.height.min(height);

    let mut lines: Vec<Line> = Vec::new();
    for y in 0..map_h {
        let mut spans: Vec<Span> = Vec::new();
        for x in 0..map_w {
            let idx = y * region.terrain.width + x;
            if idx >= region.terrain.tiles.len() {
                break;
            }
            let terrain = region.terrain.tiles[idx];
            let is_explored = app
                .explored
                .first()
                .map(|e| e.is_explored(x, y))
                .unwrap_or(true);
            if !is_explored {
                spans.push(Span::styled(
                    "░".to_string(),
                    Style::default().fg(theme.dark_brown()),
                ));
            } else {
                let is_settlement = matches!(terrain, Terrain::Settlement);
                let glyph = if is_settlement {
                    '█'
                } else {
                    terrain.glyph()
                };
                let color = if is_settlement {
                    dominant_people_color(&region.settlements, &theme)
                } else {
                    glyph_color(terrain, &theme)
                };
                spans.push(Span::styled(glyph.to_string(), Style::default().fg(color)));
            }
        }
        lines.push(Line::from(spans));
    }

    let paragraph = Paragraph::new(lines).block(
        Block::default()
            .borders(Borders::ALL)
            .title(" Minimap ")
            .border_style(Style::default().fg(theme.archive_red())),
    );
    f.render_widget(paragraph, area);
}

pub(crate) fn dominant_people_color(
    settlements: &[crate::model::Settlement],
    theme: &Theme,
) -> Color {
    use std::collections::HashMap;
    let mut counts: HashMap<String, u32> = HashMap::new();
    for s in settlements {
        for p in &s.people {
            *counts.entry(p.people.clone()).or_insert(0) += 1;
        }
    }
    let dominant = counts.into_iter().max_by_key(|&(_, c)| c).map(|(p, _)| p);

    match dominant.as_deref() {
        Some("Metsik") | Some("Häl") => theme.warm_brown(),
        Some("Sepät") | Some("Ahjo") => theme.archive_red(),
        Some("Väylä") | Some("Mëräk") => theme.warm_brown(),
        Some("Laakso") | Some("Tzäkhar") => theme.dark_brown(),
        Some("She'ar") => theme.warm_brown(),
        Some("Khör") => theme.dark_brown(),
        Some("Arkit") => theme.archive_red(),
        _ => theme.archive_red(),
    }
}

pub(crate) fn glyph_color(terrain: crate::model::Terrain, theme: &Theme) -> Color {
    use crate::model::Terrain::*;
    match terrain {
        Grass => theme.ink(),
        Forest => theme.warm_brown(),
        Water => theme.dark_brown(),
        Mountain => theme.dark_ink(),
        Road => theme.ink(),
        Settlement => theme.warm_brown(),
        House => theme.archive_red(),
        Wall => theme.dark_brown(),
        Floor => theme.warm_brown(),
        Door => theme.archive_red(),
        Hearth => theme.archive_red(),
        Farmland => theme.ink(),
        Sand => theme.warm_brown(),
        Swamp => theme.dark_brown(),
        Coast => theme.dark_brown(),
        Cave => theme.dark_ink(),
        Tundra => theme.dark_ink(),
        Steppe => theme.warm_brown(),
    }
}

#[cfg(test)]
mod minimap_tests {
    use super::*;
    use crate::model::{Person, Region, Settlement, Terrain, TerrainMap};

    fn test_person(id: &str, people: &str) -> Person {
        Person {
            id: id.into(),
            name: id.into(),
            people: people.into(),
            sex: "F".into(),
            age_band: "adult".into(),
            profession: "farmer".into(),
            social_class: "common".into(),
            craft_affinity: String::new(),
            personality: vec![],
            bias: "neutral".into(),
            needs: crate::model::Needs::default(),
            region: "r".into(),
            settlement: "a".into(),
            has_spouse: false,
            children_count: 0,
            has_debt: false,
            coins: 0,
            schedule: Default::default(),
            illnesses: Vec::new(),
            relations: vec![],
            wants: vec![],
            gift: Default::default(),
            aspiration: None,
            crimes: 0,
            wares: 0,
            age_years: 0,
        }
    }

    fn test_region() -> Region {
        let tiles = vec![
            Terrain::Grass,
            Terrain::Forest,
            Terrain::Settlement,
            Terrain::Water,
        ];
        Region {
            id: "test-region".into(),
            name: "Test Valley".into(),
            region_type: "river_valley".into(),
            region_subtype: String::new(),
            description: "A test region".into(),
            settlements: vec![Settlement {
                id: "test-settlement".into(),
                name: "Testton".into(),
                size: "hamlet".into(),
                region: "test-region".into(),
                population: 5,
                description: String::new(),
                people: vec![],
                services: vec![],
                politics: crate::model::SettlementPolitics::new(),
                faith: Default::default(),
                food_stock: 0.0,
                treasury: 0,
                goods_stock: Default::default(),
                farms: Vec::new(),
                buildings: Vec::new(),
                festival_until_day: 0,
                famine_days: 0,
                plague_days: 0,
                map_x: 0,
                map_y: 0,
                district: 0,
                remembered_deed: None,
            }],
            terrain: TerrainMap {
                width: 2,
                height: 2,
                tiles,
            },
            neighbors: Default::default(),
            structures: Vec::new(),
            weather: crate::model::Weather::Clear,
            game_richness: 1.0,
            is_march: false,
            known_fed: None,
            known_fed_as_of: 0,
        }
    }

    #[test]
    fn region_terrain_extractable() {
        let region = test_region();
        assert_eq!(region.terrain.width, 2);
        assert_eq!(region.terrain.height, 2);
        assert_eq!(region.terrain.tiles.len(), 4);
        assert_eq!(region.terrain.tiles[2], Terrain::Settlement);
    }

    #[test]
    fn settlement_present_in_region() {
        let region = test_region();
        assert_eq!(region.settlements.len(), 1);
        assert_eq!(region.settlements[0].name, "Testton");
    }

    #[test]
    fn terrain_glyphs_distinct() {
        let theme = Theme {
            monochrome: false,
            high_contrast: false,
        };
        let grass = glyph_color(Terrain::Grass, &theme);
        let water = glyph_color(Terrain::Water, &theme);
        let mountain = glyph_color(Terrain::Mountain, &theme);
        let settlement = glyph_color(Terrain::Settlement, &theme);
        assert_ne!(grass, water);
        assert_ne!(water, mountain);
        assert_ne!(mountain, settlement);
    }

    #[test]
    fn dominant_people_color_falls_back_to_red() {
        let theme = Theme {
            monochrome: false,
            high_contrast: false,
        };
        let color = dominant_people_color(&[], &theme);
        assert_eq!(color, theme.archive_red());
    }

    #[test]
    fn dominant_people_color_picks_majority() {
        let theme = Theme {
            monochrome: false,
            high_contrast: false,
        };
        let settlements = vec![Settlement {
            id: "a".into(),
            name: "A".into(),
            size: "hamlet".into(),
            region: "r".into(),
            population: 3,
            description: String::new(),
            people: vec![
                test_person("p1", "Metsik"),
                test_person("p2", "Sepät"),
                test_person("p3", "Metsik"),
            ],
            services: vec![],
            politics: crate::model::SettlementPolitics::new(),
            faith: Default::default(),
            food_stock: 0.0,
            treasury: 0,
            goods_stock: Default::default(),
            farms: Vec::new(),
            buildings: Vec::new(),
            festival_until_day: 0,
            famine_days: 0,
            plague_days: 0,
            map_x: 0,
            map_y: 0,
            district: 0,
            remembered_deed: None,
        }];
        let color = dominant_people_color(&settlements, &theme);
        assert_eq!(color, theme.warm_brown());
    }

    #[test]
    fn dominant_people_color_handles_unknown_people() {
        let theme = Theme {
            monochrome: false,
            high_contrast: false,
        };
        let settlements = vec![Settlement {
            id: "a".into(),
            name: "A".into(),
            size: "hamlet".into(),
            region: "r".into(),
            population: 1,
            description: String::new(),
            people: vec![test_person("p1", "Unknown")],
            services: vec![],
            politics: crate::model::SettlementPolitics::new(),
            faith: Default::default(),
            food_stock: 0.0,
            treasury: 0,
            goods_stock: Default::default(),
            farms: Vec::new(),
            buildings: Vec::new(),
            festival_until_day: 0,
            famine_days: 0,
            plague_days: 0,
            map_x: 0,
            map_y: 0,
            district: 0,
            remembered_deed: None,
        }];
        let color = dominant_people_color(&settlements, &theme);
        assert_eq!(color, theme.archive_red());
    }
}
