//! A character's moves as short text for the screen before a match.

use crate::sim::{Button, CharacterData, MoveData};

/// How many characters fit on a line of the briefing screen.
pub const LINE_WIDTH: usize = 29;

/// One line of text: a button's name starts the first line of its group; the rest are indented under it.
pub struct SkillLine {
    pub text: String,
    /// The line that starts a button's moves (drawn brighter).
    pub first: bool,
}

fn label(mv: &MoveData) -> String {
    let mut tags: Vec<String> = Vec::new();
    if mv.crouching {
        tags.push("crouch".into());
    }
    if mv.air {
        tags.push("air".into());
    }
    if mv.finisher_move {
        tags.push("3rd hit".into());
    }
    if mv.cooldown > 0 {
        tags.push(format!("{}s", mv.cooldown.div_ceil(60)));
    }
    if tags.is_empty() { mv.name.clone() } else { format!("{} ({})", mv.name, tags.join(", ")) }
}

/// Moves grouped by button (Light, Heavy, Special), wrapped to `LINE_WIDTH` characters.
pub fn skill_lines(c: &CharacterData) -> Vec<SkillLine> {
    let mut lines = Vec::new();
    for (button, title) in [(Button::Light, "Light"), (Button::Heavy, "Heavy"), (Button::Special, "Special")] {
        let mut names: Vec<String> = Vec::new();
        for mv in c.moves.iter().filter(|m| m.button == button) {
            let name = label(mv);
            if !names.contains(&name) {
                names.push(name);
            }
        }
        if names.is_empty() {
            continue;
        }
        let head = format!("{title}: ");
        let indent = " ".repeat(head.len());
        let mut line = head.clone();
        let mut first = true;
        for (i, name) in names.iter().enumerate() {
            let piece = if i + 1 < names.len() { format!("{name}, ") } else { name.clone() };
            if line.trim_end().len() + piece.trim_end().len() > LINE_WIDTH && line.trim().len() > title.len() + 1 {
                lines.push(SkillLine { text: line.trim_end().to_string(), first });
                first = false;
                line = indent.clone();
            }
            line.push_str(&piece);
        }
        lines.push(SkillLine { text: line.trim_end().to_string(), first });
    }
    lines
}

#[cfg(test)]
mod tests {
    use super::*;

    /// The briefing screen has room for this many skill lines under each name.
    const ROOM: usize = 9;

    #[test]
    fn every_character_fits_the_briefing_screen() {
        let mut files: Vec<_> = std::fs::read_dir("assets/characters").unwrap().filter_map(|e| e.ok()).map(|e| e.path()).collect();
        files.retain(|p| p.extension().is_some_and(|e| e == "ron"));
        assert!(!files.is_empty());
        for path in files {
            let c = CharacterData::from_ron(&std::fs::read_to_string(&path).unwrap()).unwrap();
            let lines = skill_lines(&c);
            for l in &lines {
                println!("{:>14}| {}", c.name, l.text);
            }
            assert!(lines.len() <= ROOM, "{} needs {} lines", c.name, lines.len());
            assert!(lines.iter().all(|l| l.text.len() <= LINE_WIDTH + 12), "{} has a line too long", c.name);
        }
    }
}
