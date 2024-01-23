use bevy::prelude::*;
use leafwing_input_manager::prelude::*;

mod animation;
mod camera;

use animation::{Animation, AnimationBundle, AnimationPlugin, Animations};
use camera::CameraPlugin;

fn main() {
    App::new()
        .add_plugins((
            DefaultPlugins,
            InputManagerPlugin::<PlayerInput>::default(),
            bevy_editor_pls::prelude::EditorPlugin::default(),
            CameraPlugin,
            AnimationPlugin,
        ))
        .add_systems(Startup, (spawn_map, spawn_player))
        .add_systems(
            Update,
            (
                (move_player, player_jump, player_fall, ground_detection).chain(),
                bevy::window::close_on_esc,
            ),
        )
        .run();
}

fn spawn_map(mut commands: Commands) {
    commands.spawn((
        SpriteBundle {
            transform: Transform::from_translation(Vec3::NEG_Y * 16.),
            sprite: Sprite {
                custom_size: Some(Vec2::new(200.0, 5.)),
                color: Color::WHITE,
                ..default()
            },
            ..default()
        },
        HitBox(Vec2::new(200., 5.)),
    ));
}

#[derive(Actionlike, Reflect, Clone, Hash, PartialEq, Eq, Debug)]
enum PlayerInput {
    MoveLeft,
    MoveRight,
    Jump,
}

impl PlayerInput {
    fn player_one() -> InputMap<PlayerInput> {
        let mut map = InputMap::default();

        map.insert(KeyCode::A, PlayerInput::MoveLeft);
        map.insert(KeyCode::Left, PlayerInput::MoveLeft);

        map.insert(KeyCode::D, PlayerInput::MoveRight);
        map.insert(KeyCode::Right, PlayerInput::MoveRight);

        map.insert(KeyCode::W, PlayerInput::Jump);
        map.insert(KeyCode::Up, PlayerInput::Jump);
        map.insert(KeyCode::Space, PlayerInput::Jump);

        map
    }
}

#[derive(Component)]
struct Player;

#[derive(Bundle)]
struct PlayerBundle {
    player: Player,
    input_manager: InputManagerBundle<PlayerInput>,
}

fn spawn_player(mut commands: Commands, animations: Res<Animations>) {
    let (texture_atlas, animation) = animations.get(Animation::PlayerIdle).unwrap();

    commands.spawn((
        SpriteSheetBundle {
            texture_atlas,
            sprite: TextureAtlasSprite {
                index: 0,
                ..default()
            },
            ..default()
        },
        PlayerBundle {
            player: Player,
            input_manager: InputManagerBundle {
                input_map: PlayerInput::player_one(),
                ..default()
            },
        },
        AnimationBundle::new(animation.clone()),
        Grounded(true),
        HitBox(Vec2::new(18., 32.)),
    ));
}

const MOVE_SPEED: f32 = 100.;

fn move_player(
    mut commands: Commands,
    mut player: Query<(Entity, &ActionState<PlayerInput>, &mut Transform), With<Player>>,
    time: Res<Time>,
) {
    let (entity, input, mut transform) = player.single_mut();

    if input.just_pressed(PlayerInput::Jump) {
        commands.entity(entity).insert(Jump(100.));
        return;
    }

    if input.pressed(PlayerInput::MoveLeft) {
        transform.translation.x -= MOVE_SPEED * time.delta_seconds();
    } else if input.pressed(PlayerInput::MoveRight) {
        transform.translation.x += MOVE_SPEED * time.delta_seconds();
    }
}

#[derive(Component)]
struct Jump(f32);

const FALL_SPEED: f32 = 98.;

fn player_jump(
    mut commands: Commands,
    mut player: Query<(Entity, &ActionState<PlayerInput>, &mut Transform, &mut Jump), With<Player>>,
    time: Res<Time>,
) {
    let Ok((player, input, mut transform, mut jump)) = player.get_single_mut() else {
        return;
    };

    let jump_power = (time.delta_seconds() * FALL_SPEED * 2.).min(jump.0);
    transform.translation.y += jump_power;

    jump.0 -= if input.just_pressed(PlayerInput::Jump) {
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
