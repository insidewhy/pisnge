use resvg::tiny_skia;
use resvg::usvg::{Options, Tree};
use std::error::Error;
use std::fmt;

#[derive(Debug)]
pub enum PngError {
    SvgParse(String),
    Render(String),
}

impl fmt::Display for PngError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            PngError::SvgParse(msg) => write!(f, "SVG parsing error: {}", msg),
            PngError::Render(msg) => write!(f, "PNG rendering error: {}", msg),
        }
    }
}

impl Error for PngError {}

pub fn svg_to_png(
    svg_content: &str,
    width: u32,
    height: u32,
    font_name: &str,
) -> Result<Vec<u8>, PngError> {
    let mut fontdb = resvg::usvg::fontdb::Database::new();
    fontdb.load_system_fonts();

    // Add fallback fonts for Linux systems
    fontdb.set_serif_family("Liberation Serif");
    fontdb.set_sans_serif_family("Liberation Sans");
    fontdb.set_monospace_family("Liberation Mono");

    let options = Options {
        fontdb: std::sync::Arc::new(fontdb),
        font_family: font_name.to_string(),
        font_size: 12.0,
        ..Options::default()
    };

    let tree =
        Tree::from_str(svg_content, &options).map_err(|e| PngError::SvgParse(e.to_string()))?;

    let mut pixmap = tiny_skia::Pixmap::new(width, height)
        .ok_or_else(|| PngError::Render("Failed to create pixmap".to_string()))?;

    // Fill with white background
    pixmap.fill(tiny_skia::Color::WHITE);

    resvg::render(&tree, tiny_skia::Transform::default(), &mut pixmap.as_mut());

    pixmap
        .encode_png()
        .map_err(|e| PngError::Render(e.to_string()))
}
