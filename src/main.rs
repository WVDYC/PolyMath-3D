use bevy::input::mouse::MouseMotion;
use bevy::prelude::*;
use bevy_egui::{egui, EguiContexts, EguiPlugin};

fn main() {
    App::new()
        .add_plugins(DefaultPlugins)
        .add_plugins(EguiPlugin)
        .init_resource::<EditorSettings>()
        .add_systems(Startup, setup)
        .add_systems(Update, (ui_system, apply_settings, handle_rotation, orbit_camera))
        .run();
}

#[derive(Resource)]
struct EditorSettings {
    width: f32,
    height: f32,
    depth: f32,
    cube_color: [f32; 3],
    light_pos: Vec3,
    light_color: [f32; 3],
    light_intensity: f32,
}

impl Default for EditorSettings {
    fn default() -> Self {
        Self {
            width: 1.0,
            height: 1.0,
            depth: 1.0,
            cube_color: [0.2, 0.8, 0.4],
            light_pos: Vec3::new(4.0, 5.0, 4.0),
            light_color: [1.0, 1.0, 1.0],
            light_intensity: 10_000_000.0,
        }
    }
}

#[derive(Component)]
struct InteractiveShape;

#[derive(Component)]
struct MainLight;

#[derive(Component)]
struct OrbitCamera {
    radius: f32,
    alpha: f32,
    beta: f32,
}

fn setup(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
    commands.spawn((
        PbrBundle {
            mesh: meshes.add(Cuboid::new(1.0, 1.0, 1.0)),
            material: materials.add(StandardMaterial::default()),
            transform: Transform::from_xyz(0.0, 0.0, 0.0),
            ..default()
        },
        InteractiveShape,
    ));

    commands.spawn((
        PointLightBundle {
            point_light: PointLight {
                shadows_enabled: true,
                ..default()
            },
            ..default()
        },
        MainLight,
    ));

    commands.spawn((
        Camera3dBundle {
            transform: Transform::from_xyz(0.0, 5.0, 10.0).looking_at(Vec3::ZERO, Vec3::Y),
            ..default()
        },
        OrbitCamera {
            radius: 10.0,
            alpha: 0.0,
            beta: 0.5,
        },
    ));
}

fn ui_system(mut contexts: EguiContexts, mut settings: ResMut<EditorSettings>) {
    egui::Window::new("Панель управления PolyMath 3D")
        .default_pos([10.0, 10.0])
        .show(contexts.ctx_mut(), |ui| {
            ui.collapsing("📐 Геометрия (Scale Matrix)", |ui| {
                ui.add(egui::Slider::new(&mut settings.width, 0.1..=5.0).text("Ширина"));
                ui.add(egui::Slider::new(&mut settings.height, 0.1..=5.0).text("Высота"));
                ui.add(egui::Slider::new(&mut settings.depth, 0.1..=5.0).text("Глубина"));
            });

            ui.collapsing("🎨 Цвет Куба", |ui| {
                ui.color_edit_button_rgb(&mut settings.cube_color);
            });

            ui.collapsing("💡 Настройки Света", |ui| {
                ui.label("Позиция (X, Y, Z):");
                ui.add(egui::Slider::new(&mut settings.light_pos.x, -10.0..=10.0).text("X"));
                ui.add(egui::Slider::new(&mut settings.light_pos.y, 0.0..=10.0).text("Y"));
                ui.add(egui::Slider::new(&mut settings.light_pos.z, -10.0..=10.0).text("Z"));
                ui.add_space(5.0);
                ui.label("Цвет света:");
                ui.color_edit_button_rgb(&mut settings.light_color);
                ui.add(egui::Slider::new(&mut settings.light_intensity, 0.0..=20_000_000.0).text("Яркость"));
            });

            ui.add_space(10.0);
            ui.label("WASD: Вращать объект");
            ui.label("ЛЕВАЯ КНОПКА МЫШИ: Вращать камеру");
        });
}

fn apply_settings(
    settings: Res<EditorSettings>,
    mut cube_query: Query<(&mut Transform, &Handle<StandardMaterial>), With<InteractiveShape>>,
    mut light_query: Query<(&mut Transform, &mut PointLight), (With<MainLight>, Without<InteractiveShape>)>,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
    for (mut transform, mat_handle) in &mut cube_query {
        transform.scale = Vec3::new(settings.width, settings.height, settings.depth);
        if let Some(material) = materials.get_mut(mat_handle) {
            material.base_color = Color::rgb(settings.cube_color[0], settings.cube_color[1], settings.cube_color[2]);
        }
    }

    for (mut transform, mut light) in &mut light_query {
        transform.translation = settings.light_pos;
        light.color = Color::rgb(settings.light_color[0], settings.light_color[1], settings.light_color[2]);
        light.intensity = settings.light_intensity;
    }
}

fn handle_rotation(
    keyboard_input: Res<ButtonInput<KeyCode>>,
    time: Res<Time>,
    mut query: Query<&mut Transform, With<InteractiveShape>>,
) {
    let rotation_speed = 2.5;
    let mut manual_input = false;

    for mut transform in &mut query {
        if keyboard_input.pressed(KeyCode::KeyW) || keyboard_input.pressed(KeyCode::ArrowUp) {
            transform.rotate_local_x(-rotation_speed * time.delta_seconds());
            manual_input = true;
        }
        if keyboard_input.pressed(KeyCode::KeyS) || keyboard_input.pressed(KeyCode::ArrowDown) {
            transform.rotate_local_x(rotation_speed * time.delta_seconds());
            manual_input = true;
        }
        if keyboard_input.pressed(KeyCode::KeyA) || keyboard_input.pressed(KeyCode::ArrowLeft) {
            transform.rotate_local_y(-rotation_speed * time.delta_seconds());
            manual_input = true;
        }
        if keyboard_input.pressed(KeyCode::KeyD) || keyboard_input.pressed(KeyCode::ArrowRight) {
            transform.rotate_local_y(rotation_speed * time.delta_seconds());
            manual_input = true;
        }

        if !manual_input {
            transform.rotate_local_y(0.5 * time.delta_seconds());
            transform.rotate_local_x(0.3 * time.delta_seconds());
        }
    }
}

fn orbit_camera(
    mut contexts: EguiContexts,
    mouse_buttons: Res<ButtonInput<MouseButton>>,
    mut mouse_motion_events: EventReader<MouseMotion>,
    mut query: Query<(&mut Transform, &mut OrbitCamera)>,
) {
    if contexts.ctx_mut().wants_pointer_input() || contexts.ctx_mut().is_pointer_over_area() {
        return;
    }

    let mut delta = Vec2::ZERO;
    for event in mouse_motion_events.read() {
        delta += event.delta;
    }

    if mouse_buttons.pressed(MouseButton::Left) {
        for (mut transform, mut camera) in query.iter_mut() {
            camera.alpha -= delta.x * 0.005;
            camera.beta += delta.y * 0.005;

            camera.beta = camera.beta.clamp(-1.5, 1.5);

            let x = camera.radius * camera.beta.cos() * camera.alpha.sin();
            let y = camera.radius * camera.beta.sin();
            let z = camera.radius * camera.beta.cos() * camera.alpha.cos();

            transform.translation = Vec3::new(x, y, z);
            transform.look_at(Vec3::ZERO, Vec3::Y);
        }
    }
}