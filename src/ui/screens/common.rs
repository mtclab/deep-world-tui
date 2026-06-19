use std::collections::HashMap;

use ratatui::style::{Color, Modifier, Style};

use crate::model::Terrain;
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
        Terrain::Hearth => Color::Rgb(0xe8, 0x7a, 0x2a),
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

/// How a townsperson reads on the street by their trade (#637): a roguelike
/// tells a smith from a baker from a guard at a glance, by glyph and colour —
/// not by a label in a menu. The glyph is the trade's initial-ish mark, the
/// colour its character (iron-red smith, green healer, blue guard, ...). An
/// unknown trade falls back to a plain townsperson dot.
pub(crate) fn role_glyph(profession: &str) -> (char, Color) {
    match profession {
        "smith" => ('S', Color::Rgb(220, 120, 80)), // forge-iron
        "blacksmith" => ('S', Color::Rgb(220, 120, 80)),
        "baker" => ('b', Color::Rgb(210, 180, 120)), // bread-brown
        "farmer" => ('f', Color::Rgb(150, 200, 100)),
        "herder" => ('r', Color::Rgb(150, 200, 100)),
        "fisher" => ('F', Color::Rgb(100, 180, 210)),
        "hunter" => ('H', Color::Rgb(160, 140, 90)),
        "miner" => ('M', Color::Rgb(150, 150, 160)),
        "healer" | "herbalist" => ('h', Color::Rgb(120, 210, 130)), // herb-green
        "trader" => ('$', Color::Rgb(230, 200, 90)),                // coin-gold
        "weaver" => ('w', Color::Rgb(200, 150, 200)),
        "carpenter" => ('c', Color::Rgb(190, 150, 100)),
        "scribe" => ('s', Color::Rgb(180, 180, 220)),
        "priest" => ('p', Color::Rgb(220, 220, 160)),
        "guard" | "soldier" | "warden" => ('G', Color::Rgb(110, 150, 230)), // watch-blue
        "elder" => ('E', Color::Rgb(210, 210, 210)),
        "labourer" => ('l', Color::Rgb(170, 170, 170)),
        _ => ('o', Color::Rgb(190, 180, 140)), // a plain townsperson
    }
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
                    // Drawn by trade, not ethnicity (#637): you read the street —
                    // the smith at the forge, the baker by the ovens, the guard
                    // at the gate — at a glance.
                    let (glyph, color) = role_glyph(&person.profession);
                    npcs.entry((nx, ny)).or_insert(MapNpc { glyph, color });
                }
            }
        }
    }

    // Outlaw bands of the frontier stand on the grid too (#637): a hostile mark
    // in the wild country, where you can see them and choose to walk in. Drawn
    // last so a band on open ground reads over the empty land.
    if let Some(sim) = app.sim.as_ref() {
        for band in &sim.frontier.bands {
            if band.region_idx != region_idx {
                continue;
            }
            if let Some((bx, by)) = crate::sim::frontier::band_field_tile(sim, &band.id, region_idx)
            {
                if bx >= vp.cam_x
                    && by >= vp.cam_y
                    && bx < vp.cam_x + vp.view_w
                    && by < vp.cam_y + vp.view_h
                    && !(bx == vp.px && by == vp.py)
                {
                    // 'r' for raiders, blood-red — a thing to fear or to hunt.
                    npcs.insert(
                        (bx, by),
                        MapNpc {
                            glyph: 'r',
                            color: Color::Rgb(220, 60, 60),
                        },
                    );
                }
            }
        }
    }

    npcs
}

#[cfg(test)]
mod tests {
    use super::role_glyph;

    #[test]
    fn trades_read_apart_on_the_street() {
        // The whole point (#637): a smith, a baker, a guard, a healer must each
        // read distinctly — different glyph AND different colour — so the street
        // is legible at a glance, not by opening a menu.
        let roles = [
            "smith", "baker", "guard", "healer", "trader", "farmer", "scribe",
        ];
        let mut glyphs = std::collections::HashSet::new();
        let mut colors = std::collections::HashSet::new();
        for r in roles {
            let (g, c) = role_glyph(r);
            assert!(glyphs.insert(g), "trade {r} reuses a glyph");
            // Colours may rhyme across kindred trades, but the headline ones differ.
            colors.insert(format!("{c:?}"));
        }
        assert!(
            colors.len() >= 5,
            "the trades should mostly differ in colour too"
        );
        // blacksmith reads the same as smith (same trade, two names).
        assert_eq!(role_glyph("smith"), role_glyph("blacksmith"));
        // An unknown trade falls back to a plain townsperson, not a smith.
        assert_ne!(role_glyph("mystery-trade"), role_glyph("smith"));
    }
}
