use std::f32::consts::PI;

use bevy::{
    color::palettes::{css::RED, tailwind::*},
    prelude::*,
};

const BUCKETS: usize = 50;
const CUBE_COUNT: usize = 10000;
const BOUNDS: f32 = 400.0;

fn main() {
    let mut app = App::new();

    app.add_plugins(DefaultPlugins)
        .add_systems(Startup, (setup, spawn_walls, spawn_cubes))
        .add_systems(
            Update,
            (
                contain_in_box,
                apply_friction,
                move_player,
                integrate_velocities,
                collider_lines,
                collision,
                draw_grid,
            ),
        )
        .run();
}

#[derive(Component)]
struct Cube;

#[derive(Component)]
struct ColliderRadius(f32);

#[derive(Component)]
struct Player;

#[derive(Component, Deref, DerefMut)]
struct Velocity(Vec3);

fn spawn_walls(
    mut cmds: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut mats: ResMut<Assets<StandardMaterial>>,
) {
    // wall thickness & height:
    let thick = 1.0;
    let height = 10.0;
    let half = BOUNDS;

    let mesh = Mesh3d(meshes.add(Cuboid::new(half * 2.0 + thick * 2.0, height, thick)));
    let mesh2 = Mesh3d(meshes.add(Cuboid::new(thick, height, half * 2.0 + thick * 2.0)));
    let mat = MeshMaterial3d(mats.add(StandardMaterial::from_color(GRAY_500)));

    // +Z wall
    cmds.spawn((
        mesh.clone(),
        mat.clone(),
        Transform::from_translation(Vec3::new(0.0, height / 2.0, half + thick / 2.0)),
    ));

    // -Z wall
    cmds.spawn((
        mesh.clone(),
        mat.clone(),
        Transform::from_translation(Vec3::new(0.0, height / 2.0, -half - thick / 2.0)),
    ));

    // +X wall
    cmds.spawn((
        mesh2.clone(),
        mat.clone(),
        Transform::from_translation(Vec3::new(half + thick / 2.0, height / 2.0, 0.0)),
    ));

    // -X wall
    cmds.spawn((
        mesh2.clone(),
        mat.clone(),
        Transform::from_translation(Vec3::new(-half - thick / 2.0, height / 2.0, 0.0)),
    ));
}

fn setup(
    mut cmds: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
    let cam = (
        Camera3d::default(),
        Transform::from_translation(Vec3::new(0.0, 550.0, 650.0)).looking_at(Vec3::ZERO, Vec3::Y),
    );

    let ground = (
        Mesh3d(meshes.add(Plane3d::default().mesh().size(BOUNDS * 2.0, BOUNDS * 2.0))),
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
    let mut cube = |pos: Vec3, radius: f32, clr: Color| {
        (
            Mesh3d(meshes.add(Cuboid::new(2.0, 2.0, 2.0))),
            MeshMaterial3d(materials.add(StandardMaterial::from_color(clr))),
            Transform::from_translation(pos),
            Cube,
            ColliderRadius(radius),
            Velocity(Vec3::ZERO),
        )
    };

    let side = (CUBE_COUNT as f32).sqrt().ceil() as u32;
    let spacing = 5.0;
    let half = (side as f32 - 1.0) * spacing * 0.5;

    for idx in 0..CUBE_COUNT {
        let col = (idx as u32) % side;
        let row = (idx as u32) / side;

        let x = col as f32 * spacing - half;
        let z = row as f32 * spacing - half;

        cmds.spawn(cube(Vec3::new(x, 1.0, z), 2.0, YELLOW_500.into()));
    }

    // Player
    cmds.spawn((
        cube(Vec3::new(0.0, 1.0, 300.0), 20.0, BLUE_500.into()),
        Player,
    ));
}

fn collider_lines(q_cube: Query<(&Transform, &ColliderRadius), With<Cube>>, mut gizmos: Gizmos) {
    for (tf, radius) in q_cube.iter() {
        let mut pos: Vec3 = tf.translation;
        pos.y = 0.1;
        let rot = Quat::from_rotation_x(std::f32::consts::PI / 2.0);
        let iso = Isometry3d::new(pos, rot);
        gizmos.circle(iso, radius.0, RED);
    }
}

fn move_player(
    time: Res<Time>,
    keys: Res<ButtonInput<KeyCode>>,
    mut query: Query<&mut Velocity, With<Player>>,
) {
    if let Ok(mut vel) = query.single_mut() {
        let mut dir = Vec3::ZERO;

        // up
        if keys.pressed(KeyCode::KeyW) {
            dir -= Vec3::Z;
        }

        // down
        if keys.pressed(KeyCode::KeyS) {
            dir += Vec3::Z;
        }

        // right
        if keys.pressed(KeyCode::KeyD) {
            dir += Vec3::X;
        }

        // left
        if keys.pressed(KeyCode::KeyA) {
            dir -= Vec3::X;
        }

        **vel = dir.normalize_or_zero() * 200.0 * time.delta_secs();
    }
}

fn integrate_velocities(mut q: Query<(&Velocity, &mut Transform), With<Cube>>) {
    for (vel, mut tf) in q.iter_mut() {
        tf.translation += **vel;
    }
}

fn draw_grid(mut gizmos: Gizmos) {
    let spacing = BOUNDS * 2.0 / BUCKETS as f32;

    gizmos.grid(
        Isometry3d::from_rotation(Quat::from_rotation_x(PI / 2.0)),
        UVec2::new(BUCKETS as u32, BUCKETS as u32),
        Vec2::new(spacing, spacing),
        ORANGE_500,
    );
}

fn collision(mut q: Query<(Entity, &mut Velocity, &mut Transform, &ColliderRadius), With<Cube>>) {
    let cell_size = (2.0 * BOUNDS) / (BUCKETS as f32);

    // 1) Build empty buckets
    let mut buckets: Vec<Vec<(Entity, Vec2, f32)>> = vec![Vec::new(); BUCKETS * BUCKETS];

    // 2) Hash each cube into a bucket
    for (e, _vel, tf, rad) in q.iter() {
        let x = ((tf.translation.x + BOUNDS) / cell_size).floor() as isize;
        let z = ((tf.translation.z + BOUNDS) / cell_size).floor() as isize;
        let bx = x.clamp(0, BUCKETS as isize - 1) as usize;
        let bz = z.clamp(0, BUCKETS as isize - 1) as usize;
        let idx = bx + bz * BUCKETS;
        buckets[idx].push((e, Vec2::new(tf.translation.x, tf.translation.z), rad.0));
    }

    // 3) Collect overlapping pairs by only looking in each cell + neighbors
    let mut overlaps = Vec::new();
    for bz in 0..BUCKETS {
        for bx in 0..BUCKETS {
            let base_idx = bx + bz * BUCKETS;
            let cell = &buckets[base_idx];

            // Check within the same cell
            for i in 0..cell.len() {
                for j in (i + 1)..cell.len() {
                    let (e1, p1, r1) = &cell[i];
                    let (e2, p2, r2) = &cell[j];
                    let delta = *p2 - *p1;
                    let dist = delta.length();
                    let sum_r = r1 + r2;
                    if dist < sum_r {
                        let pen = sum_r - dist;
                        let n = if dist > 0.0 { delta / dist } else { Vec2::X };
                        overlaps.push((*e1, *e2, pen, n));
                    }
                }
            }

            // Check this cell against the 8 neighbors to catch cross‐cell collisions
            for dz in -1isize..=1 {
                for dx in -1isize..=1 {
                    if dz == 0 && dx == 0 {
                        continue;
                    }
                    let nbx = bx as isize + dx;
                    let nbz = bz as isize + dz;
                    if !(0..BUCKETS as isize).contains(&nbx)
                        || !(0..BUCKETS as isize).contains(&nbz)
                    {
                        continue;
                    }
                    let nidx = nbx as usize + nbz as usize * BUCKETS;
                    for (e1, p1, r1) in cell {
                        for (e2, p2, r2) in &buckets[nidx] {
                            // avoid double‐checks
                            if e1.index() < e2.index() {
                                let delta = *p2 - *p1;
                                let dist = delta.length();
                                let sum_r = r1 + r2;
                                if dist < sum_r {
                                    let pen = sum_r - dist;
                                    let n = if dist > 0.0 { delta / dist } else { Vec2::X };
                                    overlaps.push((*e1, *e2, pen, n));
                                }
                            }
                        }
                    }
                }
            }
        }
    }

    // 4) Resolve all found overlaps (same inelastic merge as before)
    let mass = 1.0;
    for (e1, e2, penetration, normal) in overlaps {
        if let Ok(mut pair) = q.get_many_mut([e1, e2]) {
            let slice = pair.as_mut_slice();
            let (a, b) = slice.split_at_mut(1);
            let (_, v1, tf1, _) = &mut a[0];
            let (_, v2, tf2, _) = &mut b[0];

            // separation
            tf1.translation.x -= normal.x * penetration * 0.5;
            tf1.translation.z -= normal.y * penetration * 0.5;
            tf2.translation.x += normal.x * penetration * 0.5;
            tf2.translation.z += normal.y * penetration * 0.5;

            // inelastic normal‐merge
            let vel1 = Vec2::new(v1.x, v1.z);
            let vel2 = Vec2::new(v2.x, v2.z);
            let v1n = vel1.dot(normal);
            let v2n = vel2.dot(normal);
            let v_cm = (v1n * mass + v2n * mass) / (mass + mass);

            let t1 = vel1 - normal * v1n;
            let t2 = vel2 - normal * v2n;
            let new1 = t1 + normal * v_cm;
            let new2 = t2 + normal * v_cm;

            v1.x = new1.x;
            v1.z = new1.y;
            v2.x = new2.x;
            v2.z = new2.y;
        }
    }
}

fn contain_in_box(mut q: Query<(&ColliderRadius, &mut Velocity, &mut Transform), With<Cube>>) {
    for (rad, mut vel, mut tf) in q.iter_mut() {
        let r = rad.0;
        // left wall
        if tf.translation.x < -BOUNDS + r {
            tf.translation.x = -BOUNDS + r;
            if vel.x < 0.0 {
                vel.x = 0.0;
            }
        }
        // right wall
        if tf.translation.x > BOUNDS - r {
            tf.translation.x = BOUNDS - r;
            if vel.x > 0.0 {
                vel.x = 0.0;
            }
        }
        // back wall (positive Z)
        if tf.translation.z > BOUNDS - r {
            tf.translation.z = BOUNDS - r;
            if vel.z > 0.0 {
                vel.z = 0.0;
            }
        }
        // front wall (negative Z)
        if tf.translation.z < -BOUNDS + r {
            tf.translation.z = -BOUNDS + r;
            if vel.z < 0.0 {
                vel.z = 0.0;
            }
        }
    }
}

fn apply_friction(mut q: Query<&mut Velocity, With<Cube>>) {
    let friction_factor = 1.0; // keep <= 1.0
    for mut v in q.iter_mut() {
        **v *= friction_factor;
    }
}
