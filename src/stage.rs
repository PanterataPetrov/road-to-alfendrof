use serde::Deserialize;

/// Loaded from `assets/stages/*.ron`. Placeholder visuals until real backgrounds exist.
/// All stages currently share the same playable width (`GameState::STAGE_HALF_WIDTH`).
#[derive(Clone, Debug, Deserialize)]
pub struct StageData {
    pub name: String,
    /// Background colour (r, g, b) in 0..1.
    pub sky: (f32, f32, f32),
    /// Colour of the background pillars.
    pub pillar: (f32, f32, f32),
    /// The floor alternates between these two colours so camera movement is visible.
    pub floor_a: (f32, f32, f32),
    pub floor_b: (f32, f32, f32),
    /// Pixel-art background layers, back to front. When empty, plain pillars are drawn instead.
    #[serde(default)]
    pub layers: Vec<LayerData>,
    /// Image repeated along the floor (path relative to `assets/`), instead of the two colours.
    #[serde(default)]
    pub floor_tile: Option<String>,
}

/// One parallax background layer. Images are 216 px tall and centred on the screen.
#[derive(Clone, Debug, Deserialize)]
pub struct LayerData {
    /// Path relative to `assets/`.
    pub image: String,
    /// How much the layer follows the camera: 1 stays fixed on screen (sky), 0 is fixed in the
    /// world (the fence the fighters stand in front of).
    pub parallax: f32,
    pub z: f32,
    /// Make the layer slowly pulse, for things like portals and fires.
    #[serde(default)]
    pub glow: bool,
}

impl StageData {
    pub fn from_ron(text: &str) -> Result<Self, ron::error::SpannedError> {
        ron::from_str(text)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn stage_files_parse() {
        let s = StageData::from_ron(include_str!("../assets/stages/galata.ron")).unwrap();
        assert_eq!(s.name, "Galata Tower");
        assert!(s.layers.iter().any(|l| l.glow), "the portal layer should glow");
    }
}
