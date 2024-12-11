use core::f32;
use std::f32::consts::PI;

use bevy::prelude::*;
use rand::Rng;

use crate::player::RedBird;

const BASE_SPEED: f32 = -85.;
const PIPE_SPEED: f32 = -85.;

#[derive(States, Debug, Default, Hash, PartialEq, Eq, Clone)]
pub enum GameState {
    #[default]
    MainMenu,
    InGame,
    GameOver,
}

#[derive(Component)]
#[require(Sprite)]
struct BaseImg;

#[derive(Resource)]
struct GameReset(Timer);

#[derive(Component)]
struct UiImg(UiImgType);

#[derive(PartialEq)]
pub enum UiImgType {
    CoverImg,
    GameOverImg,
}

#[derive(Resource)]
pub struct WorldAssets {
    pub pipe_img: Handle<Image>,
    pub int_img_list: Vec<Handle<Image>>,
}

#[derive(Resource, Default)]
pub struct Score(pub usize);

impl Score {
    pub fn get_img_index(&self) -> [usize; 3] {
        let units = self.0 % 10;
        let tens = (self.0 / 10) % 10;
        let hundreds = (self.0 / 100) % 10;
        [hundreds, tens, units]
    }
}

#[derive(Component, Default)]
#[require(Sprite)]
pub struct Pipe {
    pub check: bool,
}

#[derive(Resource)]
struct PipeTimer(Timer);

#[derive(Resource, Default)]
struct PipeUpY(f32);

#[derive(Component)]
#[require(ImageNode)]
pub struct ScoreImg(pub Index);

#[derive(PartialEq)]
pub enum Index {
    Units,
    Tens,
    Hundreds,
}

#[derive(Component)]
#[require(Node)]
struct ScoreUi;

pub struct World;

impl Plugin for World {
    fn build(&self, app: &mut App) {
        app.add_plugins(DefaultPlugins.set(WindowPlugin {
            primary_window: Some(Window {
                title: "flappy bird".to_string(),
                resolution: (288., 512.).into(),
                resizable: false,
                ..default()
            }),
            ..default()
        }));
        app.add_systems(Startup, init_world);
        app.add_systems(
            Update,
            (base_move, pipe_move).run_if(in_state(GameState::InGame)),
        );
        app.add_systems(
            FixedUpdate,
            (
                (spawn_pipe).run_if(in_state(GameState::InGame)),
                start_game.run_if(in_state(GameState::MainMenu)),
                handle_state_change.run_if(state_changed::<GameState>),
                game_reset.run_if(in_state(GameState::GameOver)),
            ),
        );
        app.init_state::<GameState>();
    }
}

fn init_world(mut commands: Commands, server: Res<AssetServer>) {
    let int_0_img = server.load("./sprites/0.png");
    let int_1_img = server.load("./sprites/1.png");
    let int_2_img = server.load("./sprites/2.png");
    let int_3_img = server.load("./sprites/3.png");
    let int_4_img = server.load("./sprites/4.png");
    let int_5_img = server.load("./sprites/5.png");
    let int_6_img = server.load("./sprites/6.png");
    let int_7_img = server.load("./sprites/7.png");
    let int_8_img = server.load("./sprites/8.png");
    let int_9_img = server.load("./sprites/9.png");
    let int_img_list: Vec<Handle<Image>> = vec![
        int_0_img, int_1_img, int_2_img, int_3_img, int_4_img, int_5_img, int_6_img, int_7_img,
        int_8_img, int_9_img,
    ];
    let pipe_img = server.load("./sprites/pipe-green.png");
    let world_assets = WorldAssets {
        pipe_img,
        int_img_list,
    };
    commands.insert_resource(world_assets);
    commands.insert_resource(PipeTimer(Timer::from_seconds(2., TimerMode::Repeating)));
    commands.insert_resource(GameReset(Timer::from_seconds(3., TimerMode::Once)));
    commands.init_resource::<Score>();
    commands.insert_resource(PipeUpY(f32::MIN));
    commands
        .spawn((
            ScoreUi,
            Node {
                height: Val::Px(36.),
                width: Val::Px(72.),
                margin: UiRect::top(Val::Px(25.)),
                justify_self: JustifySelf::Center,
                ..default()
            },
        ))
        .with_children(|builder| {
            builder.spawn(ScoreImg(Index::Hundreds));
            builder.spawn(ScoreImg(Index::Tens));
            builder.spawn(ScoreImg(Index::Units));
        });
    let cover_img = server.load("./sprites/message.png");
    let bg_img = server.load("./sprites/background-day.png");
    let base_img_left = server.load("./sprites/base.png");
    let base_img_right = base_img_left.clone_weak();
    let game_over_img = server.load("./sprites/gameover.png");
    commands.spawn(Camera2d::default());
    commands.spawn((
        UiImg(UiImgType::CoverImg),
        Sprite {
            image: cover_img,
            ..default()
        },
        Transform {
            translation: Vec3::new(0., 0., 4.),
            ..default()
        },
        Visibility::Hidden,
    ));
    commands.spawn(Sprite {
        image: bg_img,
        ..default()
    });
    commands.spawn((
        UiImg(UiImgType::GameOverImg),
        Sprite {
            image: game_over_img,
            ..default()
        },
        Visibility::Hidden,
        Transform {
            translation: Vec3::new(0., 100., 4.),
            ..default()
        },
    ));
    commands.spawn((
        BaseImg,
        Sprite {
            image: base_img_left,
            ..default()
        },
        Transform {
            translation: Vec3::new(24., -200., 2.),
            ..default()
        },
    ));
    commands.spawn((
        BaseImg,
        Sprite {
            image: base_img_right,
            ..default()
        },
        Transform {
            translation: Vec3::new(361., -200., 2.),
            ..default()
        },
    ));
}

fn start_game(mouse: Res<ButtonInput<MouseButton>>, mut next_state: ResMut<NextState<GameState>>) {
    if mouse.just_pressed(MouseButton::Left) {
        next_state.set(GameState::InGame);
    }
}

fn base_move(time: Res<Time>, mut query: Query<&mut Transform, With<BaseImg>>) {
    let mut query = query.iter_mut();
    let mut transform_1 = query.next().unwrap();
    let mut transform_2 = query.next().unwrap();
    if transform_1.translation.x < transform_2.translation.x {
        if transform_1.translation.x <= -312. {
            transform_1.translation.x = transform_2.translation.x + 336.;
        }
    } else {
        if transform_2.translation.x <= -312. {
            transform_2.translation.x = transform_1.translation.x + 336.;
        }
    }
    transform_1.translation.x += time.delta_secs() * BASE_SPEED;
    transform_2.translation.x += time.delta_secs() * BASE_SPEED;
}

fn handle_state_change(
    mut game_reset: ResMut<GameReset>,
    mut score_ui: Query<&mut Visibility, With<ScoreUi>>,
    mut score: ResMut<Score>,
    mut commands: Commands,
    pipes: Query<Entity, With<Pipe>>,
    mut red_bird: Query<&mut Transform, With<RedBird>>,
    mut query: Query<(&mut Visibility, &UiImg), Without<ScoreUi>>,
    state: Res<State<GameState>>,
) {
    let mut score_ui = score_ui.get_single_mut().unwrap();
    let mut transform = red_bird.get_single_mut().unwrap();
    match state.get() {
        GameState::MainMenu => {
            game_reset.0.reset();
            for pipe in pipes.iter() {
                commands.entity(pipe).despawn();
            }
            *score_ui = Visibility::Hidden;
            score.0 = 0;
            transform.translation = Vec3::new(-162., 100., 3.);
            for (mut visibility, ui_img) in query.iter_mut() {
                match ui_img.0 {
                    UiImgType::CoverImg => {
                        *visibility = Visibility::Visible;
                    }
                    UiImgType::GameOverImg => {
                        *visibility = Visibility::Hidden;
                    }
                }
            }
        }
        GameState::InGame => {
            *score_ui = Visibility::Visible;
            transform.translation = Vec3::new(-72., 100., 3.);
            for (mut visibility, ui_img) in query.iter_mut() {
                match ui_img.0 {
                    UiImgType::CoverImg => {
                        *visibility = Visibility::Hidden;
                    }
                    UiImgType::GameOverImg => {
                        *visibility = Visibility::Hidden;
                    }
                }
            }
        }
        GameState::GameOver => {
            for (mut visibility, ui_img) in query.iter_mut() {
                match ui_img.0 {
                    UiImgType::CoverImg => {
                        *visibility = Visibility::Hidden;
                    }
                    UiImgType::GameOverImg => {
                        *visibility = Visibility::Visible;
                    }
                }
            }
        }
    }
}

fn spawn_pipe(
    mut last_pipe_up_y: ResMut<PipeUpY>,
    time: Res<Time>,
    mut pipe_timer: ResMut<PipeTimer>,
    world_assets: Res<WorldAssets>,
    mut commands: Commands,
) {
    pipe_timer.0.tick(time.delta());
    if pipe_timer.0.just_finished() {
        let mut rng = rand::thread_rng();
        let mut pipe_up_y = rng.gen_range(232..=350) as f32;
        while (pipe_up_y - last_pipe_up_y.0).abs() < 30. {
            pipe_up_y = rng.gen_range(232..=350) as f32;
        }
        last_pipe_up_y.0 = pipe_up_y;
        commands.spawn((
            Pipe::default(),
            Sprite {
                image: world_assets.pipe_img.clone_weak(),
                ..default()
            },
            Transform {
                translation: Vec3::new(171., pipe_up_y, 1.),
                rotation: Quat::from_rotation_z(PI),
                ..default()
            },
        ));
        commands.spawn((
            Pipe::default(),
            Sprite {
                image: world_assets.pipe_img.clone_weak(),
                ..default()
            },
            Transform {
                translation: Vec3::new(171., pipe_up_y - 470., 1.),
                ..default()
            },
        ));
    }
}

fn pipe_move(
    mut commands: Commands,
    time: Res<Time>,
    mut query: Query<(Entity, &mut Transform), With<Pipe>>,
) {
    for (entity, mut transform) in &mut query {
        if transform.translation.x < -171. {
            commands.entity(entity).despawn();
        } else {
            transform.translation.x += PIPE_SPEED * time.delta_secs();
        }
    }
}

fn game_reset(
    mut next_state: ResMut<NextState<GameState>>,
    time: Res<Time>,
    mut reset_timer: ResMut<GameReset>,
) {
    reset_timer.0.tick(time.delta());
    if reset_timer.0.just_finished() {
        next_state.set(GameState::MainMenu);
    }
}
