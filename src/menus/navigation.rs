use std::{collections::HashSet, time::Duration};
use bevy::{camera::NormalizedRenderTarget, input_focus::{InputFocus, InputFocusVisible}, math::CompassOctant, picking::{backend::HitData, pointer::{Location, PointerId}}, prelude::*, ui::auto_directional_navigation::AutoDirectionalNavigator};

// Action state and input handling
#[derive(Debug, PartialEq, Eq, Hash)]
enum DirectionalNavigationAction {
    Up,
    Down,
    Left,
    Right,
    Select,
}

impl DirectionalNavigationAction {
    fn variants() -> Vec<Self> {
        vec![
            DirectionalNavigationAction::Up,
            DirectionalNavigationAction::Down,
            DirectionalNavigationAction::Left,
            DirectionalNavigationAction::Right,
            DirectionalNavigationAction::Select,
        ]
    }

    fn keycode(&self) -> KeyCode {
        match self {
            DirectionalNavigationAction::Up => KeyCode::ArrowUp,
            DirectionalNavigationAction::Down => KeyCode::ArrowDown,
            DirectionalNavigationAction::Left => KeyCode::ArrowLeft,
            DirectionalNavigationAction::Right => KeyCode::ArrowRight,
            DirectionalNavigationAction::Select => KeyCode::Enter,
        }
    }

    fn gamepad_button(&self) -> GamepadButton {
        match self {
            DirectionalNavigationAction::Up => GamepadButton::DPadUp,
            DirectionalNavigationAction::Down => GamepadButton::DPadDown,
            DirectionalNavigationAction::Left => GamepadButton::DPadLeft,
            DirectionalNavigationAction::Right => GamepadButton::DPadRight,
            DirectionalNavigationAction::Select => GamepadButton::South,
        }
    }
}

#[derive(Default, Resource)]
pub struct UINavigationActionState {
    pressed_actions: HashSet<DirectionalNavigationAction>,
}

#[derive(Default, Component, Clone)]
pub struct NavigationDefaultFocus;

pub fn process_inputs(
    mut action_state: ResMut<UINavigationActionState>,
    keyboard_input: Res<ButtonInput<KeyCode>>,
    gamepad_input: Query<&Gamepad>,
) {
    action_state.pressed_actions.clear();

    for action in DirectionalNavigationAction::variants() {
        if keyboard_input.just_pressed(action.keycode()) {
            action_state.pressed_actions.insert(action);
        }
    }

    for gamepad in gamepad_input.iter() {
        for action in DirectionalNavigationAction::variants() {
            if gamepad.just_pressed(action.gamepad_button()) {
                action_state.pressed_actions.insert(action);
            }
        }
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

pub fn give_default_focus(query: Query<Entity, Added<NavigationDefaultFocus>>, mut input_focus: ResMut<InputFocus>) {
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
