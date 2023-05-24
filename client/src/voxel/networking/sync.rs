use super::{ClientLobby, NetworkMapping};
use crate::{
    voxel::{
        animation::Animations,
        loading::MyAssets,
        networking::{ControlledPlayer, PlayerInfo},
        player::{
            bundle::{BasePlayerBundle, MyCamera3dBundle, PlayerColliderBundle, PlayerHeadBundle},
            Body,
        },
    },
    GameState,
};
use bevy::{prelude::*, utils::HashMap};

use bevy_renet::renet::{transport::NetcodeClientTransport, RenetClient};
use common::{
    ChatMessage, ClientChannel, DisplayMessage, NetworkedEntities, NonNetworkedEntities, Player,
    PlayerCommand, ServerChannel, ServerMessages,
};

fn sync_players(
    mut cmds: Commands,
    mut client: ResMut<RenetClient>,
    transport: Res<NetcodeClientTransport>,
    mut lobby: ResMut<ClientLobby>,
    mut network_mapping: ResMut<NetworkMapping>,
    _my_assets: Res<MyAssets>,
    mut queries: ParamSet<(Query<&Transform>, Query<&ControlledPlayer>)>,
    _chat_message: ResMut<ChatMessage>,
    mut display_message: ResMut<DisplayMessage>,
) {
    let client_id = transport.client_id();
    while let Some(message) = client.receive_message(ServerChannel::ServerMessages) {
        let server_message = bincode::deserialize(&message).unwrap();
        match server_message {
            ServerMessages::PlayerCreate {
                id,
                entity,
                translation,
            } => {
                println!("Player {} connected.", id);
                let mut map = HashMap::new();
                map.insert("walk".to_string(), _my_assets.player_animation_walk.clone());
                map.insert("hit".to_string(), _my_assets.player_animation_hit.clone());

                let mut client_entity = cmds.spawn((
                    BasePlayerBundle::default(),
                    Animations(map),
                    TransformBundle {
                        local: Transform::from_xyz(translation[0], translation[1], translation[2])
                            .looking_to(Vec3::Z, Vec3::Y),
                        ..default()
                    },
                ));
                if client_id == id {
                    client_entity
                        .insert(ControlledPlayer)
                        .insert(Player { id })
                        .with_children(|player| {
                            player.spawn(Body).insert(SceneBundle {
                                scene: _my_assets.player.clone(),
                                transform: Transform::IDENTITY.looking_to(Vec3::Z, Vec3::Y),
                                ..default()
                            });
                            player.spawn(PlayerColliderBundle::default());
                            player
                                .spawn(PlayerHeadBundle::default())
                                .with_children(|head| {
                                    head.spawn(MyCamera3dBundle::default());
                                });
                        });
                } else {
                    client_entity.with_children(|player| {
                        player.spawn(SceneBundle {
                            scene: _my_assets.player.clone(),
                            transform: Transform::IDENTITY.looking_to(Vec3::Z, Vec3::Y),
                            ..default()
                        });
                    });
                }
                let player_info = PlayerInfo {
                    server_entity: entity,
                    client_entity: client_entity.id(),
                };
                lobby.players.insert(id, player_info);
                network_mapping.0.insert(entity, client_entity.id());
            }
            ServerMessages::PlayerRemove { id } => {
                println!("Player {} disconnected.", id);
                if let Some(PlayerInfo {
                    server_entity,
                    client_entity,
                }) = lobby.players.remove(&id)
                {
                    cmds.entity(client_entity).despawn();
                    network_mapping.0.remove(&server_entity);
                }
            }
        }
    }

    while let Some(message) = client.receive_message(ServerChannel::Host) {
        let host = bincode::deserialize(&message).unwrap();
        if host {
            println!("I'm the host");
        } else {
            println!("I'm not the host");
        }
    }

    while let Some(message) = client.receive_message(ServerChannel::NetworkedEntities) {
        let networked_entities: NetworkedEntities = bincode::deserialize(&message).unwrap();
        for i in 0..networked_entities.entities.len() {
            if let Some(entity) = network_mapping.0.get(&networked_entities.entities[i]) {
                // if the entity is the ControlledPlayer, we don't want to apply it
                if queries.p1().get(*entity).is_err() {
                    if let Ok(current_transform) = queries.p0().get(*entity) {
                        let translation = networked_entities.translations[i].into();
                        let rotation = networked_entities.rotations[i];
                        if translation != current_transform.translation {
                            let transform = Transform {
                                rotation,
                                translation,
                                ..Default::default()
                            };
                            cmds.entity(*entity).insert(transform);
                        }
                    }
                }
            }
        }
    }
    while let Some(message) = client.receive_message(ServerChannel::NonNetworkedEntities) {
        let non_networked_entities: NonNetworkedEntities = bincode::deserialize(&message).unwrap();
        for i in 0..non_networked_entities.entity.len() {
            if let Some(entity) = network_mapping.0.get(&non_networked_entities.entity[i]) {
                if queries.p1().get(*entity).is_err() {
                    if let Ok(current_transform) = queries.p0().get(*entity) {
                        let translation = non_networked_entities.translation[i].into();
                        let rotation = non_networked_entities.rotation[i];
                        if translation != current_transform.translation {
                            let transform = Transform {
                                rotation,
                                translation,
                                ..Default::default()
                            };
                            cmds.entity(*entity).insert(transform);
                        }
                    }
                }
            }
        }
    }
    while let Some(message) = client.receive_message(ServerChannel::ChatChannel) {
        let textmess: ChatMessage = bincode::deserialize(&message).unwrap();
        println!("Received message: {:?}", textmess.message);
        display_message.message = textmess.message.clone();
    }
}

pub fn send_one_chat(
    chat_messages: ResMut<ChatMessage>,
    player_id: Query<&Player>,
    mut client: ResMut<RenetClient>,
) {
    if player_id.get_single().is_err() {
        return;
    }
    if chat_messages.message.is_empty() {
    } else {
        let message = ChatMessage {
            client_id: player_id.get_single().unwrap().id,
            message: chat_messages.message.clone(),
        };
        if !(client.is_disconnected()) {
            let message = bincode::serialize(&message).unwrap();
            client.send_message(ClientChannel::Chat, message);
        }
        println!("Sending message: {:?}", message);
    }
}

fn sync_input(
    player_input: Query<&Transform, With<ControlledPlayer>>,
    mut client: ResMut<RenetClient>,
) {
    if player_input.get_single().is_err() {
        return;
    }
    let translation = player_input.single();
    let message = bincode::serialize(&translation.translation).unwrap();
    client.send_message(ClientChannel::Input, message)
}

fn sync_rotation(body_rot: Query<&Transform, With<Body>>, mut client: ResMut<RenetClient>) {
    if body_rot.get_single().is_err() {
        return;
    }
    let rotation = body_rot.single();
    let message = bincode::serialize(&rotation.rotation).unwrap();
    client.send_message(ClientChannel::Rots, message)
}

fn sync_player_commands(
    mut player_commands: EventReader<PlayerCommand>,
    mut client: ResMut<RenetClient>,
) {
    for command in player_commands.iter() {
        let command_message = bincode::serialize(command).unwrap();
        client.send_message(ClientChannel::Command, command_message);
    }
}

fn send_text(mut client: ResMut<RenetClient>, mut chat_message: ResMut<ChatMessage>) {
    if chat_message.message.is_empty() {
        return;
    }
    let message = bincode::serialize(&(&chat_message.message, &chat_message.client_id)).unwrap();
    client.send_message(ClientChannel::Chat, message);

    //Reiniciem els valors de chat_message per a que no es repeteixin els missatges
    //Es una mica chapuzas pero ens queda nomes un dia nois.
    chat_message.message = String::new();
    chat_message.client_id = 0;
}

pub struct NetSyncPlugin;
impl Plugin for NetSyncPlugin {
    fn build(&self, app: &mut bevy::prelude::App) {
        app.add_systems(
            (
                sync_rotation,
                sync_input,
                sync_player_commands,
                sync_players,
                send_text,
                send_one_chat,
            )
                .distributive_run_if(bevy_renet::transport::client_connected)
                .in_set(OnUpdate(GameState::Game)),
        );
    }
}
