use crate::world::{GameState, Index, Pipe, Score, ScoreImg, WorldAssets};
use bevy::prelude::*;
use std::f32::consts::PI;

const UP_SPEED: f32 = 300.;
const FLY_TIME: f32 = 0.3;
const GRAVITY_SPEED: f32 = -150.;
const FRAME_TIME: f32 = 0.1;

pub struct Player;

#[derive(Debug, PartialEq, Default)]
pub enum MoveState {
    #[default]
    Fall,
    Fly,
}

#[derive(Component)]
#[require(Sprite)]
pub struct RedBird {
    frame_index: usize,
    frame_timer: Timer,
    animation_sheet: Vec<Handle<Image>>,
    fly_timer: Timer,
    move_state: MoveState,
    fly_sound: Handle<AudioSource>,
    death_sound: Handle<AudioSource>,
}

impl Plugin for Player {
    fn build(&self, app: &mut App) {
        app.add_systems(Startup, add_player);
        app.add_systems(
            Update,
            (exec_animation, get_move).run_if(in_state(GameState::InGame)),
        );
        app.add_systems(
            FixedUpdate,
            (handle_click, death_check, update_score).run_if(in_state(GameState::InGame)),
        );
    }
}

fn add_player(mut commands: Commands, server: Res<AssetServer>) {
    let down_flap = server.load("./sprites/redbird-downflap.png");
    let mid_flap = server.load("./sprites/redbird-midflap.png");
    let up_flap = server.load("./sprites/redbird-upflap.png");
    let fly_sound = server.load("./audio/wing.ogg");
    let death_sound = server.load("./audio/die.ogg");
    commands.spawn(RedBird {
        fly_timer: Timer::from_seconds(FLY_TIME, TimerMode::Once),
        move_state: MoveState::Fall,
        frame_index: 0,
        frame_timer: Timer::from_seconds(FRAME_TIME, TimerMode::Repeating),
        animation_sheet: vec![down_flap, mid_flap, up_flap],
        fly_sound,
        death_sound,
    });
}

fn exec_animation(time: Res<Time>, mut query: Query<(&mut RedBird, &mut Sprite)>) {
    let (mut red_bird, mut sprite) = query.get_single_mut().unwrap();
    red_bird.frame_timer.tick(time.delta());
    if red_bird.frame_timer.just_finished() {
        red_bird.frame_index = if red_bird.frame_index < 2 {
            red_bird.frame_index + 1
        } else {
            0
        };
        sprite.image = red_bird.animation_sheet[red_bird.frame_index].clone_weak();
    }
}

fn get_move(time: Res<Time>, mut query: Query<(&mut Transform, &mut RedBird)>) {
    let (mut transform, mut red_bird) = query.get_single_mut().unwrap();
    if red_bird.move_state == MoveState::Fly {
        red_bird.fly_timer.tick(time.delta());
    }
    if red_bird.fly_timer.just_finished() {
        red_bird.move_state = MoveState::Fall;
    }
    let (speed, rotation) = match red_bird.move_state {
        MoveState::Fall => (GRAVITY_SPEED, -0.25 * PI),
        MoveState::Fly => (UP_SPEED, 0.),
    };
    transform.translation.y += time.delta_secs() * speed;
    transform.rotation = Quat::from_rotation_z(rotation);
}

fn death_check(
    mut next_state: ResMut<NextState<GameState>>,
    mut commands: Commands,
    query: Query<(&Transform, &RedBird)>,
    pipes: Query<&Transform, With<Pipe>>,
) {
    let (transform, red_bird) = query.get_single().unwrap();
    if transform.translation.y > 244. || transform.translation.y < -132. {
        commands.spawn(AudioPlayer::new(red_bird.death_sound.clone_weak()));
        next_state.set(GameState::GameOver);
        return;
    }
    for pipe in &pipes {
        if (pipe.translation.x - transform.translation.x).abs() < 43.
            && (pipe.translation.y - transform.translation.y).abs() < 172.
        {
            commands.spawn(AudioPlayer::new(red_bird.death_sound.clone_weak()));
            next_state.set(GameState::GameOver);
        }
    }
}

fn handle_click(
    mut commands: Commands,
    mut audios: Query<Entity, With<AudioPlayer>>,
    mut query: Query<&mut RedBird>,
    mouse: Res<ButtonInput<MouseButton>>,
) {
    if mouse.just_pressed(MouseButton::Left) {
        let mut red_bird = query.get_single_mut().unwrap();
        red_bird.fly_timer.reset();
        red_bird.move_state = MoveState::Fly;
        for entity in &mut audios {
            commands.entity(entity).despawn();
        }
        commands.spawn(AudioPlayer::new(red_bird.fly_sound.clone_weak()));
    }
}

fn update_score(
    mut pipes: Query<(&Transform, &mut Pipe)>,
    mut query: Query<(&mut ImageNode, &ScoreImg)>,
    mut score: ResMut<Score>,
    world_assets: Res<WorldAssets>,
) {
    for (transform, mut pipe) in &mut pipes {
        if (transform.translation.x < -72.)
            && (transform.translation.y >= 232.)
            && (pipe.check == false)
        {
            score.0 += 1;
            pipe.check = true;
        }
    }
    let scores_index = score.get_img_index();
    for (mut img, index) in &mut query {
        match index.0 {
            Index::Units => {
                img.image = world_assets.int_img_list[scores_index[2]].clone_weak();
            }
            Index::Tens => {
                img.image = world_assets.int_img_list[scores_index[1]].clone_weak();
            }
            Index::Hundreds => {
                img.image = world_assets.int_img_list[scores_index[0]].clone_weak();
            }
        }
    }
}
