use bevy::{prelude::*, window::WindowResized};

pub(super) fn plugin(app: &mut App) {
    app.init_resource::<DrawRegion>();
    app.register_type::<DrawRegion>();
    app.register_type::<DynamicScale>();
    app.register_type::<DynamicTransform>();

    app.add_systems(
        PreUpdate,
        (
            update_draw_region,
            update_dynamic_scale,
            update_dynamic_transform,
        )
            .chain(),
    );

    #[cfg(debug_assertions)]
    app.add_systems(Update, draw_draw_region_outline);
}

/// Регион 9x16(состоит из квадратов), внутри которого происходит вся отрисовка
/// Длина и ширина его сторон определяют размер для всех сущностей
#[derive(Resource, Reflect, Default)]
#[reflect(Resource)]
pub struct DrawRegion {
    pub width: f32,
    pub height: f32,
}

fn update_draw_region(
    mut draw_region: ResMut<DrawRegion>,
    mut resize_events: EventReader<WindowResized>,
) {
    if resize_events.is_empty() {
        return;
    }

    for r_e in resize_events.read() {
        let (aspect_ratio_width, aspect_ratio_height) = (9., 16.);
        let (window_width, window_height) = (r_e.width, r_e.height);

        // При длинном окне, DrawRegion по y на весь экран
        if window_height < window_width / aspect_ratio_width * aspect_ratio_height {
            draw_region.height = window_height;
            draw_region.width = draw_region.height / aspect_ratio_height * aspect_ratio_width;
        } else {
            // При высоком окне, DrawRegion по x на весь экран
            draw_region.width = window_width;
            draw_region.height = draw_region.width / aspect_ratio_width * aspect_ratio_height;
        }
    }
}

/// Компонент для регулирования размеров Sprite
/// Значение scale в компоненте Transform при размере окна игры 1920x1080
#[derive(Component, Reflect)]
#[reflect(Component)]
pub struct DynamicScale(pub f32);

fn update_dynamic_scale(
    mut dynamic_scale: Query<(&mut Transform, &DynamicScale)>,
    draw_region: Res<DrawRegion>,
) {
    for (mut transform, dynamic_scale) in &mut dynamic_scale {
        transform.scale = Vec3::splat(dynamic_scale.0) * draw_region.height / 1080.;
    }
}

/// Расположение сущности в квадратах DrawRegion
#[derive(Component, Reflect, Default)]
#[reflect(Component)]
pub struct DynamicTransform(pub f32, pub f32);

fn update_dynamic_transform(
    mut dynamic_transform: Query<(&mut Transform, &DynamicTransform)>,
    draw_region: Res<DrawRegion>,
) {
    for (mut transform, dynamic_transform) in &mut dynamic_transform {
        transform.translation.x = dynamic_transform.0 * draw_region.width / 9.;
        transform.translation.y = dynamic_transform.1 * draw_region.height / 16.;
    }
}

#[cfg(debug_assertions)]
fn draw_draw_region_outline(
    mut toggle: Local<bool>,
    keyboard: Res<ButtonInput<KeyCode>>,
    mut gizmos: Gizmos,
    draw_region: Res<DrawRegion>,
) {
    use bevy::math::vec2;

    if keyboard.just_pressed(KeyCode::F1) {
        *toggle ^= true;
    }
    if !*toggle {
        return;
    }

    gizmos
        .grid_2d(
            Isometry2d::IDENTITY,
            UVec2::new(9, 16),
            vec2(draw_region.width / 9., draw_region.height / 16.),
            Color::srgb(1., 0., 0.),
        )
        .outer_edges();
}
