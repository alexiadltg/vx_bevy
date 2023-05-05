use crate::debug::DebugUISet;
use crate::voxel::PlayerController;
use bevy::prelude::*;
use bevy::utils::HashMap;

use super::PlayerAnimations;

pub fn handle_player_movement(
    query: Query<(&PlayerController, &Transform)>,
    anim_query: Res<PlayerAnimations>,
    mut done: Local<bool>,
) {
    let (controller, transform) = query.single();
    let player = anim_query.1;
    let animations = anim_query.0;

    if controller.prev_xyz != transform.translation && !*done {
        player.play(animations.0[1].clone_weak()).repeat();
        *done = true;
    } else {
        *done = false;
    }
}
// TODO: handle player actions
pub fn handle_player_actions() {}
pub struct PlayerAnimationsHandlePlugin;

impl Plugin for PlayerAnimationsHandlePlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(
            (handle_player_movement, handle_player_actions)
                .chain()
                .in_base_set(CoreSet::Update)
                .after(DebugUISet::Display),
        );
    }
}
