use bevy::{prelude::*, window::PrimaryWindow};

use crate::scaling::{DrawRegion, DynamicTransform};

pub(super) fn plugin(app: &mut App) {
    app.register_type::<UiInteraction>();
    app.register_type::<UiHitbox>();

    app.add_systems(Update, (update_ui_hitboxes, trigger_on_press));

    #[cfg(debug_assertions)]
    app.add_systems(Update, draw_ui_hitboxes_outline);
}

#[derive(Component, Reflect, Default)]
#[reflect(Component)]
enum UiInteraction {
    #[default]
    None,
    Hovered,
    Pressed,
}

#[derive(Component, Reflect)]
#[reflect(Component)]
#[require(UiInteraction, DynamicTransform)]
// Длина и ширина прямоугольника хитбокса в клетках DynamicTransform
pub struct UiHitbox(pub f32, pub f32);

fn update_ui_hitboxes(
    mut query: Query<(&UiHitbox, &DynamicTransform, &mut UiInteraction)>,
    window: Query<&Window, With<PrimaryWindow>>,
    draw_region: Res<DrawRegion>,
    mouse: Res<ButtonInput<MouseButton>>,
    touch: Res<Touches>,
) {
    let window = window.single();
    let mut press_pos = if let Some(mouse_pos) = window.cursor_position() {
        mouse_pos
    } else {
        let Some(touch_pos) = touch.first_pressed_position() else {
            return;
        };
        touch_pos
    };
    press_pos.x -= window.width() / 2.;
    press_pos.y -= window.height() / 2.;
    press_pos.y *= -1.;

    let cell_width = draw_region.width / 9.;
    let cell_height = draw_region.height / 16.;
    for (hitbox, transform, mut interaction) in &mut query {
        let hitbox_bottom = (transform.1 - hitbox.1 / 2.) * cell_height;
        let hitbox_top = (transform.1 + hitbox.1 / 2.) * cell_height;
        let hitbox_left = (transform.0 - hitbox.0 / 2.) * cell_width;
        let hitbox_right = (transform.0 + hitbox.0 / 2.) * cell_width;

        if hitbox_bottom <= press_pos.y
            && press_pos.y <= hitbox_top
            && hitbox_left <= press_pos.x
            && press_pos.x <= hitbox_right
        {
            *interaction = UiInteraction::Hovered;

            if mouse.just_pressed(MouseButton::Left) || touch.any_just_pressed() {
                *interaction = UiInteraction::Pressed;
            }
            continue;
        }

        *interaction = UiInteraction::None;
    }
}

#[derive(Event)]
pub struct OnPress;

fn trigger_on_press(
    interaction_query: Query<(Entity, &UiInteraction)>,
    mut commands: Commands,
) {
    for (entity, interaction) in &interaction_query {
        if matches!(interaction, UiInteraction::Pressed) {
            commands.trigger_targets(OnPress, entity);
        }
    }
}

#[cfg(debug_assertions)]
fn draw_ui_hitboxes_outline(
    mut toggle: Local<bool>,
    keyboard: Res<ButtonInput<KeyCode>>,
    mut gizmos: Gizmos,
    draw_region: Res<DrawRegion>,
    query: Query<(&UiHitbox, &DynamicTransform)>,
) {
    use bevy::math::vec2;

    if keyboard.just_pressed(KeyCode::F3) {
        *toggle ^= true;
    }
    if !*toggle {
        return;
    }

    let cell_width = draw_region.width / 9.;
    let cell_height = draw_region.height / 16.;
    for (hitbox, transform) in &query {
        gizmos.rect_2d(
            Isometry2d::from_translation(vec2(
                transform.0 * cell_width,
                transform.1 * cell_height,
            )),
            vec2(hitbox.0 * cell_width, hitbox.1 * cell_height),
            Color::srgb(0., 0., 1.),
        );
    }
}
