use std::collections::HashMap;

use ratatui::style::{Color, Modifier, Style};

use crate::model::{PeopleKind, Terrain};
use crate::ui::app::App;
use crate::ui::theme::Theme;

pub(crate) fn need_bar(val: f64, width: usize) -> String {
    let filled = (val * width as f64).round() as usize;
    let empty = width.saturating_sub(filled);
    format!("{}{}", "█".repeat(filled), "░".repeat(empty))
}

pub(crate) fn stance_label(bias: f64) -> &'static str {
    if bias > 0.05 {
        "++ ally"
    } else if bias > -0.05 {
        "~  neutral"
    } else if bias > -0.15 {
        "-  wary"
    } else {
        "-- hostile"
    }
}

pub(crate) fn stance_color(bias: f64, theme: &Theme) -> Color {
    if bias > 0.05 {
        theme.need_color(1.0)
    } else if bias < -0.05 {
        theme.need_color(0.0)
    } else {
        theme.dark_brown()
    }
}

pub(crate) fn reputation_label(val: f64) -> &'static str {
    if val > 0.6 {
        "++ favored"
    } else if val > 0.3 {
        "+  pleased"
    } else if val > 0.0 {
        "   noticed"
    } else if val == 0.0 {
        "?  unknown"
    } else if val > -0.3 {
        "-  wary"
    } else if val > -0.6 {
        "-- displeased"
    } else {
        "--- angered"
    }
}

pub(crate) fn focus_cursor(is_selected: bool) -> &'static str {
    if is_selected {
        "▸"
    } else {
        " "
    }
}

pub(crate) fn pulse_style(base_color: Color, tick: u64, low: bool, reduced_motion: bool) -> Style {
    if low && !reduced_motion && tick % 4 < 2 {
        Style::default().fg(base_color).add_modifier(Modifier::BOLD)
    } else {
        Style::default().fg(base_color)
    }
}

pub(crate) const STATUS_HEIGHT: u16 = 3;

pub(crate) fn terrain_color(terrain: Terrain) -> Color {
    match terrain {
        Terrain::Grass => Color::Rgb(0x6b, 0x8e, 0x4a),
        Terrain::Forest => Color::Rgb(0x3a, 0x5a, 0x2a),
        Terrain::Water => Color::Rgb(0x4a, 0x7a, 0x9e),
        Terrain::Mountain => Color::Rgb(0x8a, 0x7a, 0x6a),
        Terrain::Road => Color::Rgb(0x9a, 0x8a, 0x6a),
        Terrain::Settlement => Color::Rgb(0x9a, 0x8a, 0x7a),
        Terrain::House => Color::Rgb(0x7a, 0x2e, 0x1d),
        Terrain::Wall => Color::Rgb(0x6a, 0x5a, 0x4a),
        Terrain::Floor => Color::Rgb(0x5a, 0x4a, 0x3a),
        Terrain::Door => Color::Rgb(0xc2, 0x9a, 0x4a),
        Terrain::Farmland => Color::Rgb(0x8a, 0x9a, 0x4a),
        Terrain::Sand => Color::Rgb(0xc2, 0x9a, 0x6b),
        Terrain::Swamp => Color::Rgb(0x5a, 0x6a, 0x3a),
        Terrain::Coast => Color::Rgb(0x6a, 0x9a, 0xba),
        Terrain::Cave => Color::Rgb(0x4a, 0x4a, 0x5a),
        Terrain::Tundra => Color::Rgb(0xaa, 0xc0, 0xcc),
        Terrain::DeepDesert => Color::Rgb(0xd2, 0xba, 0x8a),
    }
}

pub(crate) fn dim_color(c: Color) -> Color {
    match c {
        Color::Rgb(r, g, b) => Color::Rgb(r / 2, g / 2, b / 2),
        other => other,
    }
}

pub(crate) fn terrain_color_at(terrain: Terrain, dark: bool) -> Color {
    let c = terrain_color(terrain);
    if dark {
        dim_color(c)
    } else {
        c
    }
}

pub(crate) struct MapViewport {
    pub px: usize,
    pub py: usize,
    pub view_w: usize,
    pub view_h: usize,
    pub cam_x: usize,
    pub cam_y: usize,
}

pub(crate) struct MapNpc {
    pub glyph: char,
    pub color: Color,
}

pub(crate) fn build_npc_map(
    app: &App,
    region_idx: usize,
    vp: &MapViewport,
) -> HashMap<(usize, usize), MapNpc> {
    let mut npcs: HashMap<(usize, usize), MapNpc> = HashMap::new();

    let regions = match app.sim.as_ref() {
        Some(sim) => &sim.world.regions,
        None => return npcs,
    };
    let region = match regions.get(region_idx) {
        Some(r) => r,
        None => return npcs,
    };

    if let Some(ref ps) = app.player_start {
        for companion in &ps.companions {
            let dirs: [(i32, i32); 4] = [(0, -1), (-1, 0), (1, 0), (0, 1)];
            let cidx = companion.animal as u32;
            let (dx, dy) = dirs[(cidx as usize) % 4];
            let cx = (vp.px as i32 + dx) as usize;
            let cy = (vp.py as i32 + dy) as usize;
            if cx >= vp.cam_x
                && cy >= vp.cam_y
                && cx < vp.cam_x + vp.view_w
                && cy < vp.cam_y + vp.view_h
            {
                npcs.entry((cx, cy)).or_insert(MapNpc {
                    glyph: companion.animal.glyph(),
                    color: Color::Green,
                });
            }
        }
    }

    for settlement in region.settlements.iter() {
        for (pi, nx, ny) in
            crate::gen::town::npc_street_positions(settlement, app.clock.day, app.clock.hour)
        {
            if nx == vp.px && ny == vp.py {
                continue;
            }
            if nx >= vp.cam_x
                && ny >= vp.cam_y
                && nx < vp.cam_x + vp.view_w
                && ny < vp.cam_y + vp.view_h
            {
                if let Some(person) = settlement.people.get(pi) {
                    let pk = PeopleKind::from_name(&person.people);
                    npcs.entry((nx, ny)).or_insert(MapNpc {
                        glyph: pk.glyph(),
                        color: Color::Yellow,
                    });
                }
            }
        }
    }

    npcs
}
