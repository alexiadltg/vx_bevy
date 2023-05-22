use crate::GameState;
use bevy::prelude::*;

use self::end::{detect_player, spawn_arrow, spawn_end_portal};

use super::{networking::ControlledPlayer, Stats};

mod end;

pub struct EventsHandlerPlugin;

impl Plugin for EventsHandlerPlugin {
    fn build(&self, app: &mut App) {
        app.add_system(spawn_end_portal.in_schedule(OnEnter(GameState::Game)))
            .add_systems((
                detect_player, 
                spawn_arrow,
                add_score
            ).in_set(OnUpdate(GameState::Game)));
    }
}

#[derive(Component)]
pub struct EndPortal {}

#[derive(Component)]
pub struct Arrow {
    timer: Timer,
}

fn add_score(mut player_q: Query<&mut Stats, With<ControlledPlayer>>) {
    for mut player in player_q.iter_mut() {
        player.score += 1;
    }
}