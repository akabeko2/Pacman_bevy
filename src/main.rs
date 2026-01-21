use bevy::prelude::*;
use bevy::window::PrimaryWindow; // これがないとウィンドウサイズが取れません

// --- コンポーネント定義 ---
// --- コンポーネント・リソース ---
#[derive(Component)] struct Player { life: u32 }
#[derive(Component)] struct Wall;
#[derive(Component)] struct Enemy;
#[derive(Component)] struct Food;
#[derive(Component)] struct CurrentDirection(Vec3);
#[derive(Component)] struct Collider { size: Vec2 }
#[derive(Component)] struct ScoreText;
#[derive(Resource)] struct Score(u32);
#[derive(Resource)] struct MapInfo { offset_x: f32, offset_y: f32, snap_threshold: f32,}
#[derive(Component, Deref, DerefMut)] struct AnimationTimer(Timer);

// --- 定数 ---
const SPEED: f32 = 200.0;
const TILE_SIZE: f32 = 40.0;
const CHARCTER_SIZE: f32 = 40.0;
const LEVEL_MAP: &[&str] = &[
    "WWWWWWWWWWWWWWW",
    "W.............W",
    "W...WW.WW.W.W.W",
    "W.P....E....W.W",
    "W...WWWWWWW.W.W",
    "W.............W",
    "WWWWWWWWWWWWWWW",
];

fn main() {
    App::new()
        .add_plugins(DefaultPlugins)
        .add_systems(Startup, setup)
        .add_systems(Startup, setup_ui)
        .insert_resource(Score(0))
        .add_systems(Update, (move_player, eat_food, update_score_ui, animate_pacman))
        .run();
}

// --- 初期配置 (Setup) ---
fn setup(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    mut texture_atlas_layouts: ResMut<Assets<TextureAtlasLayout>>, // レイアウト管理用
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<ColorMaterial>>,
) {
    // カメラ
    commands.spawn(Camera2d::default());



    // マップの全体のサイズを計算
    let map_height = LEVEL_MAP.len() as f32 * TILE_SIZE;
    let map_width = LEVEL_MAP[0].len() as f32 * TILE_SIZE;

    // 描画の開始位置（左上）を計算
    // Bevyの原点(0,0)は画面中央なので、マップの半分だけ左上にずらす
    let offset_x = -map_width / 2.0 + (TILE_SIZE / 2.0);
    let offset_y = map_height / 2.0 - (TILE_SIZE / 2.0);

    commands.insert_resource(MapInfo {
        offset_x,
        offset_y,
        snap_threshold: TILE_SIZE / 5.0, // 8.0px
    });

    // 行（縦）のループ
    for (row_index, row_str) in LEVEL_MAP.iter().enumerate() {
        // 列（横）のループ
        for (col_index, char) in row_str.chars().enumerate() {
            // グリッド座標 -> ピクセル座標への変換
            // xは右に増える (+), yは下にいくほど下がる (-)
            let position = Vec3::new(
                offset_x + col_index as f32 * TILE_SIZE,
                offset_y - row_index as f32 * TILE_SIZE,
                0.0,
            );
            // 文字に応じてSpawnするものを分岐
            match char {
                'W' => {
                    commands.spawn((
                        Sprite::from_color(
                            Color::srgb(0.0, 0.0, 1.0),
                            Vec2::new(TILE_SIZE, TILE_SIZE),
                        ),
                        Transform::from_translation(position),
                        Collider {
                            size: Vec2::new(TILE_SIZE, TILE_SIZE),
                        },
                        Wall,
                    ));
                }
                'P' => {
                    // 1. 画像を読み込む
                    let texture = asset_server.load("pacman.png");

                    // 2. レイアウトを作成
                    // TextureAtlasLayout::from_grid(1コマのサイズ, 横の列数, 縦の行数, パディング, オフセット)
                    let layout = TextureAtlasLayout::from_grid(UVec2::new(30, 30), 3, 1, None, None);
                    let texture_atlas_layout = texture_atlas_layouts.add(layout);
                    commands.spawn((
                        Sprite {
                                    image: texture,
                                    // ここで「アトラスを使うぞ」と指定し、初期フレーム(index: 0)を設定
                                    texture_atlas: Some(TextureAtlas {
                                        layout: texture_atlas_layout,
                                        index: 0,
                                    }),
                                    ..default()
                                },
                        Transform::from_translation(position),
                        AnimationTimer(Timer::from_seconds(0.1, TimerMode::Repeating)),
                        Player { life: 3 },
                        CurrentDirection(Vec3::ZERO),
                        Collider { size: Vec2::new(CHARCTER_SIZE, CHARCTER_SIZE) },
                    ));
                }
                'E' => {
                    commands.spawn((
                        Sprite {
                            image: asset_server.load("ghost.png"),
                            custom_size: Some(Vec2::new(CHARCTER_SIZE, CHARCTER_SIZE)),
                            ..default()
                        },
                        Transform::from_translation(position),
                        Enemy,
                    ));
                }
                // '.' (Dot/Food)
                '.' => {
                    commands.spawn((
                        Mesh2d(meshes.add(Circle::new(4.0))),
                        MeshMaterial2d(materials.add(Color::srgb(1.0, 0.8, 0.8))), // Pinkish/White dot
                        Transform::from_translation(position),
                        Food,
                    ));
                }
                _ => {}
            }
        }
    }
}

fn setup_ui(mut commands: Commands) {
    // スコアの文字を表示
    commands.spawn((
        // テキストの設定
        Text::new("Score: 0"),
        TextFont {
            font_size: 40.0,
            ..default()
        },
        TextColor(Color::WHITE),
        
        // レイアウト設定 (CSSの absolute positioning と同じ)
        Node {
            position_type: PositionType::Absolute, // 絶対配置
            bottom: Val::Px(20.0), // 下から20px
            right: Val::Px(20.0),  // 右から20px
            ..default()
        },
        
        // タグをつける (これで後から検索できる)
        ScoreText,
    ));
}

fn update_score_ui(
        score: Res<Score>,
    mut query: Query<&mut Text, With<ScoreText>>,
) {
    if score.is_changed() {
        // テキストを取り出す
        if let  Ok(mut text) = query.single_mut(){
            text.0 = format!("Score: {}", score.0);
        }
    }
}

// --- Helper Functions ---
fn check_aabb_collision(pos_a: Vec3, size_a: Vec2, pos_b: Vec3, size_b: Vec2) -> bool {
    let distance_x = (pos_a.x - pos_b.x).abs();
    let distance_y = (pos_a.y - pos_b.y).abs();
    let min_distance_x = (size_a.x / 2.0) + (size_b.x / 2.0);
    let min_distance_y = (size_a.y / 2.0) + (size_b.y / 2.0);

    distance_x < min_distance_x && distance_y < min_distance_y
}

// --- 移動ロジック (Move Player) ---
fn move_player(
    mut player_query: Query<(&mut Transform, &mut CurrentDirection, &Collider), With<Player>>,
    window_query: Query<&Window, With<PrimaryWindow>>,
    wall_query: Query<(&Transform, &Collider), (With<Wall>, Without<Player>)>,
    keyboard_input: Res<ButtonInput<KeyCode>>,
    time: Res<Time>,
    map_info: Res<MapInfo>,
) {
    if let Ok((mut transform, mut current_dir, player_collider)) = player_query.single_mut() {
        let window = window_query.single().unwrap();

        let x_limit = window.width() / 2.0;
        let y_limit = window.height() / 2.0;
        let previous_dir = current_dir.0;

        // Use Resource
        let offset_x = map_info.offset_x;
        let offset_y = map_info.offset_y;
        let snap_threshold = map_info.snap_threshold;

        let mut next_dir = Vec3::ZERO;

        if keyboard_input.pressed(KeyCode::ArrowLeft) {
            next_dir = Vec3::new(-1.0, 0.0, 0.0);
        } else if keyboard_input.pressed(KeyCode::ArrowRight) {
            next_dir = Vec3::new(1.0, 0.0, 0.0);
        } else if keyboard_input.pressed(KeyCode::ArrowUp) {
            next_dir = Vec3::new(0.0, 1.0, 0.0);
        } else if keyboard_input.pressed(KeyCode::ArrowDown) {
            next_dir = Vec3::new(0.0, -1.0, 0.0);
        }

        if next_dir != Vec3::ZERO && next_dir != previous_dir {
            // Assisted Turning (Cornering) Logic
            let current_x = transform.translation.x;
            let current_y = transform.translation.y;

            // Turning Horizontal -> Vertical ?
            if next_dir.y != 0.0 && previous_dir.x != 0.0 {
                // Snap X to nearest column
                let grid_x_index = ((current_x - offset_x) / TILE_SIZE).round();
                let nearest_x = offset_x + grid_x_index * TILE_SIZE;

                if (current_x - nearest_x).abs() < snap_threshold {
                    transform.translation.x = nearest_x;
                    current_dir.0 = next_dir;
                }
            }
            // Turning Vertical -> Horizontal ?
            else if next_dir.x != 0.0 && previous_dir.y != 0.0 {
                // Snap Y to nearest row
                // Note: y is down-negative relative to rows?
                // position.y = offset_y - row * TILE_SIZE
                // row * TILE_SIZE = offset_y - position.y
                let grid_y_index = ((offset_y - current_y) / TILE_SIZE).round();
                let nearest_y = offset_y - grid_y_index * TILE_SIZE;

                if (current_y - nearest_y).abs() < snap_threshold {
                    transform.translation.y = nearest_y;
                    current_dir.0 = next_dir;
                }
            }
            // Simple turn (180 or starting from stop)
            else {
                current_dir.0 = next_dir;
            }
        }

        if current_dir.0.length() > 0.0 {
            let target_translation =
                transform.translation + current_dir.0 * SPEED * time.delta_secs();

            let mut collision = false;
            let player_size = player_collider.size;

            for (wall_transform, wall_collider) in wall_query.iter() {
                let wall_size = wall_collider.size;
                let wall_translation = wall_transform.translation;

                if check_aabb_collision(
                    target_translation,
                    player_size,
                    wall_translation,
                    wall_size,
                ) {
                    collision = true;
                    current_dir.0 = previous_dir;
                    break;
                }
            }

            if !collision {
                transform.translation = target_translation;

                // // 画面端の判定 (Clamp)
                // transform.translation.x = transform
                //     .translation
                //     .x
                //     .clamp(-x_limit + TILE_SIZE / 2.0, x_limit - TILE_SIZE / 2.0);
                // transform.translation.y = transform
                //     .translation
                //     .y
                //     .clamp(-y_limit + TILE_SIZE / 2.0, y_limit - TILE_SIZE / 2.0);
            }
        }
    }
}

fn eat_food(
    mut commands: Commands,
    // Add Entity to the tuple to retrieve it
    food_query: Query<(Entity, &Transform), With<Food>>,
    player_query: Query<&Transform, With<Player>>,
    mut scoreboard: ResMut<Score>,
) {
    if let Ok(player_transform) = player_query.single() {
        for (food_entity, food_transform) in food_query.iter() {
            // Check distance/collision
            if player_transform
                .translation
                .distance(food_transform.translation)
                < 20.0
            {
                // Despawn the specific entity
                commands.entity(food_entity).despawn();
                // Increment score
                scoreboard.0 += 1;
            }
        }
    }
}

fn animate_pacman(
    time: Res<Time>,
    // Spriteを書き換えるので &mut Sprite
    // タイマーも進めるので &mut AnimationTimer
    mut query: Query<(&mut AnimationTimer, &mut Sprite), With<Player>>,
) {
    for (mut timer, mut sprite) in query.iter_mut() {
        // 1. タイマーを進める
        timer.0.tick(time.delta());

        // 2. 設定時間が来たらフレームを進める
        if timer.0.just_finished() {
            // sprite.texture_atlas の中身を取り出す (Optionなので)
            if let Some(atlas) = &mut sprite.texture_atlas {
                // indexを 0 -> 1 -> 2 -> 0 -> 1... と循環させる
                // ここでは全3フレームと仮定しているので % 3
                atlas.index = (atlas.index + 1) % 3;
            }
        }
    }
}
