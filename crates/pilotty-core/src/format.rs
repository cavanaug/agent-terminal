//! Format module for styled snapshot output.
//!
//! Provides compact serialization for style and color data, and functions
//! to build position-based `style_map` and `color_map` from segmented grid data.
//!
//! # Design
//!
//! `CellStyle` has verbose `derive(Serialize)` used for Element protocol serialization.
//! This module provides separate compact serialization with short keys (b/i/d/u/v/fg/bg)
//! for token-efficient styled snapshot output.

use serde::{Deserialize, Serialize};
use serde_json::Value;
use unicode_width::UnicodeWidthChar;

use crate::elements::grid::ScreenGrid;
use crate::elements::segment::Cluster;
use crate::elements::style::{CellStyle, Color};

/// Render mode controlling which style data is included in snapshots.
///
/// Each tier adds exactly one field to the snapshot:
/// - Basic: text only (no maps)
/// - Styled: text + style_map (text attributes only)
/// - Color: text + style_map + color_map (full style + color data)
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum RenderMode {
    /// No style data — text only.
    #[default]
    Basic,
    /// Text attributes (bold, italic, dim, underline, inverse) via style_map.
    Styled,
    /// Full style + color data via style_map + color_map.
    Color,
}

impl RenderMode {
    /// Check if this mode allows the given tier.
    ///
    /// Styled is allowed if mode >= Styled; Color is allowed if mode >= Color.
    #[must_use]
    pub fn allows_style(&self) -> bool {
        matches!(self, RenderMode::Styled | RenderMode::Color)
    }

    /// Check if this mode allows color data.
    #[must_use]
    pub fn allows_color(&self) -> bool {
        matches!(self, RenderMode::Color)
    }
}

/// A position-based style entry for the style_map.
///
/// Each entry describes a range of characters sharing the same text attributes.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct StyleMapEntry {
    /// Row index (0-based).
    pub r: u16,
    /// Column index (0-based).
    pub c: u16,
    /// Length in characters.
    pub l: u16,
    /// Compact style attributes.
    pub s: Value,
}

/// A position-based color entry for the color_map.
///
/// Each entry describes a range of characters sharing the same color attributes.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ColorMapEntry {
    /// Row index (0-based).
    pub r: u16,
    /// Column index (0-based).
    pub c: u16,
    /// Length in characters.
    pub l: u16,
    /// Compact color attributes.
    pub s: Value,
}

/// Produce a compact style object with short keys for text attributes.
///
/// Only includes non-default attributes to minimize token usage.
/// Keys: b=bold, i=italic, d=dim, u=underline, v=inverse.
#[must_use]
pub fn compact_style(style: &CellStyle) -> Value {
    let mut map = serde_json::Map::new();
    if style.bold {
        map.insert("b".into(), Value::Bool(true));
    }
    if style.italic {
        map.insert("i".into(), Value::Bool(true));
    }
    if style.dim {
        map.insert("d".into(), Value::Bool(true));
    }
    if style.underline {
        map.insert("u".into(), Value::Bool(true));
    }
    if style.inverse {
        map.insert("v".into(), Value::Bool(true));
    }
    Value::Object(map)
}

/// Produce a compact color object with short keys for color attributes.
///
/// Only includes non-default colors to minimize token usage.
/// Keys: fg=foreground, bg=background.
#[must_use]
pub fn compact_color(style: &CellStyle) -> Value {
    let mut map = serde_json::Map::new();
    if style.fg_color != Color::Default {
        map.insert("fg".into(), serialize_color(&style.fg_color));
    }
    if style.bg_color != Color::Default {
        map.insert("bg".into(), serialize_color(&style.bg_color));
    }
    Value::Object(map)
}

fn serialize_color(color: &Color) -> Value {
    match color {
        Color::Default => Value::Null,
        Color::Indexed { index } => Value::Number((*index).into()),
        Color::Rgb { r, g, b } => Value::String(format!("#{r:02x}{g:02x}{b:02x}")),
    }
}

/// Check if a CellStyle has any non-default text attributes.
fn has_style_attrs(style: &CellStyle) -> bool {
    style.bold || style.italic || style.dim || style.underline || style.inverse
}

/// Check if a CellStyle has any non-default color attributes.
fn has_color_attrs(style: &CellStyle) -> bool {
    style.fg_color != Color::Default || style.bg_color != Color::Default
}

/// Segment a grid into clusters (all clusters, including whitespace-only).
///
/// This is a public wrapper around the segmentation pipeline for use by
/// the format module. Unlike `segment()`, this does NOT filter whitespace
/// clusters — we need them all for accurate style/color mapping.
#[must_use]
pub fn segment_grid<G: ScreenGrid>(grid: &G) -> Vec<Cluster> {
    // Re-implement full-grid segmentation here since the inner function
    // in elements::segment is private. We use the ScreenGrid trait directly.
    let mut clusters = Vec::new();
    for row in 0..grid.rows() {
        let mut current_text = String::new();
        let mut current_style: Option<CellStyle> = None;
        let mut start_col: u16 = 0;

        for col in 0..grid.cols() {
            let Some(cell) = grid.cell(row, col) else {
                continue;
            };
            match current_style {
                Some(ref style) if *style == cell.style => {
                    current_text.push(cell.ch);
                }
                _ => {
                    if let Some(style) = current_style.take() {
                        if !current_text.is_empty() {
                            clusters.push(Cluster::new(
                                row,
                                start_col,
                                std::mem::take(&mut current_text),
                                style,
                            ));
                        }
                    }
                    start_col = col;
                    current_style = Some(cell.style);
                    current_text.push(cell.ch);
                }
            }
        }
        if let Some(style) = current_style {
            if !current_text.is_empty() {
                clusters.push(Cluster::new(row, start_col, current_text, style));
            }
        }
    }
    clusters
}

/// Build a style_map from a grid.
///
/// Returns a list of `StyleMapEntry` for all clusters with non-default text
/// attributes (bold, italic, dim, underline, inverse).
#[must_use]
pub fn build_style_map<G: ScreenGrid>(grid: &G) -> Vec<StyleMapEntry> {
    let clusters = segment_grid(grid);
    clusters
        .iter()
        .filter(|c| has_style_attrs(&c.style))
        .map(|c| StyleMapEntry {
            r: c.row,
            c: c.col,
            l: c.width,
            s: compact_style(&c.style),
        })
        .collect()
}

/// Build a color_map from a grid.
///
/// Returns a list of `ColorMapEntry` for all clusters with non-default color
/// attributes (fg or bg color).
#[must_use]
pub fn build_color_map<G: ScreenGrid>(grid: &G) -> Vec<ColorMapEntry> {
    let clusters = segment_grid(grid);
    clusters
        .iter()
        .filter(|c| has_color_attrs(&c.style))
        .map(|c| ColorMapEntry {
            r: c.row,
            c: c.col,
            l: c.width,
            s: compact_color(&c.style),
        })
        .collect()
}

/// Convert a `CellStyle` to an ANSI SGR escape sequence, respecting `RenderMode` tier filtering.
///
/// - `Basic`: returns empty string (no SGR output)
/// - `Styled`: emits text attributes only (bold=1, dim=2, italic=3, underline=4, inverse=7)
/// - `Color`: emits text attributes + foreground/background color codes
///
/// Multiple attributes are combined into a single `\x1b[a;b;cm` sequence.
/// Returns empty string when the style is fully default or when the render mode filters
/// everything out.
#[must_use]
pub fn style_to_sgr(style: &CellStyle, render_mode: RenderMode) -> String {
    let mut codes: Vec<String> = Vec::new();

    if render_mode.allows_style() {
        if style.bold {
            codes.push("1".into());
        }
        if style.dim {
            codes.push("2".into());
        }
        if style.italic {
            codes.push("3".into());
        }
        if style.underline {
            codes.push("4".into());
        }
        if style.inverse {
            codes.push("7".into());
        }
    }

    if render_mode.allows_color() {
        match style.fg_color {
            Color::Default => {}
            Color::Indexed { index } if index < 8 => {
                codes.push(format!("{}", 30 + index));
            }
            Color::Indexed { index } => {
                codes.push(format!("38;5;{index}"));
            }
            Color::Rgb { r, g, b } => {
                codes.push(format!("38;2;{r};{g};{b}"));
            }
        }
        match style.bg_color {
            Color::Default => {}
            Color::Indexed { index } if index < 8 => {
                codes.push(format!("{}", 40 + index));
            }
            Color::Indexed { index } => {
                codes.push(format!("48;5;{index}"));
            }
            Color::Rgb { r, g, b } => {
                codes.push(format!("48;2;{r};{g};{b}"));
            }
        }
    }

    if codes.is_empty() {
        String::new()
    } else {
        format!("\x1b[{}m", codes.join(";"))
    }
}

/// Render clusters as ANSI SGR-escaped text lines.
///
/// Produces a plain header line followed by rows of styled terminal output.
/// Each non-default-styled cluster is wrapped with reset + SGR sequences.
/// Lines containing any SGR are reset at the end. A cursor marker `[X]` is
/// inserted at the cursor position.
///
/// # Arguments
///
/// * `clusters` — segmented grid clusters (from `segment_grid`)
/// * `cursor_row` — cursor row (0-based)
/// * `cursor_col` — cursor column (0-based)
/// * `rows` — grid height
/// * `cols` — grid width
/// * `render_mode` — tier controlling which style data is emitted
#[must_use]
pub fn render_ansi_lines(
    clusters: &[Cluster],
    cursor_row: u16,
    cursor_col: u16,
    rows: u16,
    cols: u16,
    render_mode: RenderMode,
) -> String {
    let mut output = String::new();

    // Header line — always plain, no SGR
    output.push_str(&format!(
        "--- Terminal {cols}x{rows} | Cursor: ({cursor_row}, {cursor_col}) ---\n"
    ));

    for row in 0..rows {
        let row_clusters: Vec<&Cluster> = clusters.iter().filter(|c| c.row == row).collect();
        let mut in_sgr = false;

        for cluster in &row_clusters {
            // Skip SGR for trailing whitespace clusters with default style
            let is_default_whitespace =
                cluster.is_whitespace_only() && cluster.style == CellStyle::default();

            let sgr = if is_default_whitespace {
                String::new()
            } else {
                style_to_sgr(&cluster.style, render_mode)
            };

            if !sgr.is_empty() {
                // Reset before applying new style (reset-then-set strategy)
                output.push_str("\x1b[0m");
                output.push_str(&sgr);
                in_sgr = true;
            } else if in_sgr {
                // Returning to default style — reset active SGR
                output.push_str("\x1b[0m");
                in_sgr = false;
            }

            // Emit cluster text, inserting cursor marker if needed
            emit_cluster_text(&mut output, cluster, cursor_row, cursor_col);
        }

        if in_sgr {
            output.push_str("\x1b[0m");
        }
        output.push('\n');
    }

    output
}

/// Emit cluster text into the output buffer, inserting a `[X]` cursor marker
/// at the cursor column if it falls within this cluster.
fn emit_cluster_text(output: &mut String, cluster: &Cluster, cursor_row: u16, cursor_col: u16) {
    if cluster.row != cursor_row
        || cursor_col < cluster.col
        || cursor_col >= cluster.col + cluster.width
    {
        output.push_str(&cluster.text);
        return;
    }

    // Cursor is within this cluster — find the character at cursor_col
    let mut col_pos = cluster.col;
    for (i, ch) in cluster.text.chars().enumerate() {
        if col_pos == cursor_col {
            // Insert cursor marker around this character
            // Write chars before
            let prefix: String = cluster.text.chars().take(i).collect();
            output.push_str(&prefix);
            output.push('[');
            output.push(ch);
            output.push(']');
            // Write chars after
            let suffix: String = cluster.text.chars().skip(i + 1).collect();
            output.push_str(&suffix);
            return;
        }
        col_pos += UnicodeWidthChar::width(ch).unwrap_or(1) as u16;
    }

    // Cursor column didn't match any character start — just emit text
    output.push_str(&cluster.text);
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::elements::grid::test_support::SimpleGrid;

    // ========================================================================
    // RenderMode tests
    // ========================================================================

    #[test]
    fn render_mode_default_is_basic() {
        assert_eq!(RenderMode::default(), RenderMode::Basic);
    }

    #[test]
    fn render_mode_allows_style() {
        assert!(!RenderMode::Basic.allows_style());
        assert!(RenderMode::Styled.allows_style());
        assert!(RenderMode::Color.allows_style());
    }

    #[test]
    fn render_mode_allows_color() {
        assert!(!RenderMode::Basic.allows_color());
        assert!(!RenderMode::Styled.allows_color());
        assert!(RenderMode::Color.allows_color());
    }

    #[test]
    fn render_mode_serde_roundtrip() {
        let json = serde_json::to_string(&RenderMode::Styled).unwrap();
        assert_eq!(json, "\"styled\"");
        let mode: RenderMode = serde_json::from_str("\"color\"").unwrap();
        assert_eq!(mode, RenderMode::Color);
        let mode: RenderMode = serde_json::from_str("\"basic\"").unwrap();
        assert_eq!(mode, RenderMode::Basic);
    }

    // ========================================================================
    // compact_style tests
    // ========================================================================

    #[test]
    fn compact_style_default_is_empty() {
        let style = CellStyle::default();
        let compact = compact_style(&style);
        assert_eq!(compact, Value::Object(serde_json::Map::new()));
    }

    #[test]
    fn compact_style_bold_only() {
        let style = CellStyle::new().with_bold(true);
        let compact = compact_style(&style);
        assert_eq!(compact["b"], Value::Bool(true));
        assert!(compact.get("i").is_none());
    }

    #[test]
    fn compact_style_all_attrs() {
        let style = CellStyle::new()
            .with_bold(true)
            .with_italic(true)
            .with_dim(true)
            .with_underline(true)
            .with_inverse(true);
        let compact = compact_style(&style);
        assert_eq!(compact["b"], Value::Bool(true));
        assert_eq!(compact["i"], Value::Bool(true));
        assert_eq!(compact["d"], Value::Bool(true));
        assert_eq!(compact["u"], Value::Bool(true));
        assert_eq!(compact["v"], Value::Bool(true));
    }

    // ========================================================================
    // compact_color tests
    // ========================================================================

    #[test]
    fn compact_color_default_is_empty() {
        let style = CellStyle::default();
        let compact = compact_color(&style);
        assert_eq!(compact, Value::Object(serde_json::Map::new()));
    }

    #[test]
    fn compact_color_indexed_fg() {
        let style = CellStyle::new().with_fg(Color::indexed(1));
        let compact = compact_color(&style);
        assert_eq!(compact["fg"], Value::Number(1.into()));
    }

    #[test]
    fn compact_color_rgb_bg() {
        let style = CellStyle::new().with_bg(Color::rgb(255, 128, 0));
        let compact = compact_color(&style);
        assert_eq!(compact["bg"], Value::String("#ff8000".into()));
    }

    #[test]
    fn compact_color_both() {
        let style = CellStyle::new()
            .with_fg(Color::indexed(1))
            .with_bg(Color::rgb(0, 0, 0));
        let compact = compact_color(&style);
        assert_eq!(compact["fg"], Value::Number(1.into()));
        assert_eq!(compact["bg"], Value::String("#000000".into()));
    }

    // ========================================================================
    // build_style_map tests
    // ========================================================================

    #[test]
    fn build_style_map_empty_grid_no_entries() {
        let grid = SimpleGrid::from_text(&["hello"], 5);
        let map = build_style_map(&grid);
        assert!(map.is_empty(), "Default style should produce no entries");
    }

    #[test]
    fn build_style_map_bold_range() {
        let mut grid = SimpleGrid::from_text(&["Hello World"], 11);
        let bold = CellStyle::new().with_bold(true);
        grid.style_range(0, 0, 5, bold);

        let map = build_style_map(&grid);
        assert_eq!(map.len(), 1);
        assert_eq!(map[0].r, 0);
        assert_eq!(map[0].c, 0);
        assert_eq!(map[0].l, 5);
        assert_eq!(map[0].s["b"], Value::Bool(true));
    }

    #[test]
    fn build_style_map_multiple_ranges() {
        let mut grid = SimpleGrid::from_text(&["AABBBCC"], 7);
        grid.style_range(0, 0, 2, CellStyle::new().with_bold(true));
        grid.style_range(0, 2, 5, CellStyle::new().with_italic(true));
        grid.style_range(0, 5, 7, CellStyle::new().with_underline(true));

        let map = build_style_map(&grid);
        assert_eq!(map.len(), 3);
        assert_eq!(map[0].s["b"], Value::Bool(true));
        assert_eq!(map[1].s["i"], Value::Bool(true));
        assert_eq!(map[2].s["u"], Value::Bool(true));
    }

    #[test]
    fn build_style_map_multirow() {
        let mut grid = SimpleGrid::from_text(&["Line1", "Line2"], 5);
        grid.style_range(0, 0, 5, CellStyle::new().with_bold(true));
        grid.style_range(1, 0, 5, CellStyle::new().with_dim(true));

        let map = build_style_map(&grid);
        assert_eq!(map.len(), 2);
        assert_eq!(map[0].r, 0);
        assert_eq!(map[1].r, 1);
    }

    // ========================================================================
    // build_color_map tests
    // ========================================================================

    #[test]
    fn build_color_map_empty_grid_no_entries() {
        let grid = SimpleGrid::from_text(&["hello"], 5);
        let map = build_color_map(&grid);
        assert!(map.is_empty(), "Default color should produce no entries");
    }

    #[test]
    fn build_color_map_fg_range() {
        let mut grid = SimpleGrid::from_text(&["RedText"], 7);
        grid.style_range(0, 0, 3, CellStyle::new().with_fg(Color::indexed(1)));

        let map = build_color_map(&grid);
        assert_eq!(map.len(), 1);
        assert_eq!(map[0].r, 0);
        assert_eq!(map[0].c, 0);
        assert_eq!(map[0].l, 3);
        assert_eq!(map[0].s["fg"], Value::Number(1.into()));
    }

    #[test]
    fn build_color_map_bg_range() {
        let mut grid = SimpleGrid::from_text(&["Highlighted"], 11);
        grid.style_range(0, 0, 11, CellStyle::new().with_bg(Color::rgb(255, 255, 0)));

        let map = build_color_map(&grid);
        assert_eq!(map.len(), 1);
        assert_eq!(map[0].s["bg"], Value::String("#ffff00".into()));
    }

    #[test]
    fn build_color_map_ignores_style_only() {
        let mut grid = SimpleGrid::from_text(&["BoldOnly"], 8);
        grid.style_range(0, 0, 8, CellStyle::new().with_bold(true));

        let map = build_color_map(&grid);
        assert!(map.is_empty(), "Bold-only should not appear in color_map");
    }

    // ========================================================================
    // style_map and color_map independence tests
    // ========================================================================

    #[test]
    fn style_map_ignores_color_only() {
        let mut grid = SimpleGrid::from_text(&["Colored"], 7);
        grid.style_range(0, 0, 7, CellStyle::new().with_fg(Color::indexed(1)));

        let style_map = build_style_map(&grid);
        assert!(
            style_map.is_empty(),
            "Color-only should not appear in style_map"
        );
    }

    #[test]
    fn combined_style_and_color() {
        let mut grid = SimpleGrid::from_text(&["BoldRed"], 7);
        let style = CellStyle::new().with_bold(true).with_fg(Color::indexed(1));
        grid.style_range(0, 0, 7, style);

        let style_map = build_style_map(&grid);
        let color_map = build_color_map(&grid);

        assert_eq!(style_map.len(), 1);
        assert_eq!(color_map.len(), 1);
        assert_eq!(style_map[0].s["b"], Value::Bool(true));
        assert_eq!(color_map[0].s["fg"], Value::Number(1.into()));
    }

    #[test]
    fn segment_grid_includes_whitespace() {
        let grid = SimpleGrid::from_text(&["A B"], 3);
        let clusters = segment_grid(&grid);
        // All default style, so should be one cluster "A B"
        assert_eq!(clusters.len(), 1);
        assert_eq!(clusters[0].text, "A B");
    }

    // ========================================================================
    // style_to_sgr tests
    // ========================================================================

    #[test]
    fn sgr_bold_only() {
        let style = CellStyle::new().with_bold(true);
        assert_eq!(style_to_sgr(&style, RenderMode::Color), "\x1b[1m");
    }

    #[test]
    fn sgr_italic_only() {
        let style = CellStyle::new().with_italic(true);
        assert_eq!(style_to_sgr(&style, RenderMode::Color), "\x1b[3m");
    }

    #[test]
    fn sgr_dim_only() {
        let style = CellStyle::new().with_dim(true);
        assert_eq!(style_to_sgr(&style, RenderMode::Color), "\x1b[2m");
    }

    #[test]
    fn sgr_underline_only() {
        let style = CellStyle::new().with_underline(true);
        assert_eq!(style_to_sgr(&style, RenderMode::Color), "\x1b[4m");
    }

    #[test]
    fn sgr_inverse_only() {
        let style = CellStyle::new().with_inverse(true);
        assert_eq!(style_to_sgr(&style, RenderMode::Color), "\x1b[7m");
    }

    #[test]
    fn sgr_combined_attrs() {
        let style = CellStyle::new().with_bold(true).with_underline(true);
        assert_eq!(style_to_sgr(&style, RenderMode::Color), "\x1b[1;4m");
    }

    #[test]
    fn sgr_fg_indexed_standard() {
        let style = CellStyle::new().with_fg(Color::indexed(1));
        assert_eq!(style_to_sgr(&style, RenderMode::Color), "\x1b[31m");
    }

    #[test]
    fn sgr_fg_indexed_extended() {
        let style = CellStyle::new().with_fg(Color::indexed(100));
        assert_eq!(style_to_sgr(&style, RenderMode::Color), "\x1b[38;5;100m");
    }

    #[test]
    fn sgr_fg_rgb() {
        let style = CellStyle::new().with_fg(Color::rgb(255, 0, 128));
        assert_eq!(
            style_to_sgr(&style, RenderMode::Color),
            "\x1b[38;2;255;0;128m"
        );
    }

    #[test]
    fn sgr_bg_indexed_standard() {
        let style = CellStyle::new().with_bg(Color::indexed(2));
        assert_eq!(style_to_sgr(&style, RenderMode::Color), "\x1b[42m");
    }

    #[test]
    fn sgr_bg_rgb() {
        let style = CellStyle::new().with_bg(Color::rgb(10, 20, 30));
        assert_eq!(
            style_to_sgr(&style, RenderMode::Color),
            "\x1b[48;2;10;20;30m"
        );
    }

    #[test]
    fn sgr_styled_tier_no_colors() {
        let style = CellStyle::new().with_bold(true).with_fg(Color::indexed(1));
        // Styled mode: only text attrs, no color
        assert_eq!(style_to_sgr(&style, RenderMode::Styled), "\x1b[1m");
    }

    #[test]
    fn sgr_basic_tier_nothing() {
        let style = CellStyle::new()
            .with_bold(true)
            .with_italic(true)
            .with_fg(Color::indexed(1));
        assert_eq!(style_to_sgr(&style, RenderMode::Basic), "");
    }

    #[test]
    fn sgr_default_style_empty() {
        let style = CellStyle::default();
        assert_eq!(style_to_sgr(&style, RenderMode::Color), "");
        assert_eq!(style_to_sgr(&style, RenderMode::Styled), "");
        assert_eq!(style_to_sgr(&style, RenderMode::Basic), "");
    }

    // ========================================================================
    // render_ansi_lines tests
    // ========================================================================

    #[test]
    fn render_plain_text_no_sgr() {
        let grid = SimpleGrid::from_text(&["hello"], 5);
        let clusters = segment_grid(&grid);
        // Cursor off-screen so no marker
        let out = render_ansi_lines(&clusters, 99, 99, 1, 5, RenderMode::Color);
        assert!(
            !out.lines().skip(1).any(|l| l.contains('\x1b')),
            "Default-style text should have no escape sequences"
        );
        assert!(out.contains("hello"));
    }

    #[test]
    fn render_bold_cluster() {
        let mut grid = SimpleGrid::from_text(&["bold"], 4);
        grid.style_range(0, 0, 4, CellStyle::new().with_bold(true));
        let clusters = segment_grid(&grid);
        let out = render_ansi_lines(&clusters, 99, 99, 1, 4, RenderMode::Color);
        let line = out.lines().nth(1).unwrap();
        assert!(line.contains("\x1b[1m"), "Should contain bold SGR");
        assert!(line.ends_with("\x1b[0m"), "Should end with reset");
        assert!(line.contains("bold"));
    }

    #[test]
    fn render_cursor_marker_styled() {
        let mut grid = SimpleGrid::from_text(&["ABCD"], 4);
        grid.style_range(0, 0, 4, CellStyle::new().with_bold(true));
        let clusters = segment_grid(&grid);
        // Cursor at row 0, col 2 (the 'C')
        let out = render_ansi_lines(&clusters, 0, 2, 1, 4, RenderMode::Color);
        let line = out.lines().nth(1).unwrap();
        assert!(
            line.contains("[C]"),
            "Cursor marker should wrap 'C': {line}"
        );
        assert!(line.contains("\x1b[1m"), "SGR should still be present");
    }

    #[test]
    fn render_multirow() {
        let grid = SimpleGrid::from_text(&["row0", "row1"], 4);
        let clusters = segment_grid(&grid);
        let out = render_ansi_lines(&clusters, 99, 99, 2, 4, RenderMode::Color);
        let lines: Vec<&str> = out.lines().collect();
        assert_eq!(lines.len(), 3); // header + 2 rows
        assert!(lines[1].contains("row0"));
        assert!(lines[2].contains("row1"));
    }

    #[test]
    fn render_empty_grid() {
        let out = render_ansi_lines(&[], 0, 0, 0, 10, RenderMode::Color);
        // Only the header line
        let lines: Vec<&str> = out.lines().collect();
        assert_eq!(lines.len(), 1);
        assert!(lines[0].starts_with("--- Terminal"));
    }

    #[test]
    fn render_header_no_sgr() {
        let mut grid = SimpleGrid::from_text(&["styled"], 6);
        grid.style_range(0, 0, 6, CellStyle::new().with_bold(true));
        let clusters = segment_grid(&grid);
        let out = render_ansi_lines(&clusters, 0, 0, 1, 6, RenderMode::Color);
        let header = out.lines().next().unwrap();
        assert!(
            !header.contains('\x1b'),
            "Header must not contain escape sequences: {header}"
        );
    }

    #[test]
    fn render_line_ends_with_reset() {
        let mut grid = SimpleGrid::from_text(&["text"], 4);
        grid.style_range(0, 0, 4, CellStyle::new().with_italic(true));
        let clusters = segment_grid(&grid);
        let out = render_ansi_lines(&clusters, 99, 99, 1, 4, RenderMode::Color);
        let line = out.lines().nth(1).unwrap();
        assert!(
            line.ends_with("\x1b[0m"),
            "Line with SGR must end with reset: {line}"
        );
    }

    #[test]
    fn render_cursor_marker_default_style() {
        let grid = SimpleGrid::from_text(&["ABCD"], 4);
        let clusters = segment_grid(&grid);
        let out = render_ansi_lines(&clusters, 0, 1, 1, 4, RenderMode::Color);
        let line = out.lines().nth(1).unwrap();
        assert!(
            line.contains("A[B]CD"),
            "Cursor at col 1 should wrap 'B': {line}"
        );
    }

    #[test]
    fn render_default_whitespace_no_sgr() {
        // Grid: "Hi   " where last 3 chars are default-style spaces
        let grid = SimpleGrid::from_text(&["Hi   "], 5);
        let clusters = segment_grid(&grid);
        let out = render_ansi_lines(&clusters, 99, 99, 1, 5, RenderMode::Color);
        let line = out.lines().nth(1).unwrap();
        assert!(
            !line.contains('\x1b'),
            "Default whitespace should not emit SGR"
        );
    }

    #[test]
    fn render_render_mode_styled_filters_colors() {
        let mut grid = SimpleGrid::from_text(&["test"], 4);
        grid.style_range(
            0,
            0,
            4,
            CellStyle::new().with_bold(true).with_fg(Color::indexed(1)),
        );
        let clusters = segment_grid(&grid);
        let out = render_ansi_lines(&clusters, 99, 99, 1, 4, RenderMode::Styled);
        let line = out.lines().nth(1).unwrap();
        assert!(line.contains("\x1b[1m"), "Bold should be present");
        assert!(
            !line.contains("31"),
            "Fg color should be filtered in Styled mode"
        );
    }

    // ========================================================================
    // Edge-case tests (S02)
    // ========================================================================

    #[test]
    fn render_mixed_styled_unstyled_lines() {
        let mut grid = SimpleGrid::from_text(&["styled", "plain."], 6);
        grid.style_range(0, 0, 6, CellStyle::new().with_bold(true));
        let clusters = segment_grid(&grid);
        let out = render_ansi_lines(&clusters, 99, 99, 2, 6, RenderMode::Color);
        let lines: Vec<&str> = out.lines().collect();
        // Row 0 (lines[1]) is styled — must contain SGR
        assert!(
            lines[1].contains('\x1b'),
            "Styled row should have SGR: {}",
            lines[1]
        );
        // Row 1 (lines[2]) is unstyled — must NOT contain SGR
        assert!(
            !lines[2].contains('\x1b'),
            "Unstyled row should have no SGR: {}",
            lines[2]
        );
    }

    #[test]
    fn render_cursor_at_end_of_line() {
        let grid = SimpleGrid::from_text(&["ABCD"], 4);
        let clusters = segment_grid(&grid);
        let out = render_ansi_lines(&clusters, 0, 3, 1, 4, RenderMode::Color);
        let line = out.lines().nth(1).unwrap();
        assert!(
            line.contains("[D]"),
            "Cursor at last col should wrap 'D': {line}"
        );
    }

    #[test]
    fn render_all_default_styles() {
        let grid = SimpleGrid::from_text(&["aaa", "bbb"], 3);
        let clusters = segment_grid(&grid);
        let out = render_ansi_lines(&clusters, 99, 99, 2, 3, RenderMode::Color);
        for (i, line) in out.lines().skip(1).enumerate() {
            assert!(
                !line.contains('\x1b'),
                "Row {i} with default style should have no SGR: {line}"
            );
        }
    }

    #[test]
    fn render_trailing_whitespace_styled() {
        let mut grid = SimpleGrid::from_text(&["Hi   "], 5);
        // Style the ENTIRE row including trailing spaces
        grid.style_range(0, 0, 5, CellStyle::new().with_bold(true));
        let clusters = segment_grid(&grid);
        let out = render_ansi_lines(&clusters, 99, 99, 1, 5, RenderMode::Color);
        let line = out.lines().nth(1).unwrap();
        assert!(
            line.contains("\x1b[1m"),
            "Styled trailing whitespace should emit SGR: {line}"
        );
    }

    #[test]
    fn render_style_transition_mid_line() {
        let mut grid = SimpleGrid::from_text(&["AABB"], 4);
        grid.style_range(0, 0, 2, CellStyle::new().with_bold(true));
        grid.style_range(0, 2, 4, CellStyle::new().with_italic(true));
        let clusters = segment_grid(&grid);
        let out = render_ansi_lines(&clusters, 99, 99, 1, 4, RenderMode::Color);
        let line = out.lines().nth(1).unwrap();
        assert!(
            line.contains("\x1b[0m"),
            "Reset expected between style transitions: {line}"
        );
        assert!(line.contains("\x1b[1m"), "Bold SGR expected: {line}");
        assert!(line.contains("\x1b[3m"), "Italic SGR expected: {line}");
    }

    #[test]
    fn render_basic_strips_all_sgr() {
        let mut grid = SimpleGrid::from_text(&["color"], 5);
        grid.style_range(
            0,
            0,
            5,
            CellStyle::new().with_bold(true).with_fg(Color::indexed(1)),
        );
        let clusters = segment_grid(&grid);
        let out = render_ansi_lines(&clusters, 99, 99, 1, 5, RenderMode::Basic);
        for line in out.lines() {
            assert!(
                !line.contains('\x1b'),
                "Basic mode should strip all SGR: {line}"
            );
        }
    }

    #[test]
    fn render_cursor_on_styled_whitespace() {
        let mut grid = SimpleGrid::from_text(&["A B "], 4);
        grid.style_range(0, 0, 4, CellStyle::new().with_italic(true));
        let clusters = segment_grid(&grid);
        // Cursor at col 1 — the space between 'A' and 'B'
        let out = render_ansi_lines(&clusters, 0, 1, 1, 4, RenderMode::Color);
        let line = out.lines().nth(1).unwrap();
        assert!(
            line.contains("[ ]"),
            "Cursor on space should show [ ]: {line}"
        );
        assert!(
            line.contains("\x1b[3m"),
            "Italic SGR should be present: {line}"
        );
    }
}
