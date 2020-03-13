use std::path::{Path, PathBuf};
use std::collections::BTreeMap;

use serde::Deserialize;
use fontgen_export::{FontMetadata, GlyphMetadata};

#[derive(Deserialize)]
pub struct FontSpec<'a> {
    #[serde(borrow)]
    pub font_path: &'a Path,

    pub font_size: u32,
    pub cell_width: u32,
    pub cell_height: u32,
    pub atlas_padding_x: u32,
    pub atlas_padding_y: u32,
    pub atlas_width: u32,
    pub atlas_height: u32,
    pub glyphs: Vec<String>,
}

fn main() {
    let clap_matches = clap::App::new(env!("CARGO_PKG_NAME"))
        .version(env!("CARGO_PKG_VERSION"))
        .author(env!("CARGO_PKG_AUTHORS"))
        .about(env!("CARGO_PKG_DESCRIPTION"))
        .arg(
            clap::Arg::with_name("FONT_SPEC_PATH")
                .takes_value(true)
                .required(true),
        )
        .arg(
            clap::Arg::with_name("output_path")
                .short("o")
                .value_name("PATH")
        )
        .get_matches();

    let freetype = freetype::Library::init().expect("failed to init FreeType");
    let font_spec_path = Path::new(clap_matches.value_of("FONT_SPEC_PATH").unwrap());
    let font_spec_bytes = std::fs::read(font_spec_path).expect("failed to open font spec file");
    let font_spec: FontSpec =
        serde_json::from_slice(&font_spec_bytes).expect("malformed font spec file");
    let font_path = font_spec_path.parent().unwrap().join(font_spec.font_path);
    println!("Loading font: {}", font_path.display());
    let font_face = freetype
        .new_face(font_path, 0)
        .expect("failed to create font face");
    font_face.set_pixel_sizes(font_spec.font_size, 0).unwrap();

    let atlas_width = font_spec.atlas_width as usize;
    let atlas_padding_x = font_spec.atlas_padding_x as usize;
    let atlas_padding_y = font_spec.atlas_padding_y as usize;
    let atlas_width_cells = (font_spec.atlas_width - font_spec.atlas_padding_x * 2) / font_spec.cell_width;
    let mut atlas_buf = vec![0u8; font_spec.atlas_width as usize * font_spec.atlas_height as usize];
    let mut glyph_metadata = BTreeMap::new();
    for (i, glyph) in font_spec.glyphs.into_iter().enumerate() {
        let cell_x = i % atlas_width_cells as usize;
        let cell_y = i / atlas_width_cells as usize;
        // Just use the first char. In the future, full unicode graphemes could be considered.
        let glyph_char = match glyph.chars().next() {
            Some(ch) => ch,
            None => {
                eprintln!("Invalid glyph: `{}`", glyph);
                continue;
            }
        };
        if font_face
            .load_char(glyph_char as usize, freetype::face::LoadFlag::RENDER)
            .is_err()
        {
            eprintln!("Failed to load glyph: `{}`", glyph_char);
            continue;
        }
        let atlas_start_x = cell_x * font_spec.cell_width as usize + atlas_padding_x;
        let atlas_start_y = cell_y * font_spec.cell_height as usize + atlas_padding_y;
        let glyph_advance = font_face.glyph().advance();
        let glyph_metrics = font_face.glyph().metrics();
        let glyph_bitmap = font_face.glyph().bitmap();
        for y in 0..glyph_bitmap.rows() as usize {
            let atlas_y = atlas_start_y + y;
            for x in 0..glyph_bitmap.width() as usize {
                let atlas_x = atlas_start_x + x;
                atlas_buf[atlas_y * atlas_width + atlas_x] =
                    glyph_bitmap.buffer()[y * glyph_bitmap.width() as usize + x];
            }
        }
        glyph_metadata.insert(glyph, GlyphMetadata {
            x: atlas_start_x as u32,
            y: atlas_start_y as u32,
            width: glyph_bitmap.width() as u32,
            height: glyph_bitmap.rows() as u32,
            hori_bearing_x: (glyph_metrics.horiBearingX >> 6) as i32,
            hori_bearing_y: (glyph_metrics.horiBearingY >> 6) as i32,
            advance_x: (glyph_advance.x >> 6) as i32,
            advance_y: (glyph_advance.y >> 6) as i32,
        });
    }
    let out_path = clap_matches
        .value_of("output_path")
        .map(PathBuf::from)
        .unwrap_or_else(|| PathBuf::from("./out"));
    println!("Saving as: {}.[png,json]", out_path.display());
    let out_image_path = out_path.with_extension("png");
    let out_image_file = std::fs::File::create(&out_image_path).unwrap();
    let encoder = png::Encoder::new(out_image_file, font_spec.atlas_width, font_spec.atlas_height);
    let mut writer = encoder.write_header().unwrap();
    writer.write_image_data(&atlas_buf).unwrap();
    let out_metadata_json = serde_json::to_vec_pretty(&FontMetadata {
        line_height: (font_face.size_metrics().unwrap().height >> 6) as u32,
        atlas_path: Path::new(".").join(out_image_path.file_name().unwrap()),
        glyphs: glyph_metadata,
    })
    .unwrap();
    std::fs::write(out_path.with_extension("json"), out_metadata_json).unwrap();
}
