use font_kit::family_handle::FamilyHandle;
use font_kit::handle::Handle;
use font_kit::source::SystemSource;
use rusttype::{Font, Scale};
use std::fs;

pub fn load_system_font_bytes(font_name: &str) -> Option<Vec<u8>> {
    let source = SystemSource::new();
    let handle = source
        .select_by_postscript_name(font_name)
        .ok()
        .or_else(|| {
            source
                .select_family_by_name(font_name)
                .ok()
                .and_then(|family: FamilyHandle| family.fonts().get(0).cloned())
        })?;

    let path = match handle {
        Handle::Path { path, .. } => path,
        Handle::Memory { .. } => return None, // skipping memory handles for simplicity
    };

    fs::read(path).ok()
}

pub fn measure_text_width(text: &str, font_data: &[u8], pixel_height: f32) -> f32 {
    let font = Font::try_from_bytes(font_data).expect("Invalid font");
    let scale = Scale::uniform(pixel_height);

    font.layout(text, scale, rusttype::point(0.0, 0.0))
        .map(|g| g.unpositioned().h_metrics().advance_width)
        .sum()
}

pub fn measure_text_height(font_data: &[u8], pixel_height: f32) -> f32 {
    let font = Font::try_from_bytes(font_data).expect("Invalid font");
    let scale = Scale::uniform(pixel_height);
    let v_metrics = font.v_metrics(scale);

    // Return the total height (ascent + descent)
    v_metrics.ascent - v_metrics.descent
}
