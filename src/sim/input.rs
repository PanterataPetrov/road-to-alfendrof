/// One frame of controller state for one player, as a bitflag.
/// Keyboard, CPU and network players all produce this same type.
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
    /// Hold to guard (the shield): light attacks can never get through it.
    pub const BLOCK: u16 = 1 << 7;

    pub fn has(self, flag: u16) -> bool {
        self.0 & flag != 0
    }

    /// Horizontal direction: -1 left, 0 neutral, 1 right. Opposite directions cancel.
    pub fn horizontal(self) -> i32 {
        self.has(Self::RIGHT) as i32 - self.has(Self::LEFT) as i32
    }
}
