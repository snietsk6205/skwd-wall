mod card_flip;
pub mod hand;
pub mod layout;
mod layout_custom;
mod reel;
pub mod sandy;
mod snapshot;
mod transition;

pub use crate::contracts::rendering::{InstanceRaw, SandySnap, TransSnap};
pub(crate) use card_flip::{card_flip_phases, card_flip_shader_payload};
pub(crate) use reel::{cell_key, roll_in_cut, roll_out_cut};
pub use snapshot::{BackPanel, Chrome, RemovalAction, RenderSnapshot};
pub use transition::LaunchAnim;
#[cfg(test)]
pub(crate) use transition::{BACKDROP_FLAG, PROJECTED_FLAG};
pub(crate) use transition::{Transition, apply_entrance_motion};
