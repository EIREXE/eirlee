use bevy::{input_focus::InputFocus, picking::{hover::PickingInteraction, Pickable}, prelude::*, text::FontSourceTemplate, ui::auto_directional_navigation::AutoDirectionalNavigation};

use crate::{game_settings::CommonAssets, menus::style::{FGUiButtonType, FGUiStyle}};

#[derive(Component, Default, Clone)]
#[require(Button)]
pub struct FGUiButton {
    pub button_type: FGUiButtonType
}

pub fn menu_button(label: &str) -> impl Scene {
    bsn! {
        FGUiButton
        Node {
            width: px(150),
            height: px(65),
            border: px(5),
            justify_content: JustifyContent::Center,
            align_items: AlignItems::Center,
            margin: UiRect { left: px(0), right: px(0), top: px(20), bottom: px(20) }
        }
        BoxShadow::new(Color::WHITE, px(0.0), px(0.0), px(10.0), px(10.0))
        BorderColor::from(Color::BLACK)
        BackgroundColor(Color::srgb(0.15, 0.15, 0.15))
        AutoDirectionalNavigation::default()
        Children [(
            Pickable::IGNORE
            Text(label)
            TextFont {
                font: FontSourceTemplate::Handle("fonts/roboto.ttf"),
                font_size: px(super::style::H3_SIZE),
            }
            TextColor(Color::srgb(0.9, 0.9, 0.9))
            TextShadow
        ),
        ]
    }
}

pub fn button_setup(query: Query<(Entity, &FGUiButton, &mut Node), Added<FGUiButton>>, common_assets: Res<CommonAssets>, styles: Res<Assets<FGUiStyle>>, mut commands: Commands) {
    let styles = styles.get(&common_assets.ui_style).expect("UI styles should be loaded");
    for (entity, button, mut node) in query {
        styles.button_styles[&button.button_type].normal.apply(&mut commands, entity, &mut node);
    }
}

pub fn button_style_system(query: Query<(Entity, &FGUiButton, &PickingInteraction, &mut Node), Changed<PickingInteraction>>, common_assets: Res<CommonAssets>, styles: Res<Assets<FGUiStyle>>, mut commands: Commands) {
    let styles = styles.get(&common_assets.ui_style).expect("UI styles should be loaded");
    
    for (entity, button, interaction, mut node) in query {
        let button_style_to_use = match interaction {
            PickingInteraction::Pressed => &styles.button_styles[&button.button_type].press,
            PickingInteraction::Hovered => &styles.button_styles[&button.button_type].hover,
            PickingInteraction::None => &styles.button_styles[&button.button_type].normal,
        };
        button_style_to_use.apply(&mut commands, entity, &mut node);
    }
}

pub fn button_focus_style_system(query: Query<(Entity, &FGUiButton)>, focus: Res<InputFocus>, mut commands: Commands, common_assets: Res<CommonAssets>, styles: Res<Assets<FGUiStyle>>) {
    if let Some(focused_entity) = focus.get() {
        
        if !query.contains(focused_entity) {
            return;
        }
        
        let styles = styles.get(&common_assets.ui_style).expect("UI styles should be loaded");

        for (entity, button) in query {
            if focused_entity == entity {
                let style = &styles.button_styles[&button.button_type];
                commands.entity(focused_entity).insert(Outline {
                    width: px(style.focus_outline_width_px),
                    offset: px(style.focus_outline_offset_px),
                    color: style.focus_outline_color
                });
            } else {
                commands.entity(entity).remove::<Outline>();
            }
        }
    }
}
