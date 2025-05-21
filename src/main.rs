use bevy::{
    color::palettes::{
        css::RED,
        tailwind::{BLUE_600, GREEN_600, RED_600},
    },
    prelude::*,
};

fn main() {
    let mut app = App::new();

    app.add_plugins(DefaultPlugins)
        .add_systems(Startup, (setup, spawn_cubes))
        .add_systems(Update, (move_player, collider_lines, collision_restitution))
        .run();
}

#[derive(Component)]
struct Cube;

#[derive(Component)]
struct Player;

fn setup(
    mut cmds: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
    let cam = (
        Camera3d::default(),
        Transform::from_translation(Vec3::new(0.0, 150.0, 250.0)).looking_at(Vec3::ZERO, Vec3::Y),
    );

    let ground = (
        Mesh3d(meshes.add(Plane3d::default().mesh().size(200.0, 200.0))),
        MeshMaterial3d(materials.add(StandardMaterial::from_color(GREEN_600))),
    );

    let translation = Vec3::new(0.0, 0.0, 0.0);
    let rotation = Quat::from_euler(EulerRot::XYZ, -0.7, 0.2, 0.0);
    let light = (
        DirectionalLight {
            illuminance: 5000.0,
            shadows_enabled: true,
            shadow_depth_bias: 1.5,
            shadow_normal_bias: 1.0,
            ..default()
        },
        Transform {
            translation,
            rotation,
            ..default()
        },
    );

    cmds.spawn(cam);
    cmds.spawn(ground);
    cmds.spawn(light);
}

fn spawn_cubes(
    mut cmds: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
    let mut cube = |pos: Vec3, clr: Color| {
        (
            Mesh3d(meshes.add(Cuboid::new(25.0, 25.0, 25.0))),
            MeshMaterial3d(materials.add(StandardMaterial::from_color(clr))),
            Transform::from_translation(pos),
            Cube,
        )
    };

    cmds.spawn((cube(Vec3::new(40.0, 12.5, 0.0), BLUE_600.into()), Player));
    cmds.spawn(cube(Vec3::new(-40.0, 12.5, 0.0), RED_600.into()));
}

fn collider_lines(q_cube: Query<&Transform, With<Cube>>, mut gizmos: Gizmos) {
    let radius = 25.0;

    for tf in q_cube.iter() {
        let mut pos: Vec3 = tf.translation;
        pos.y = 0.1;
        let rot = Quat::from_rotation_x(std::f32::consts::PI / 2.0);
        let iso = Isometry3d::new(pos, rot);
        gizmos.circle(iso, radius, RED);
    }
}

fn move_player(
    time: Res<Time>,
    keys: Res<ButtonInput<KeyCode>>,
    mut q_player: Query<&mut Transform, With<Player>>,
) {
    let Ok(mut tf) = q_player.single_mut() else {
        return;
    };

    let mut direction = Vec3::ZERO;

    // up
    if keys.pressed(KeyCode::KeyW) {
        direction -= Vec3::Z;
    }

    // down
    if keys.pressed(KeyCode::KeyS) {
        direction += Vec3::Z;
    }

    // right
    if keys.pressed(KeyCode::KeyD) {
        direction += Vec3::X;
    }

    // left
    if keys.pressed(KeyCode::KeyA) {
        direction -= Vec3::X;
    }

    tf.translation += direction * time.delta_secs() * 50.0;
}

fn collision_restitution(
    mut q_player: Query<&mut Transform, With<Player>>,
    q_cubes: Query<&Transform, (With<Cube>, Without<Player>)>,
) {
    let Ok(mut player_tf) = q_player.single_mut() else {
        return;
    };
    let player_pos2 = Vec2::new(player_tf.translation.x, player_tf.translation.z);
    let radius = 25.0;

    for cube_tf in q_cubes.iter() {
        let cube_pos2 = Vec2::new(cube_tf.translation.x, cube_tf.translation.z);
        let delta = player_pos2 - cube_pos2;
        let dist = delta.length();

        // if they overlap...
        if dist < radius * 2.0 {
            // how far “into” the cube we are
            let penetration = radius * 2.0 - dist;
            // safe-guard zero-length
            let normal = if dist > 0.0 {
                delta / dist
            } else {
                Vec2::new(1.0, 0.0)
            };
            // push the player *out* along XZ
            player_tf.translation.x += normal.x * penetration;
            player_tf.translation.z += normal.y * penetration;
        }
    }
}
