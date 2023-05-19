#![allow(
    clippy::type_complexity,
    clippy::manual_clamp,
    clippy::module_inception
)]

use bevy::{prelude::*, window::WindowMode};
use bevy_rapier3d::prelude::{NoUserData, RapierPhysicsPlugin};
use bevy_renet::{transport::NetcodeClientPlugin, RenetClientPlugin};

mod debug;
mod voxel;

fn main() {
    let mut app = App::default();
    app.add_plugins(DefaultPlugins.set(WindowPlugin {
        primary_window: Some(Window {
            title: "vx_bevy".into(),
            mode: WindowMode::Windowed,
            ..default()
        }),
        ..default()
    }))
    .add_state::<GameState>()
    .insert_resource(AmbientLight {
        color: Color::WHITE,
        brightness: 1.0,
    })
    .add_plugin(RenetClientPlugin)
    .add_plugin(NetcodeClientPlugin)
    .add_plugin(RapierPhysicsPlugin::<NoUserData>::default())
    .add_plugin(voxel::ui::UiPlugin)
    .add_plugin(voxel::loading::LoadingHandlerPlugin)
    .add_plugin(voxel::combat::CombatPlugin)
    .add_plugin(voxel::VoxelWorldPlugin)
    .add_plugin(debug::DebugUIPlugins)
    .add_plugin(voxel::ActorPlugin)
    .add_plugin(voxel::networking::NetworkingPlugin)
    .run();
}

#[derive(Clone, Eq, PartialEq, Debug, Hash, Default, States)]
enum GameState {
    #[default]
    Loading,
    Game,
}
