/// One frame of controller state for one player, as a bitflag. Keyboard, CPU and network
/// players all produce it.
#[derive(Clone, Copy, Default, PartialEq, Eq, Hash, Debug)]
pub struct Input(pub u16);

impl Input {
    pub const LEFT: u16 = 1 << 0;
    pub const RIGHT: u16 = 1 << 1;
    pub const UP: u16 = 1 << 2;
    pub const DOWN: u16 = 1 << 3;
    pub const LIGHT: u16 = 1 << 4;
    pub const HEAVY: u16 = 1 << 5;
    pub const SPECIAL: u16 = 1 << 6;
    /// Hold to guard: light attacks can never get through.
    pub const BLOCK: u16 = 1 << 7;
    /// Q and E: roll left or right (on the ground or in the air).
    pub const ROLL_LEFT: u16 = 1 << 8;
    pub const ROLL_RIGHT: u16 = 1 << 9;
    /// Ready to start, on the screen before a match; the fight itself ignores it.
    pub const READY: u16 = 1 << 10;
    /// The player has the pause menu open: they are not playing (the fight ignores it; the screen says so).
    pub const AWAY: u16 = 1 << 11;
    /// The player gives up: the match is over and they lose.
    pub const SURRENDER: u16 = 1 << 12;

    pub fn has(self, flag: u16) -> bool {
        self.0 & flag != 0
    }

    /// -1 left, 0 neutral, 1 right; opposite directions cancel.
    pub fn horizontal(self) -> i32 {
        self.has(Self::RIGHT) as i32 - self.has(Self::LEFT) as i32
    }
}
