use bevy::prelude::*;

use std::collections::HashMap;

pub struct AnimationPlugin;

impl Plugin for AnimationPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Update, animate_sprite)
            .init_resource::<Animations>();
    }
}

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

#[derive(Component, Clone, Copy)]
pub struct SpriteAnimation {
    pub len: usize,
    pub frame_time: f32,
}

impl SpriteAnimation {
    fn new(len: usize, fps: usize) -> SpriteAnimation {
        SpriteAnimation {
            len,
            frame_time: 1. / fps as f32,
        }
    }
}

#[derive(Component)]
pub struct FrameTime(pub f32);

#[derive(Bundle)]
pub struct AnimationBundle {
    pub animation: SpriteAnimation,
    frame_time: FrameTime,
}

impl AnimationBundle {
    pub fn new(animation: SpriteAnimation) -> Self {
        AnimationBundle {
            animation,
            frame_time: FrameTime(0.),
        }
    }
}

#[derive(Resource)]
pub struct Animations {
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
                (texture_atlas.add(atlas), SpriteAnimation::new(6, 5)),
            );
        });

        Animations { map }
    }
}

impl Animations {
    pub fn get(&self, id: Animation) -> Option<(Handle<TextureAtlas>, SpriteAnimation)> {
        self.map.get(&id).cloned()
    }
}

#[derive(Hash, PartialEq, Eq)]
pub enum Animation {
    PlayerIdle,
}
