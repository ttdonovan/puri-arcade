use bevy::prelude::*;
use bevy_editor_pls::prelude::*;

use std::collections::HashMap;

fn main() {
    App::new()
        .add_plugins((DefaultPlugins, EditorPlugin::default(), CameraPlugin))
        .add_systems(Startup, (spawn_map, spawn_player))
        .add_systems(
            Update,
            (
                animate_sprite,
                (move_player, player_jump, player_fall, ground_detection).chain(),
                bevy::window::close_on_esc,
            ),
        )
        .init_resource::<Animations>()
        .run();
}

struct CameraPlugin;

impl Plugin for CameraPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Startup, spawn_camera);
    }
}

fn spawn_camera(mut commands: Commands) {
    commands.spawn(Camera2dBundle::default());
}

fn spawn_map(mut commands: Commands) {
    commands.spawn((
        SpriteBundle {
            transform: Transform::from_translation(Vec3::NEG_Y * 16.),
            sprite: Sprite {
                custom_size: Some(Vec2::new(200.0, 5.)),
                color: Color::WHITE,
                ..Default::default()
            },
            ..Default::default()
        },
        HitBox(Vec2::new(200., 5.)),
    ));
}

#[derive(Component)]
struct Player;

fn spawn_player(mut commands: Commands, animations: Res<Animations>) {
    let (texture_atlas, animation) = animations.get(Animation::PlayerIdle).unwrap();

    commands.spawn((
        SpriteSheetBundle {
            texture_atlas,
            sprite: TextureAtlasSprite {
                index: 0,
                ..Default::default()
            },
            ..Default::default()
        },
        Player,
        animation,
        FrameTime(0.0),
        Grounded(true),
        HitBox(Vec2::new(18., 32.)),
    ));
}

const MOVE_SPEED: f32 = 100.;

fn move_player(
    mut commands: Commands,
    mut player: Query<(Entity, &mut Transform), With<Player>>,
    time: Res<Time>,
    input: Res<Input<KeyCode>>,
) {
    let (entity, mut transform) = player.single_mut();

    if input.any_just_pressed([KeyCode::W, KeyCode::Up, KeyCode::Space]) {
        commands.entity(entity).insert(Jump(100.));
        return;
    }

    if input.any_pressed([KeyCode::A, KeyCode::Left]) {
        transform.translation.x -= MOVE_SPEED * time.delta_seconds();
    } else if input.any_pressed([KeyCode::D, KeyCode::Right]) {
        transform.translation.x += MOVE_SPEED * time.delta_seconds();
    }
}

#[derive(Component)]
struct Jump(f32);

const FALL_SPEED: f32 = 98.;

fn player_jump(
    mut commands: Commands,
    mut player: Query<(Entity, &mut Transform, &mut Jump), With<Player>>,
    input: Res<Input<KeyCode>>,
    time: Res<Time>,
) {
    let Ok((player, mut transform, mut jump)) = player.get_single_mut() else {
        return;
    };

    let jump_power = (time.delta_seconds() * FALL_SPEED * 2.).min(jump.0);
    transform.translation.y += jump_power;

    jump.0 -= if input.any_pressed([KeyCode::W, KeyCode::Up, KeyCode::Space]) {
        jump_power
    } else {
        jump_power * 2.
    };

    if jump.0 <= 0. {
        commands.entity(player).remove::<Jump>();
    }
}

fn player_fall(
    mut player: Query<(&mut Transform, &HitBox), (With<Player>, Without<Jump>)>,
    hitboxs: Query<(&HitBox, &Transform), Without<Player>>,
    time: Res<Time>,
) {
    let Ok((mut p_offset, &p_hitbox)) = player.get_single_mut() else {
        return;
    };

    let new_pos = p_offset.translation - Vec3::Y * FALL_SPEED * time.delta_seconds();

    for (&hitbox, offset) in &hitboxs {
        if check_hit(p_hitbox, new_pos, hitbox, offset.translation) {
            return;
        }
    }

    p_offset.translation = new_pos;
}

#[derive(Component)]
struct Grounded(bool);

fn ground_detection(
    mut player: Query<(&Transform, &mut Grounded), With<Player>>,
    mut last: Local<Transform>,
) {
    let (p_offset, mut grounded) = player.single_mut();

    let current = if p_offset.translation.y == last.translation.y {
        true
    } else {
        false
    };

    if current != grounded.0 {
        grounded.0 = current;
    }

    *last = *p_offset;
}

#[derive(Component, Clone, Copy)]
struct HitBox(Vec2);

fn check_hit(hitbox: HitBox, offset: Vec3, other_hitbox: HitBox, other_offset: Vec3) -> bool {
    let h_size = hitbox.0.y / 2.;
    let w_size: f32 = hitbox.0.x / 2.;

    let oh_size = other_hitbox.0.y / 2.;
    let ow_size: f32 = other_hitbox.0.x / 2.;

    offset.x + w_size > other_offset.x - ow_size
        && offset.x - w_size < other_offset.x + ow_size
        && offset.y + h_size > other_offset.y - oh_size
        && offset.y - h_size < other_offset.y + oh_size
}

#[derive(Component, Clone, Copy)]
struct SpriteAnimation {
    len: usize,
    frame_time: f32,
}

#[derive(Component)]
struct FrameTime(f32);

fn animate_sprite(
    mut animations: Query<(&mut TextureAtlasSprite, &SpriteAnimation, &mut FrameTime)>,
    time: Res<Time>,
) {
    for (mut sprite, animation, mut frame_time) in animations.iter_mut() {
        frame_time.0 += time.delta_seconds();

        if frame_time.0 >= animation.frame_time {
            let frames = (frame_time.0 / animation.frame_time) as usize;
            sprite.index += frames;

            if sprite.index >= animation.len {
                sprite.index %= animation.len;
            }

            frame_time.0 -= animation.frame_time;
        }
    }
}

#[derive(Resource)]
struct Animations {
    map: HashMap<Animation, (Handle<TextureAtlas>, SpriteAnimation)>,
}

impl FromWorld for Animations {
    fn from_world(world: &mut World) -> Self {
        let mut map = HashMap::new();

        world.resource_scope(|world, mut texture_atlas: Mut<Assets<TextureAtlas>>| {
            let asset_server = world.resource::<AssetServer>();

            let atlas = TextureAtlas::from_grid(
                asset_server.load("puri.png"),
                Vec2::splat(32.),
                6,
                1,
                None,
                None,
            );

            map.insert(
                Animation::PlayerIdle,
                (
                    texture_atlas.add(atlas),
                    SpriteAnimation {
                        len: 6,
                        frame_time: 1. / 5.,
                    },
                ),
            );
        });

        Animations { map }
    }
}

impl Animations {
    fn get(&self, id: Animation) -> Option<(Handle<TextureAtlas>, SpriteAnimation)> {
        self.map.get(&id).cloned()
    }
}

#[derive(Hash, PartialEq, Eq)]
enum Animation {
    PlayerIdle,
}
