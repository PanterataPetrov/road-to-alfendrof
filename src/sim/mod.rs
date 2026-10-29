//! Pure game simulation. No Bevy, no floats, no unordered iteration: it must be deterministic
//! so it can be rolled back for online play.

pub mod data;
pub mod fighter;
pub mod input;
pub mod state;

pub use data::{AnimKind, Button, CharacterData, CurseData, MarkKind, MinionData, MoveData, ProjectileData};
pub use fighter::{Fighter, FighterState, PX};
pub use input::Input;
pub use state::{GameState, Minion, MinionMode, Phase, Projectile};
