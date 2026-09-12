use super::input::MenuInputState;
use bevy::{
    camera::NormalizedRenderTarget,
    input_focus::{InputFocus, InputFocusVisible},
    math::CompassOctant,
    picking::{
        backend::HitData,
        pointer::{Location, PointerId},
    },
    prelude::*,
    ui::auto_directional_navigation::AutoDirectionalNavigator,
};
use bevy_egui::egui::IntoAtoms;
use std::collections::HashSet;

// Action state and input handling
#[derive(Debug, PartialEq, Eq, Hash)]
enum DirectionalNavigationAction {
    Up,
    Down,
    Left,
    Right,
    Select,
}

impl DirectionalNavigationAction {}

#[derive(Default, Resource)]
pub struct UINavigationActionState {
    pressed_actions: HashSet<DirectionalNavigationAction>,
}

#[derive(Default, Resource)]
pub struct UINavigationRepeat {
    held_for: f32,
    repeating: bool,
}

#[derive(Default, Component, Clone)]
pub struct NavigationDefaultFocus;

pub fn process_inputs(
    mut action_state: ResMut<UINavigationActionState>,
    mut repeat: ResMut<UINavigationRepeat>,
    menu_inputs: Res<MenuInputState>,
    time: Res<Time>,
) {
    action_state.pressed_actions.clear();
    let input = menu_inputs.aggregate();
    let has_direction = input.movement.length() != 0.0;

    let mut dir_distances = [
        (input.movement_digital_up, input.movement.dot(Vec2::Y)),
        (input.movement_digital_down, input.movement.dot(Vec2::NEG_Y)),
        (input.movement_digital_left, input.movement.dot(Vec2::NEG_X)),
        (input.movement_digital_right, input.movement.dot(Vec2::X)),
    ];

    let dir_distances_slice = dir_distances.as_mut_slice();
    dir_distances_slice.sort_by(|a, b| a.1.total_cmp(&b.1));

    let (curr_dir, _) = dir_distances_slice.last().copied().unwrap();

    let repeat_press = if has_direction && !curr_dir.is_just_pressed() {
        repeat.held_for += time.delta_secs();
        if !repeat.repeating && repeat.held_for >= 0.3 {
            repeat.repeating = true;
            true
        } else if repeat.repeating && (repeat.held_for - 0.3) % 0.08 < time.delta_secs() {
            true
        } else {
            false
        }
    } else {
        if !has_direction {
            repeat.held_for = 0.0;
            repeat.repeating = false;
        } else if curr_dir.is_just_pressed() {
            repeat.held_for = 0.0;
            repeat.repeating = false;
        }
        false
    };
    if curr_dir.is_just_pressed() || repeat_press {
        if input.movement.y > 0.0 {
            action_state
                .pressed_actions
                .insert(DirectionalNavigationAction::Up);
        }
        if input.movement.y < 0.0 {
            action_state
                .pressed_actions
                .insert(DirectionalNavigationAction::Down);
        }
        if input.movement.x < 0.0 {
            action_state
                .pressed_actions
                .insert(DirectionalNavigationAction::Left);
        }
        if input.movement.x > 0.0 {
            action_state
                .pressed_actions
                .insert(DirectionalNavigationAction::Right);
        }
    }
    if input.accept.is_pressed() {
        action_state
            .pressed_actions
            .insert(DirectionalNavigationAction::Select);
    }
}

const FOCUSED_BORDER: Srgba = bevy::color::palettes::tailwind::BLUE_50;

pub fn highlight_focused_element(
    input_focus: Res<InputFocus>,
    input_focus_visible: Res<InputFocusVisible>,
    mut query: Query<(Entity, &mut BorderColor)>,
) {
    for (entity, mut border_color) in query.iter_mut() {
        if input_focus.get() == Some(entity) && input_focus_visible.0 {
            *border_color = BorderColor::all(FOCUSED_BORDER);
        } else {
            *border_color = BorderColor::DEFAULT;
        }
    }
}

pub fn navigate(
    action_state: Res<UINavigationActionState>,
    mut auto_directional_navigator: AutoDirectionalNavigator,
) {
    let net_east_west = action_state
        .pressed_actions
        .contains(&DirectionalNavigationAction::Right) as i8
        - action_state
            .pressed_actions
            .contains(&DirectionalNavigationAction::Left) as i8;

    let net_north_south = action_state
        .pressed_actions
        .contains(&DirectionalNavigationAction::Up) as i8
        - action_state
            .pressed_actions
            .contains(&DirectionalNavigationAction::Down) as i8;

    // Use Dir2::from_xy to convert input to direction, then convert to CompassOctant
    let maybe_direction = Dir2::from_xy(net_east_west as f32, net_north_south as f32)
        .ok()
        .map(CompassOctant::from);

    if let Some(direction) = maybe_direction {
        match auto_directional_navigator.navigate(direction) {
            Ok(_entity) => {
                // Successfully navigated
            }
            Err(e) => {
                error!("Navigation error: {}", e)
            }
        }
    }
}

pub fn give_default_focus(
    query: Query<Entity, Added<NavigationDefaultFocus>>,
    mut input_focus: ResMut<InputFocus>,
) {
    for entity in query {
        input_focus.set(entity, bevy::input_focus::FocusCause::Navigated);
    }
}

pub fn interact_with_focused_button(
    action_state: Res<UINavigationActionState>,
    input_focus: Res<InputFocus>,
    mut commands: Commands,
) {
    if action_state
        .pressed_actions
        .contains(&DirectionalNavigationAction::Select)
        && let Some(focused_entity) = input_focus.get()
    {
        commands.trigger(Pointer::new(
            PointerId::Mouse,
            Location {
                target: NormalizedRenderTarget::None {
                    width: 0,
                    height: 0,
                },
                position: Vec2::ZERO,
            },
            Press {
                button: PointerButton::Primary,
                hit: HitData {
                    camera: Entity::PLACEHOLDER,
                    depth: 0.0,
                    position: None,
                    normal: None,
                    extra: None,
                },
                count: 1,
            },
            focused_entity,
        ));
    }
}
