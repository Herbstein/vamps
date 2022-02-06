use std::time::Duration;

use bevy::{
    core::{Time, Timer},
    input::Input,
    math::{Vec2, Vec3},
    prelude::{
        App, Color, Commands, Component, KeyCode, OrthographicCameraBundle, Query, Res, ResMut,
        Transform, With,
    },
    sprite::{Sprite, SpriteBundle},
    DefaultPlugins,
};
use bevy_rapier2d::{
    na::Vector2,
    physics::{
        ColliderBundle, ColliderPositionSync, NoUserData, RapierConfiguration, RapierPhysicsPlugin,
        RigidBodyBundle,
    },
    prelude::{
        CoefficientCombineRule, ColliderMaterial, ColliderPositionComponent, ColliderShape,
        RigidBodyMassPropsFlags, RigidBodyType, RigidBodyVelocityComponent,
    },
};
use rand::prelude::IteratorRandom;

#[derive(Component)]
struct Player;

#[derive(Component)]
struct Monster;

#[derive(Component)]
struct Obstacle;

#[derive(Component)]
struct Projectile {
    direction: Vec3,
    lives: usize,
}

struct AttackTimer(Timer);

#[derive(Component)]
struct Health(f32);

fn main() {
    App::new()
        .add_plugins(DefaultPlugins)
        .add_plugin(RapierPhysicsPlugin::<NoUserData>::default())
        .insert_resource(AttackTimer(Timer::new(Duration::from_millis(500), true)))
        .add_startup_system(setup)
        .add_system(player_attack)
        .add_system(projectile_movement)
        .add_system(player_movement)
        .add_system(monster_movement)
        .run();
}

fn setup(mut commands: Commands, mut rapier_config: ResMut<RapierConfiguration>) {
    rapier_config.gravity = Vector2::zeros();

    commands.spawn_bundle(OrthographicCameraBundle::new_2d());

    commands
        .spawn_bundle(SpriteBundle {
            sprite: Sprite {
                color: Color::rgb(0.5, 0.5, 1.0),
                custom_size: Some(Vec2::new(50.0, 50.0)),
                ..Default::default()
            },
            ..Default::default()
        })
        .insert_bundle(RigidBodyBundle {
            body_type: RigidBodyType::KinematicVelocityBased.into(),
            ..RigidBodyBundle::default()
        })
        .insert_bundle(ColliderBundle {
            position: [0.0, 0.0].into(),
            shape: ColliderShape::cuboid(50.0 / 2.0, 50.0 / 2.0).into(),
            material: ColliderMaterial {
                friction: 0.0,
                friction_combine_rule: CoefficientCombineRule::Min,
                restitution: 0.0,
                ..Default::default()
            }
            .into(),
            ..Default::default()
        })
        .insert(ColliderPositionSync::Discrete)
        .insert(Player)
        .insert(Health(100.0));

    for pos in [Vector2::new(100.0, 215.0), Vector2::new(-100.0, 215.0)] {
        commands
            .spawn_bundle(SpriteBundle {
                sprite: Sprite {
                    color: Color::rgb(0.5, 0.5, 0.5),
                    custom_size: Some(Vec2::new(50.0, 50.0)),
                    ..Default::default()
                },
                ..Default::default()
            })
            .insert_bundle(RigidBodyBundle {
                mass_properties: RigidBodyMassPropsFlags::ROTATION_LOCKED.into(),
                ..RigidBodyBundle::default()
            })
            .insert_bundle(ColliderBundle {
                position: pos.into(),
                shape: ColliderShape::cuboid(
                    50.0 / rapier_config.scale / 2.0,
                    50.0 / rapier_config.scale / 2.0,
                )
                .into(),
                material: ColliderMaterial {
                    friction: 0.0,
                    friction_combine_rule: CoefficientCombineRule::Min,
                    restitution: 1.0,
                    ..Default::default()
                }
                .into(),
                ..Default::default()
            })
            .insert(ColliderPositionSync::Discrete)
            .insert(Monster)
            .insert(Health(20.0));
    }

    let pos = [-250.0, 250.0];
    for x in pos {
        for y in pos {
            commands
                .spawn_bundle(SpriteBundle {
                    transform: Transform {
                        translation: Vec3::new(x, y, 0.0),
                        scale: Vec3::new(50.0, 50.0, 0.0),
                        ..Default::default()
                    },
                    sprite: Sprite {
                        color: Color::rgb(0.5, 0.2, 0.2),
                        ..Default::default()
                    },
                    ..Default::default()
                })
                .insert(Obstacle);
        }
    }
}

fn player_attack(
    mut commands: Commands,
    time: Res<Time>,
    mut attack_timer: ResMut<AttackTimer>,
    player_transform_query: Query<&Transform, With<Player>>,
    monsters_transform_query: Query<&Transform, With<Monster>>,
) {
    // Attack when the timer elapses
    if attack_timer.0.tick(time.delta()).finished() {
        // Find player location
        let player_translation = player_transform_query.single().translation;

        // Find random monster in scene
        let monster_transform = monsters_transform_query
            .iter()
            .choose(&mut rand::thread_rng());

        // Only spawn a projectile if any monster is present
        if let Some(monster_transform) = monster_transform {
            let direction = (monster_transform.translation - player_translation).normalize();

            // Spawn a new projectile
            commands
                .spawn_bundle(SpriteBundle {
                    transform: Transform {
                        translation: player_translation + direction * 28.0,
                        scale: Vec3::new(10.0, 10.0, 0.0),
                        ..Default::default()
                    },
                    sprite: Sprite {
                        color: Color::rgb(0.2, 0.5, 0.2),
                        ..Default::default()
                    },
                    ..Default::default()
                })
                .insert(Projectile {
                    direction,
                    lives: 1,
                });
        }
    }
}

fn projectile_movement(mut projectile_transform_query: Query<(&mut Transform, &Projectile)>) {
    for (mut projectile_transform, projectile) in projectile_transform_query.iter_mut() {
        projectile_transform.translation += projectile.direction * 4.0;
    }
}

fn player_movement(
    keyboard_input: Res<Input<KeyCode>>,
    mut player_transform_query: Query<&mut RigidBodyVelocityComponent, With<Player>>,
) {
    let up = keyboard_input.pressed(KeyCode::W) || keyboard_input.pressed(KeyCode::Up);
    let down = keyboard_input.pressed(KeyCode::S) || keyboard_input.pressed(KeyCode::Down);
    let left = keyboard_input.pressed(KeyCode::A) || keyboard_input.pressed(KeyCode::Left);
    let right = keyboard_input.pressed(KeyCode::D) || keyboard_input.pressed(KeyCode::Right);

    let x_axis = -(left as i8) + right as i8;
    let y_axis = -(down as i8) + up as i8;

    let mut direction = Vector2::new(x_axis as f32, y_axis as f32);
    if direction != Vector2::zeros() {
        direction /= direction.magnitude();
    }

    for mut rb_vels in player_transform_query.iter_mut() {
        rb_vels.linvel = direction * 150.0;
    }
}

fn monster_movement(
    player_transform_query: Query<&ColliderPositionComponent, With<Player>>,
    mut monster_transform_query: Query<
        (&ColliderPositionComponent, &mut RigidBodyVelocityComponent),
        With<Monster>,
    >,
) {
    let player_transform = player_transform_query.single();

    for (position, mut velocity) in monster_transform_query.iter_mut() {
        let mut direction = player_transform.0.translation.vector - position.0.translation.vector;
        if direction != Vector2::zeros() {
            direction /= direction.magnitude();
        }

        velocity.linvel = direction * 150.0 * 0.35;
    }
}
