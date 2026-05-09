//! Format module for styled snapshot output.
//!
//! Provides `RenderFeatures` for controlling what data is included in snapshots,
//! and functions to build `rows` (json format) or legacy `style_map`/`color_map`
//! from segmented grid data.
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
use crate::snapshot::{RowEntry, SpanEntry, SpanStyle};

/// Feature flags controlling which style data is included in snapshots.
///
/// Accepted by `--render` as a comma-separated list: `text`, `style`, `color`.
/// Default is all three enabled.
///
/// | Flags | JSON `rows` output | ANSI output |
/// |---|---|---|
/// | `text` only | `rows[].t`, no spans | plain text, no escape codes |
/// | `text,style` | `rows[].t` + spans with bold/italic/etc | text + text-attr SGR |
/// | `text,color` | `rows[].t` + spans with fg/bg | text + color SGR |
/// | `text,style,color` | `rows[].t` + spans with all attrs | full ANSI output |
///
/// Token budget: use `--render text` for pure text extraction, omitting all
/// style/color data to minimize tokens for LLM agents.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub struct RenderFeatures {
    /// Include plain text content. Always true in practice; present for completeness.
    pub text: bool,
    /// Include text attributes (bold, italic, dim, underline, inverse).
    pub style: bool,
    /// Include color data (foreground and background colors).
    pub color: bool,
}

impl Default for RenderFeatures {
    /// Default: all features enabled (text + style + color).
    fn default() -> Self {
        Self { text: true, style: true, color: true }
    }
}

impl RenderFeatures {
    /// All features enabled (text + style + color).
    pub fn full() -> Self {
        Self { text: true, style: true, color: true }
    }

    /// Text only — no style or color data.
    pub fn text_only() -> Self {
        Self { text: true, style: false, color: false }
    }

    /// Text + style attributes, no color.
    pub fn text_and_style() -> Self {
        Self { text: true, style: true, color: false }
    }

    /// Text + color, no style attributes.
    pub fn text_and_color() -> Self {
        Self { text: true, style: false, color: true }
    }

    /// Parse from a comma-separated feature string (e.g. "text,style,color").
    ///
    /// Valid tokens: `text`, `style`, `color`.
    /// Returns an error string for unknown tokens or if no valid tokens were found.
    pub fn parse(s: &str) -> Result<Self, String> {
        let mut features = Self { text: false, style: false, color: false };
        let mut any = false;
        for token in s.split(',').map(str::trim) {
            match token {
                "text"  => { features.text  = true; any = true; }
                "style" => { features.style = true; any = true; }
                "color" => { features.color = true; any = true; }
                other   => return Err(format!("unknown render feature: '{other}' (valid: text, style, color)")),
            }
        }
        if !any {
            return Err("--render requires at least one feature: text, style, color".into());
        }
        // text is always implicitly enabled
        features.text = true;
        Ok(features)
    }

    /// Serialize as a comma-separated feature string for display/protocol use.
    pub fn to_feature_string(self) -> String {
        let mut parts = vec!["text"]; // text is always present
        if self.style { parts.push("style"); }
        if self.color { parts.push("color"); }
        parts.join(",")
    }
}

// ── Legacy RenderMode (kept for internal compatibility during transition) ──

/// Legacy render mode retained for internal compatibility helpers.
///
/// New code should use `RenderFeatures`. This enum maps 1:1:
/// - `Basic`  → `RenderFeatures { text: true, style: false, color: false }`
/// - `Styled` → `RenderFeatures { text: true, style: true,  color: false }`
/// - `Color`  → `RenderFeatures { text: true, style: true,  color: true  }`
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum RenderMode {
    /// No style data — text only.
    Basic,
    /// Text attributes (bold, italic, dim, underline, inverse) via style_map.
    Styled,
    /// Full style + color data via style_map + color_map.
    #[default]
    Color,
}

impl RenderMode {
    #[must_use]
    pub fn allows_style(&self) -> bool {
        matches!(self, RenderMode::Styled | RenderMode::Color)
    }

    #[must_use]
    pub fn allows_color(&self) -> bool {
        matches!(self, RenderMode::Color)
    }
}

impl From<RenderFeatures> for RenderMode {
    fn from(f: RenderFeatures) -> Self {
        match (f.style, f.color) {
            (false, false) => RenderMode::Basic,
            (true,  false) => RenderMode::Styled,
            (_,     true)  => RenderMode::Color,
        }
    }
}

impl From<RenderMode> for RenderFeatures {
    fn from(m: RenderMode) -> Self {
        match m {
            RenderMode::Basic  => RenderFeatures::text_only(),
            RenderMode::Styled => RenderFeatures::text_and_style(),
            RenderMode::Color  => RenderFeatures::full(),
        }
    }
}

// ── Legacy map entry structs (kept for existing `full` format) ────────────────

/// A position-based style entry for the legacy style_map.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct StyleMapEntry {
    pub r: u16,
    pub c: u16,
    pub l: u16,
    pub s: Value,
}

/// A position-based color entry for the legacy color_map.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ColorMapEntry {
    pub r: u16,
    pub c: u16,
    pub l: u16,
    pub s: Value,
}

// ── Compact style/color helpers ───────────────────────────────────────────────

/// Produce a compact style object with short keys for text attributes.
#[must_use]
pub fn compact_style(style: &CellStyle) -> Value {
    let mut map = serde_json::Map::new();
    if style.bold      { map.insert("b".into(), Value::Bool(true)); }
    if style.italic    { map.insert("i".into(), Value::Bool(true)); }
    if style.dim       { map.insert("d".into(), Value::Bool(true)); }
    if style.underline { map.insert("u".into(), Value::Bool(true)); }
    if style.inverse   { map.insert("v".into(), Value::Bool(true)); }
    Value::Object(map)
}

/// Produce a compact color object with short keys for color attributes.
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
        Color::Default             => Value::Null,
        Color::Indexed { index }  => Value::Number((*index).into()),
        Color::Rgb { r, g, b }    => Value::String(format!("#{r:02x}{g:02x}{b:02x}")),
    }
}

fn serialize_color_to_span(color: &Color) -> Option<Value> {
    match color {
        Color::Default             => None,
        Color::Indexed { index }  => Some(Value::Number((*index).into())),
        Color::Rgb { r, g, b }    => Some(Value::String(format!("#{r:02x}{g:02x}{b:02x}"))),
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

// ── Grid segmentation ─────────────────────────────────────────────────────────

/// Segment a grid into clusters (all clusters, including whitespace-only).
#[must_use]
pub fn segment_grid<G: ScreenGrid>(grid: &G) -> Vec<Cluster> {
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

// ── New json-format builder ───────────────────────────────────────────────────

/// Build a `rows` array for the `json` output format.
///
/// Each `RowEntry` contains the row index, trimmed plain text, and any style
/// spans (governed by `features`). Rows with no content are still included
/// (with empty `t`). Rows with no spans have `spans` omitted in serialization.
///
/// This is the primary output function for LLM/agent consumers.
#[must_use]
pub fn build_rows<G: ScreenGrid>(grid: &G, features: RenderFeatures) -> Vec<RowEntry> {
    let clusters = segment_grid(grid);
    let total_rows = grid.rows();
    let mut rows: Vec<RowEntry> = (0..total_rows)
        .map(|r| RowEntry { r, t: String::new(), spans: Vec::new() })
        .collect();

    // Populate text from clusters
    for cluster in &clusters {
        let row = &mut rows[cluster.row as usize];
        row.t.push_str(&cluster.text);
    }

    // Right-trim text
    for row in &mut rows {
        let trimmed = row.t.trim_end().to_string();
        row.t = trimmed;
    }

    // Populate spans from clusters
    if features.style || features.color {
        for cluster in &clusters {
            let needs_span = (features.style && has_style_attrs(&cluster.style))
                || (features.color && has_color_attrs(&cluster.style));
            if !needs_span {
                continue;
            }

            let span_style = SpanStyle {
                fg: if features.color { serialize_color_to_span(&cluster.style.fg_color) } else { None },
                bg: if features.color { serialize_color_to_span(&cluster.style.bg_color) } else { None },
                b: if features.style && cluster.style.bold      { Some(true) } else { None },
                i: if features.style && cluster.style.italic    { Some(true) } else { None },
                d: if features.style && cluster.style.dim       { Some(true) } else { None },
                u: if features.style && cluster.style.underline { Some(true) } else { None },
                v: if features.style && cluster.style.inverse   { Some(true) } else { None },
            };

            if !span_style.is_empty() {
                rows[cluster.row as usize].spans.push(SpanEntry {
                    c: cluster.col,
                    l: cluster.width,
                    s: span_style,
                });
            }
        }
    }

    rows
}

// ── Legacy map builders ───────────────────────────────────────────────────────

/// Build a style_map from a grid (legacy `full` format).
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

/// Build a color_map from a grid (legacy `full` format).
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

// ── ANSI rendering ────────────────────────────────────────────────────────────

/// Convert a `CellStyle` to an ANSI SGR escape sequence, respecting `RenderFeatures`.
#[must_use]
pub fn style_to_sgr(style: &CellStyle, features: RenderFeatures) -> String {
    let mut codes: Vec<String> = Vec::new();

    if features.style {
        if style.bold      { codes.push("1".into()); }
        if style.dim       { codes.push("2".into()); }
        if style.italic    { codes.push("3".into()); }
        if style.underline { codes.push("4".into()); }
        if style.inverse   { codes.push("7".into()); }
    }

    if features.color {
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

/// Render clusters as ANSI SGR-escaped text lines (`--format ansi`).
///
/// Produces a plain header line followed by rows of styled terminal output.
/// Each non-default-styled cluster is wrapped with reset + SGR sequences.
/// Lines containing any SGR are reset at the end. A cursor marker `[X]` is
/// inserted at the cursor position.
#[must_use]
pub fn render_ansi_lines(
    clusters: &[Cluster],
    cursor_row: u16,
    cursor_col: u16,
    rows: u16,
    cols: u16,
    features: RenderFeatures,
) -> String {
    let mut output = String::new();

    output.push_str(&format!(
        "--- Terminal {cols}x{rows} | Cursor: ({cursor_row}, {cursor_col}) ---\n"
    ));

    for row in 0..rows {
        let row_clusters: Vec<&Cluster> = clusters.iter().filter(|c| c.row == row).collect();
        let mut in_sgr = false;

        for cluster in &row_clusters {
            let is_default_whitespace =
                cluster.is_whitespace_only() && cluster.style == CellStyle::default();

            let sgr = if is_default_whitespace {
                String::new()
            } else {
                style_to_sgr(&cluster.style, features)
            };

            if !sgr.is_empty() {
                output.push_str("\x1b[0m");
                output.push_str(&sgr);
                in_sgr = true;
            } else if in_sgr {
                output.push_str("\x1b[0m");
                in_sgr = false;
            }

            emit_cluster_text(&mut output, cluster, cursor_row, cursor_col);
        }

        if in_sgr {
            output.push_str("\x1b[0m");
        }
        output.push('\n');
    }

    output
}

fn emit_cluster_text(output: &mut String, cluster: &Cluster, cursor_row: u16, cursor_col: u16) {
    if cluster.row != cursor_row
        || cursor_col < cluster.col
        || cursor_col >= cluster.col + cluster.width
    {
        output.push_str(&cluster.text);
        return;
    }

    let mut col_pos = cluster.col;
    for (i, ch) in cluster.text.chars().enumerate() {
        if col_pos == cursor_col {
            let prefix: String = cluster.text.chars().take(i).collect();
            output.push_str(&prefix);
            output.push('[');
            output.push(ch);
            output.push(']');
            let suffix: String = cluster.text.chars().skip(i + 1).collect();
            output.push_str(&suffix);
            return;
        }
        col_pos += UnicodeWidthChar::width(ch).unwrap_or(1) as u16;
    }

    output.push_str(&cluster.text);
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::elements::grid::test_support::SimpleGrid;

    // ========================================================================
    // RenderFeatures tests
    // ========================================================================

    #[test]
    fn render_features_default_is_full() {
        let f = RenderFeatures::default();
        assert!(f.text);
        assert!(f.style);
        assert!(f.color);
    }

    #[test]
    fn render_features_text_only() {
        let f = RenderFeatures::text_only();
        assert!(f.text);
        assert!(!f.style);
        assert!(!f.color);
    }

    #[test]
    fn render_features_parse_single() {
        let f = RenderFeatures::parse("text").unwrap();
        assert!(f.text);
        assert!(!f.style);
        assert!(!f.color);
    }

    #[test]
    fn render_features_parse_two() {
        let f = RenderFeatures::parse("text,color").unwrap();
        assert!(f.text);
        assert!(!f.style);
        assert!(f.color);
    }

    #[test]
    fn render_features_parse_all() {
        let f = RenderFeatures::parse("text,style,color").unwrap();
        assert!(f.text && f.style && f.color);
    }

    #[test]
    fn render_features_parse_style_implies_text() {
        // style alone should enable text implicitly
        let f = RenderFeatures::parse("style").unwrap();
        assert!(f.text);
        assert!(f.style);
        assert!(!f.color);
    }

    #[test]
    fn render_features_parse_unknown_token_errors() {
        assert!(RenderFeatures::parse("text,bold").is_err());
    }

    #[test]
    fn render_features_parse_order_independent() {
        let a = RenderFeatures::parse("color,style,text").unwrap();
        let b = RenderFeatures::parse("text,style,color").unwrap();
        assert_eq!(a, b);
    }

    #[test]
    fn render_features_to_feature_string_full() {
        let s = RenderFeatures::full().to_feature_string();
        assert_eq!(s, "text,style,color");
    }

    #[test]
    fn render_features_to_feature_string_text_only() {
        let s = RenderFeatures::text_only().to_feature_string();
        assert_eq!(s, "text");
    }

    #[test]
    fn render_features_roundtrip_via_string() {
        let original = RenderFeatures::text_and_color();
        let s = original.to_feature_string();
        let parsed = RenderFeatures::parse(&s).unwrap();
        assert_eq!(original, parsed);
    }

    // ========================================================================
    // RenderMode conversion tests
    // ========================================================================

    #[test]
    fn render_mode_default_is_color() {
        assert_eq!(RenderMode::default(), RenderMode::Color);
    }

    #[test]
    fn render_features_from_render_mode() {
        assert_eq!(RenderFeatures::from(RenderMode::Basic),  RenderFeatures::text_only());
        assert_eq!(RenderFeatures::from(RenderMode::Styled), RenderFeatures::text_and_style());
        assert_eq!(RenderFeatures::from(RenderMode::Color),  RenderFeatures::full());
    }

    #[test]
    fn render_mode_from_render_features() {
        assert_eq!(RenderMode::from(RenderFeatures::text_only()),       RenderMode::Basic);
        assert_eq!(RenderMode::from(RenderFeatures::text_and_style()),  RenderMode::Styled);
        assert_eq!(RenderMode::from(RenderFeatures::full()),            RenderMode::Color);
        // color without style → Color (has color)
        assert_eq!(RenderMode::from(RenderFeatures::text_and_color()),  RenderMode::Color);
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
    // build_rows tests
    // ========================================================================

    #[test]
    fn build_rows_text_only_no_spans() {
        let grid = SimpleGrid::from_text(&["hello", "world"], 5);
        let rows = build_rows(&grid, RenderFeatures::text_only());
        assert_eq!(rows.len(), 2);
        assert_eq!(rows[0].t, "hello");
        assert_eq!(rows[1].t, "world");
        assert!(rows[0].spans.is_empty());
        assert!(rows[1].spans.is_empty());
    }

    #[test]
    fn build_rows_with_bold_span() {
        let mut grid = SimpleGrid::from_text(&["Hello World"], 11);
        let bold = CellStyle::new().with_bold(true);
        grid.style_range(0, 0, 5, bold);

        let rows = build_rows(&grid, RenderFeatures::full());
        assert_eq!(rows.len(), 1);
        assert_eq!(rows[0].t, "Hello World");
        // Bold span covering "Hello"
        let bold_span = rows[0].spans.iter().find(|s| s.s.b == Some(true));
        assert!(bold_span.is_some(), "should have a bold span");
        let span = bold_span.unwrap();
        assert_eq!(span.c, 0);
        assert_eq!(span.l, 5);
    }

    #[test]
    fn build_rows_with_color_span() {
        let mut grid = SimpleGrid::from_text(&["RedText"], 7);
        grid.style_range(0, 0, 3, CellStyle::new().with_fg(Color::indexed(1)));

        let rows = build_rows(&grid, RenderFeatures::full());
        let color_span = rows[0].spans.iter().find(|s| s.s.fg.is_some());
        assert!(color_span.is_some());
        let span = color_span.unwrap();
        assert_eq!(span.c, 0);
        assert_eq!(span.l, 3);
        assert_eq!(span.s.fg, Some(Value::Number(1.into())));
    }

    #[test]
    fn build_rows_text_only_filters_color() {
        let mut grid = SimpleGrid::from_text(&["RedBold"], 7);
        grid.style_range(0, 0, 7, CellStyle::new().with_bold(true).with_fg(Color::indexed(1)));

        let rows = build_rows(&grid, RenderFeatures::text_only());
        assert!(rows[0].spans.is_empty(), "text_only should produce no spans");
    }

    #[test]
    fn build_rows_style_only_no_color_in_spans() {
        let mut grid = SimpleGrid::from_text(&["test"], 4);
        grid.style_range(0, 0, 4, CellStyle::new().with_bold(true).with_fg(Color::indexed(1)));

        let rows = build_rows(&grid, RenderFeatures::text_and_style());
        assert!(!rows[0].spans.is_empty());
        let span = &rows[0].spans[0];
        assert_eq!(span.s.b, Some(true));
        assert!(span.s.fg.is_none(), "style-only should omit fg color");
    }

    #[test]
    fn build_rows_color_only_no_style_in_spans() {
        let mut grid = SimpleGrid::from_text(&["test"], 4);
        grid.style_range(0, 0, 4, CellStyle::new().with_bold(true).with_fg(Color::indexed(1)));

        let rows = build_rows(&grid, RenderFeatures::text_and_color());
        assert!(!rows[0].spans.is_empty());
        let span = &rows[0].spans[0];
        assert!(span.s.b.is_none(), "color-only should omit bold");
        assert_eq!(span.s.fg, Some(Value::Number(1.into())));
    }

    #[test]
    fn build_rows_default_style_no_spans() {
        let grid = SimpleGrid::from_text(&["plain text"], 10);
        let rows = build_rows(&grid, RenderFeatures::full());
        assert!(rows[0].spans.is_empty(), "default-style text should have no spans");
    }

    #[test]
    fn build_rows_row_count_matches_grid() {
        let grid = SimpleGrid::from_text(&["a", "b", "c"], 1);
        let rows = build_rows(&grid, RenderFeatures::full());
        assert_eq!(rows.len(), 3);
        assert_eq!(rows[0].r, 0);
        assert_eq!(rows[1].r, 1);
        assert_eq!(rows[2].r, 2);
    }

    #[test]
    fn build_rows_text_is_right_trimmed() {
        let grid = SimpleGrid::from_text(&["hi   "], 5);
        let rows = build_rows(&grid, RenderFeatures::full());
        assert_eq!(rows[0].t, "hi");
    }

    #[test]
    fn build_rows_multirow_mixed_styling() {
        let mut grid = SimpleGrid::from_text(&["bold line", "plain line"], 10);
        grid.style_range(0, 0, 9, CellStyle::new().with_bold(true));

        let rows = build_rows(&grid, RenderFeatures::full());
        assert!(!rows[0].spans.is_empty(), "row 0 should have spans");
        assert!(rows[1].spans.is_empty(), "row 1 should have no spans");
    }

    #[test]
    fn build_rows_rgb_color_hex_format() {
        let mut grid = SimpleGrid::from_text(&["test"], 4);
        grid.style_range(0, 0, 4, CellStyle::new().with_fg(Color::rgb(255, 128, 0)));

        let rows = build_rows(&grid, RenderFeatures::full());
        let span = &rows[0].spans[0];
        assert_eq!(span.s.fg, Some(Value::String("#ff8000".into())));
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
        assert_eq!(clusters.len(), 1);
        assert_eq!(clusters[0].text, "A B");
    }

    // ========================================================================
    // style_to_sgr tests (now uses RenderFeatures)
    // ========================================================================

    #[test]
    fn sgr_bold_only() {
        let style = CellStyle::new().with_bold(true);
        assert_eq!(style_to_sgr(&style, RenderFeatures::full()), "\x1b[1m");
    }

    #[test]
    fn sgr_italic_only() {
        let style = CellStyle::new().with_italic(true);
        assert_eq!(style_to_sgr(&style, RenderFeatures::full()), "\x1b[3m");
    }

    #[test]
    fn sgr_dim_only() {
        let style = CellStyle::new().with_dim(true);
        assert_eq!(style_to_sgr(&style, RenderFeatures::full()), "\x1b[2m");
    }

    #[test]
    fn sgr_underline_only() {
        let style = CellStyle::new().with_underline(true);
        assert_eq!(style_to_sgr(&style, RenderFeatures::full()), "\x1b[4m");
    }

    #[test]
    fn sgr_inverse_only() {
        let style = CellStyle::new().with_inverse(true);
        assert_eq!(style_to_sgr(&style, RenderFeatures::full()), "\x1b[7m");
    }

    #[test]
    fn sgr_combined_attrs() {
        let style = CellStyle::new().with_bold(true).with_underline(true);
        assert_eq!(style_to_sgr(&style, RenderFeatures::full()), "\x1b[1;4m");
    }

    #[test]
    fn sgr_fg_indexed_standard() {
        let style = CellStyle::new().with_fg(Color::indexed(1));
        assert_eq!(style_to_sgr(&style, RenderFeatures::full()), "\x1b[31m");
    }

    #[test]
    fn sgr_fg_indexed_extended() {
        let style = CellStyle::new().with_fg(Color::indexed(100));
        assert_eq!(style_to_sgr(&style, RenderFeatures::full()), "\x1b[38;5;100m");
    }

    #[test]
    fn sgr_fg_rgb() {
        let style = CellStyle::new().with_fg(Color::rgb(255, 0, 128));
        assert_eq!(
            style_to_sgr(&style, RenderFeatures::full()),
            "\x1b[38;2;255;0;128m"
        );
    }

    #[test]
    fn sgr_bg_indexed_standard() {
        let style = CellStyle::new().with_bg(Color::indexed(2));
        assert_eq!(style_to_sgr(&style, RenderFeatures::full()), "\x1b[42m");
    }

    #[test]
    fn sgr_bg_rgb() {
        let style = CellStyle::new().with_bg(Color::rgb(10, 20, 30));
        assert_eq!(
            style_to_sgr(&style, RenderFeatures::full()),
            "\x1b[48;2;10;20;30m"
        );
    }

    #[test]
    fn sgr_style_only_features_no_colors() {
        let style = CellStyle::new().with_bold(true).with_fg(Color::indexed(1));
        assert_eq!(style_to_sgr(&style, RenderFeatures::text_and_style()), "\x1b[1m");
    }

    #[test]
    fn sgr_color_only_features_no_attrs() {
        let style = CellStyle::new().with_bold(true).with_fg(Color::indexed(1));
        assert_eq!(style_to_sgr(&style, RenderFeatures::text_and_color()), "\x1b[31m");
    }

    #[test]
    fn sgr_text_only_features_empty() {
        let style = CellStyle::new().with_bold(true).with_fg(Color::indexed(1));
        assert_eq!(style_to_sgr(&style, RenderFeatures::text_only()), "");
    }

    #[test]
    fn sgr_default_style_always_empty() {
        let style = CellStyle::default();
        assert_eq!(style_to_sgr(&style, RenderFeatures::full()), "");
        assert_eq!(style_to_sgr(&style, RenderFeatures::text_only()), "");
    }

    // ========================================================================
    // render_ansi_lines tests (now uses RenderFeatures)
    // ========================================================================

    #[test]
    fn render_plain_text_no_sgr() {
        let grid = SimpleGrid::from_text(&["hello"], 5);
        let clusters = segment_grid(&grid);
        let out = render_ansi_lines(&clusters, 99, 99, 1, 5, RenderFeatures::full());
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
        let out = render_ansi_lines(&clusters, 99, 99, 1, 4, RenderFeatures::full());
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
        let out = render_ansi_lines(&clusters, 0, 2, 1, 4, RenderFeatures::full());
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
        let out = render_ansi_lines(&clusters, 99, 99, 2, 4, RenderFeatures::full());
        let lines: Vec<&str> = out.lines().collect();
        assert_eq!(lines.len(), 3);
        assert!(lines[1].contains("row0"));
        assert!(lines[2].contains("row1"));
    }

    #[test]
    fn render_empty_grid() {
        let out = render_ansi_lines(&[], 0, 0, 0, 10, RenderFeatures::full());
        let lines: Vec<&str> = out.lines().collect();
        assert_eq!(lines.len(), 1);
        assert!(lines[0].starts_with("--- Terminal"));
    }

    #[test]
    fn render_header_no_sgr() {
        let mut grid = SimpleGrid::from_text(&["styled"], 6);
        grid.style_range(0, 0, 6, CellStyle::new().with_bold(true));
        let clusters = segment_grid(&grid);
        let out = render_ansi_lines(&clusters, 0, 0, 1, 6, RenderFeatures::full());
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
        let out = render_ansi_lines(&clusters, 99, 99, 1, 4, RenderFeatures::full());
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
        let out = render_ansi_lines(&clusters, 0, 1, 1, 4, RenderFeatures::full());
        let line = out.lines().nth(1).unwrap();
        assert!(
            line.contains("A[B]CD"),
            "Cursor at col 1 should wrap 'B': {line}"
        );
    }

    #[test]
    fn render_default_whitespace_no_sgr() {
        let grid = SimpleGrid::from_text(&["Hi   "], 5);
        let clusters = segment_grid(&grid);
        let out = render_ansi_lines(&clusters, 99, 99, 1, 5, RenderFeatures::full());
        let line = out.lines().nth(1).unwrap();
        assert!(
            !line.contains('\x1b'),
            "Default whitespace should not emit SGR"
        );
    }

    #[test]
    fn render_style_features_filters_colors() {
        let mut grid = SimpleGrid::from_text(&["test"], 4);
        grid.style_range(
            0, 0, 4,
            CellStyle::new().with_bold(true).with_fg(Color::indexed(1)),
        );
        let clusters = segment_grid(&grid);
        let out = render_ansi_lines(&clusters, 99, 99, 1, 4, RenderFeatures::text_and_style());
        let line = out.lines().nth(1).unwrap();
        assert!(line.contains("\x1b[1m"), "Bold should be present");
        assert!(
            !line.contains("31"),
            "Fg color should be filtered with style-only features"
        );
    }

    #[test]
    fn render_text_only_strips_all_sgr() {
        let mut grid = SimpleGrid::from_text(&["color"], 5);
        grid.style_range(
            0, 0, 5,
            CellStyle::new().with_bold(true).with_fg(Color::indexed(1)),
        );
        let clusters = segment_grid(&grid);
        let out = render_ansi_lines(&clusters, 99, 99, 1, 5, RenderFeatures::text_only());
        for line in out.lines() {
            assert!(
                !line.contains('\x1b'),
                "text_only features should strip all SGR: {line}"
            );
        }
    }

    #[test]
    fn render_mixed_styled_unstyled_lines() {
        let mut grid = SimpleGrid::from_text(&["styled", "plain."], 6);
        grid.style_range(0, 0, 6, CellStyle::new().with_bold(true));
        let clusters = segment_grid(&grid);
        let out = render_ansi_lines(&clusters, 99, 99, 2, 6, RenderFeatures::full());
        let lines: Vec<&str> = out.lines().collect();
        assert!(
            lines[1].contains('\x1b'),
            "Styled row should have SGR: {}",
            lines[1]
        );
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
        let out = render_ansi_lines(&clusters, 0, 3, 1, 4, RenderFeatures::full());
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
        let out = render_ansi_lines(&clusters, 99, 99, 2, 3, RenderFeatures::full());
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
        grid.style_range(0, 0, 5, CellStyle::new().with_bold(true));
        let clusters = segment_grid(&grid);
        let out = render_ansi_lines(&clusters, 99, 99, 1, 5, RenderFeatures::full());
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
        let out = render_ansi_lines(&clusters, 99, 99, 1, 4, RenderFeatures::full());
        let line = out.lines().nth(1).unwrap();
        assert!(
            line.contains("\x1b[0m"),
            "Reset expected between style transitions: {line}"
        );
        assert!(line.contains("\x1b[1m"), "Bold SGR expected: {line}");
        assert!(line.contains("\x1b[3m"), "Italic SGR expected: {line}");
    }

    #[test]
    fn render_cursor_on_styled_whitespace() {
        let mut grid = SimpleGrid::from_text(&["A B "], 4);
        grid.style_range(0, 0, 4, CellStyle::new().with_italic(true));
        let clusters = segment_grid(&grid);
        let out = render_ansi_lines(&clusters, 0, 1, 1, 4, RenderFeatures::full());
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
