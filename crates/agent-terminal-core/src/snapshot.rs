//! Screen state capture and change detection.
//!
//! This module provides types for capturing terminal screen state, including
//! text content, cursor position, and detected UI elements.
//!
//! # Snapshot Formats
//!
//! The daemon supports two output formats:
//!
//! | `--format` | Content | Use Case |
//! |------------|---------|----------|
//! | **ansi** | ANSI-escaped text (default) | Human-readable terminal output |
//! | **json** | JSON with `rows` array | LLM/agent structured consumption |
//!
//! The `--render` flag controls which data is included. It accepts a
//! comma-separated list of features: `text`, `style`, `color`.
//! Default is `text,style,color` (all features enabled).
//!
//! # JSON Format (`--format json`)
//!
//! Each row is a self-contained object with its text and optional style spans:
//!
//! ```json
//! {
//!   "rows": [
//!     { "r": 0, "t": "hello world", "spans": [
//!       { "c": 6, "l": 5, "s": { "fg": "#ff0000", "b": true } }
//!     ]},
//!     { "r": 1, "t": "plain text" }
//!   ]
//! }
//! ```
//!
//! JQ example — extract rows 10–20:
//! ```bash
//! agent-terminal snapshot --format json | jq '.rows[] | select(.r >= 10 and .r <= 20)'
//! ```
//!
//! # Change Detection
//!
//! The `content_hash` field provides efficient change detection. Agents can
//! compare hashes across snapshots without parsing the full element list:
//!
//! ```ignore
//! if new_snapshot.content_hash != old_snapshot.content_hash {
//!     // Screen changed, re-analyze elements
//! }
//! ```

use serde::{Deserialize, Serialize};

use crate::elements::Element;
use crate::format::{ColorMapEntry, StyleMapEntry};

/// Terminal dimensions.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub struct TerminalSize {
    pub cols: u16,
    pub rows: u16,
}

/// Cursor position and visibility.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub struct CursorState {
    pub row: u16,
    pub col: u16,
    pub visible: bool,
}

/// A single row of screen text (legacy `full` format).
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct TextLine {
    /// Row index (0-based).
    pub r: u16,
    /// Right-trimmed text content of this row.
    pub t: String,
}

/// Style attributes for a span in the `json` format.
///
/// All fields are optional; only non-default attributes are serialized.
/// This struct is intentionally extensible — future terminal capabilities
/// (double underline, squiggle underline, hyperlinks, etc.) will be added
/// as new optional fields here without breaking existing consumers.
///
/// Short key names minimize token usage for LLM/agent consumers:
/// - `fg` / `bg` — foreground / background color (hex `"#rrggbb"` or indexed `N`)
/// - `b` — bold
/// - `i` — italic
/// - `d` — dim
/// - `u` — underline
/// - `v` — inverse (reverse video)
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, Default)]
pub struct SpanStyle {
    /// Foreground color: `"#rrggbb"` (RGB) or integer index (256-color).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub fg: Option<serde_json::Value>,
    /// Background color: `"#rrggbb"` (RGB) or integer index (256-color).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub bg: Option<serde_json::Value>,
    /// Bold text attribute.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub b: Option<bool>,
    /// Italic text attribute.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub i: Option<bool>,
    /// Dim (faint) text attribute.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub d: Option<bool>,
    /// Underline text attribute.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub u: Option<bool>,
    /// Inverse (reverse video) text attribute.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub v: Option<bool>,
    // Future extensibility (add here without breaking change):
    // pub u2: Option<bool>,   // double underline
    // pub sq: Option<bool>,   // squiggle underline
    // pub href: Option<String>, // hyperlink URL
}

impl SpanStyle {
    /// Returns true if this style has no non-default attributes set.
    pub fn is_empty(&self) -> bool {
        self.fg.is_none()
            && self.bg.is_none()
            && self.b.is_none()
            && self.i.is_none()
            && self.d.is_none()
            && self.u.is_none()
            && self.v.is_none()
    }
}

/// A styled span within a row in the `json` format.
///
/// Describes a contiguous run of characters sharing the same style.
/// Column offsets are 0-based; length is in display columns.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct SpanEntry {
    /// Column start (0-based).
    pub c: u16,
    /// Length in display columns.
    pub l: u16,
    /// Style attributes for this span.
    pub s: SpanStyle,
}

/// A single terminal row in the `json` format.
///
/// Contains the full plain text of the row and any styled spans.
/// An agent can extract plain text with `.t` and correlate styling
/// with `.spans` — all within a single self-contained object.
///
/// Rows with no styling omit the `spans` field entirely.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct RowEntry {
    /// Row index (0-based).
    pub r: u16,
    /// Right-trimmed plain text content of this row.
    pub t: String,
    /// Style spans for this row. Omitted when empty (no styling).
    #[serde(skip_serializing_if = "Vec::is_empty", default)]
    pub spans: Vec<SpanEntry>,
}

/// Complete screen state snapshot.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ScreenState {
    pub snapshot_id: u64,
    pub size: TerminalSize,
    /// TERM value for this session (e.g. "xterm-256color").
    pub term: String,
    pub cursor: CursorState,
    /// Screen as an array of row objects (`--format json`).
    ///
    /// Each row contains its index (`r`), plain text (`t`), and styled spans (`spans`).
    /// This is the primary output field for LLM/agent consumption. Rows with no
    /// styling omit the `spans` field; rows with entirely default text are still
    /// included (with empty `t`).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub rows: Option<Vec<RowEntry>>,
    /// Screen text as an array of rows (legacy `full` format).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub text: Option<Vec<TextLine>>,
    /// Detected interactive UI elements.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub elements: Option<Vec<Element>>,
    /// Hash of screen content for change detection.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub content_hash: Option<u64>,
    /// Position-based style attribute map (legacy `full` format).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub style_map: Option<Vec<StyleMapEntry>>,
    /// Position-based color attribute map (legacy `full` format).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub color_map: Option<Vec<ColorMapEntry>>,
}

impl ScreenState {
    pub fn empty(cols: u16, rows: u16) -> Self {
        Self {
            snapshot_id: 0,
            size: TerminalSize { cols, rows },
            term: "xterm-256color".to_string(),
            cursor: CursorState {
                row: 0,
                col: 0,
                visible: true,
            },
            rows: None,
            text: None,
            elements: None,
            content_hash: None,
            style_map: None,
            color_map: None,
        }
    }
}

/// Compute a content hash from screen text.
///
/// Uses FNV-1a, a fast non-cryptographic hash suitable for change detection.
#[must_use]
pub fn compute_content_hash(text: &str) -> u64 {
    // FNV-1a parameters for 64-bit
    const FNV_OFFSET: u64 = 0xcbf29ce484222325;
    const FNV_PRIME: u64 = 0x00000100000001B3;

    let mut hash = FNV_OFFSET;
    for byte in text.bytes() {
        hash ^= u64::from(byte);
        hash = hash.wrapping_mul(FNV_PRIME);
    }
    hash
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn content_hash_deterministic() {
        let text = "Hello, World!";
        let hash1 = compute_content_hash(text);
        let hash2 = compute_content_hash(text);
        assert_eq!(hash1, hash2);
    }

    #[test]
    fn content_hash_differs_for_different_text() {
        let hash1 = compute_content_hash("Hello");
        let hash2 = compute_content_hash("World");
        assert_ne!(hash1, hash2);
    }

    #[test]
    fn content_hash_empty_string() {
        // Empty string should return the FNV-1a offset basis
        let hash = compute_content_hash("");
        assert_eq!(hash, 0xcbf29ce484222325);
    }

    #[test]
    fn content_hash_single_char_difference() {
        // Even a single character difference should produce different hashes
        let hash1 = compute_content_hash("test");
        let hash2 = compute_content_hash("tess");
        assert_ne!(hash1, hash2);
    }

    #[test]
    fn content_hash_unicode() {
        // Unicode text should hash consistently
        let text = "日本語テスト 🚀";
        let hash1 = compute_content_hash(text);
        let hash2 = compute_content_hash(text);
        assert_eq!(hash1, hash2);
        // Should differ from ASCII
        assert_ne!(hash1, compute_content_hash("ascii"));
    }

    // ========================================================================
    // SpanStyle tests
    // ========================================================================

    #[test]
    fn span_style_default_is_empty() {
        let s = SpanStyle::default();
        assert!(s.is_empty());
    }

    #[test]
    fn span_style_with_bold_not_empty() {
        let s = SpanStyle { b: Some(true), ..Default::default() };
        assert!(!s.is_empty());
    }

    #[test]
    fn span_style_serialize_skips_none_fields() {
        let s = SpanStyle { b: Some(true), ..Default::default() };
        let json = serde_json::to_string(&s).unwrap();
        assert!(json.contains("\"b\":true"));
        assert!(!json.contains("\"fg\""));
        assert!(!json.contains("\"i\""));
    }

    #[test]
    fn span_style_all_fields_serialize() {
        let s = SpanStyle {
            fg: Some(serde_json::Value::String("#ff0000".into())),
            bg: Some(serde_json::Value::Number(1.into())),
            b: Some(true),
            i: Some(true),
            d: Some(true),
            u: Some(true),
            v: Some(true),
        };
        let json = serde_json::to_string(&s).unwrap();
        assert!(json.contains("\"fg\":\"#ff0000\""));
        assert!(json.contains("\"bg\":1"));
        assert!(json.contains("\"b\":true"));
        assert!(json.contains("\"i\":true"));
        assert!(json.contains("\"d\":true"));
        assert!(json.contains("\"u\":true"));
        assert!(json.contains("\"v\":true"));
    }

    #[test]
    fn span_style_roundtrip() {
        let s = SpanStyle {
            fg: Some(serde_json::Value::String("#aabbcc".into())),
            b: Some(true),
            ..Default::default()
        };
        let json = serde_json::to_string(&s).unwrap();
        let s2: SpanStyle = serde_json::from_str(&json).unwrap();
        assert_eq!(s, s2);
    }

    // ========================================================================
    // RowEntry tests
    // ========================================================================

    #[test]
    fn row_entry_no_spans_omits_field() {
        let row = RowEntry { r: 0, t: "hello".into(), spans: vec![] };
        let json = serde_json::to_string(&row).unwrap();
        assert!(!json.contains("spans"), "empty spans should be omitted: {json}");
    }

    #[test]
    fn row_entry_with_spans_serializes() {
        let row = RowEntry {
            r: 3,
            t: "hello world".into(),
            spans: vec![SpanEntry {
                c: 6,
                l: 5,
                s: SpanStyle { fg: Some(serde_json::Value::String("#ff0000".into())), ..Default::default() },
            }],
        };
        let json = serde_json::to_string(&row).unwrap();
        assert!(json.contains("\"r\":3"));
        assert!(json.contains("\"t\":\"hello world\""));
        assert!(json.contains("\"spans\""));
        assert!(json.contains("\"c\":6"));
        assert!(json.contains("\"l\":5"));
        assert!(json.contains("#ff0000"));
    }

    #[test]
    fn row_entry_roundtrip() {
        let row = RowEntry {
            r: 1,
            t: "test line".into(),
            spans: vec![
                SpanEntry { c: 0, l: 4, s: SpanStyle { b: Some(true), ..Default::default() } },
                SpanEntry { c: 5, l: 4, s: SpanStyle { fg: Some(serde_json::Value::Number(1.into())), ..Default::default() } },
            ],
        };
        let json = serde_json::to_string(&row).unwrap();
        let row2: RowEntry = serde_json::from_str(&json).unwrap();
        assert_eq!(row, row2);
    }

    // ========================================================================
    // ScreenState rows field tests
    // ========================================================================

    #[test]
    fn screen_state_empty_has_no_rows() {
        let s = ScreenState::empty(80, 24);
        assert!(s.rows.is_none());
    }

    #[test]
    fn screen_state_with_rows_serializes() {
        let mut s = ScreenState::empty(80, 24);
        s.rows = Some(vec![
            RowEntry { r: 0, t: "line one".into(), spans: vec![] },
            RowEntry { r: 1, t: "line two".into(), spans: vec![] },
        ]);
        let json = serde_json::to_string(&s).unwrap();
        assert!(json.contains("\"rows\""));
        assert!(json.contains("\"line one\""));
        assert!(json.contains("\"line two\""));
    }

    #[test]
    fn screen_state_rows_none_omitted() {
        let s = ScreenState::empty(80, 24);
        let json = serde_json::to_string(&s).unwrap();
        // "rows" appears inside "size":{"cols":80,"rows":24} — that is expected.
        // What must be absent is the top-level "rows" array field (i.e. "rows":[...]).
        assert!(!json.contains("\"rows\":["), "rows: None should be omitted");
    }
}
