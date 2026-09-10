use bevy::{
    prelude::*, text::FontSourceTemplate,
    ui::auto_directional_navigation::AutoDirectionalNavigation,
};

use crate::{AppState, menus::{MenuMarker, button::menu_button, navigation::NavigationDefaultFocus}};

#[derive(SceneComponent, Default, Clone)]
pub struct MainMenu;

impl MainMenu {
    fn scene() -> impl Scene {
        bsn! {
                BackgroundColor(Color::srgb(0.15, 0.0, 0.0))
                Children [
                    (
                        Text("Project Shinespark")
                        TextFont {
                            font: FontSourceTemplate::Handle("fonts/roboto.ttf"),
                            font_size: px(super::style::H1_SIZE),
                        }
                        TextColor(Color::srgb(0.9, 0.9, 0.9))
                    ),
                    (
                        Text("This is a beta, kuro owes me 5€")
                        TextFont {
                            font: FontSourceTemplate::Handle("fonts/roboto.ttf"),
                            font_size: px(super::style::H2_SIZE),
                        }
                        TextColor(Color::srgb(0.9, 0.9, 0.9))
                    ),
                    (
                        Node {
                            width: percent(100),
                            height: percent(100),
                            align_items: AlignItems::Center,
                            justify_content: JustifyContent::Center,
                            column_gap: px(5),
                            flex_direction: FlexDirection::Column,
                        }
                        Children [
                            (
                                menu_button("Versus")
                                on(|_event: On<Pointer<Press>>, mut next_state: ResMut<NextState<AppState>>,| next_state.set(AppState::CharacterSelect))
                                NavigationDefaultFocus
                            ),
                        ]
                    ),
                ]
        }
    }
}

pub fn setup_main_menu(mut commands: Commands) {
}
