use std::path::PathBuf;
use std::collections::BTreeMap;

use serde::{Serialize, Deserialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FontMetadata {
    pub atlas_path: PathBuf,
    pub line_height: u32,
    pub glyphs: BTreeMap<String, GlyphMetadata>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GlyphMetadata {
    pub x: u32,
    pub y: u32,
    pub width: u32,
    pub height: u32,
    pub hori_bearing_x: i32,
    pub hori_bearing_y: i32,
    pub advance_x: i32,
    pub advance_y: i32,
}
