use serde::Deserialize;

/// Loaded from `assets/stages/*.ron`. All stages share the same playable width (`GameState::STAGE_HALF_WIDTH`).
#[derive(Clone, Debug, Deserialize)]
pub struct StageData {
    pub name: String,
    /// Background colour (r, g, b), 0..1.
    pub sky: (f32, f32, f32),
    /// Colour of the background pillars.
    pub pillar: (f32, f32, f32),
    /// The floor alternates between these two colours.
    pub floor_a: (f32, f32, f32),
    pub floor_b: (f32, f32, f32),
    /// Background layers, back to front. Plain pillars when empty.
    #[serde(default)]
    pub layers: Vec<LayerData>,
    /// Tile repeated along the floor (path under `assets/`) instead of the two colours.
    #[serde(default)]
    pub floor_tile: Option<String>,
}

/// One background layer, centred on the stage (`y` raises it).
#[derive(Clone, Debug, Deserialize)]
pub struct LayerData {
    /// Path under `assets/`.
    pub image: String,
    /// How much it follows the camera: 1 stays on screen (sky), 0 stays in the world.
    pub parallax: f32,
    pub z: f32,
    /// Height of the layer's centre in world pixels.
    #[serde(default)]
    pub y: f32,
    /// Pulses slowly (portals, fires).
    #[serde(default)]
    pub glow: bool,
    /// Fit the scenery above the floor to the camera, preserving its aspect ratio.
    #[serde(default)]
    pub fit_view: bool,
    /// Smooth photographic scenery without changing pixel-art sprite filtering.
    #[serde(default)]
    pub smooth: bool,
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
