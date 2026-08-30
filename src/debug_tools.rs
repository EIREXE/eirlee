//! Local developer overlays and their controller-driven palette.

use std::collections::HashMap;

use bevy::prelude::*;
use bevy_egui::{EguiContexts, EguiPrimaryContextPass, egui};

use crate::player::Player;

pub const PLAYER_DEBUG_ROW_COUNT: usize = 6;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum FighterDebugRowKind {
    None,
    CurrentState,
    Velocity,
    Input,
    Position,
    Facing,
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum PresentationMotionMode {
    #[default]
    Interpolation,
    Extrapolation,
}

impl PresentationMotionMode {
    pub const fn label(self) -> &'static str {
        match self {
            Self::Interpolation => "Interpolation",
            Self::Extrapolation => "Extrapolation",
        }
    }

    fn cycle(&mut self) {
        *self = match self {
            Self::Interpolation => Self::Extrapolation,
            Self::Extrapolation => Self::Interpolation,
        };
    }
}

impl FighterDebugRowKind {
    const ALL: [Self; 6] = [
        Self::None,
        Self::CurrentState,
        Self::Velocity,
        Self::Input,
        Self::Position,
        Self::Facing,
    ];

    pub const fn label(self) -> &'static str {
        match self {
            Self::None => "None",
            Self::CurrentState => "Current state",
            Self::Velocity => "Velocity",
            Self::Input => "Input",
            Self::Position => "Position",
            Self::Facing => "Facing",
        }
    }

    fn cycle(self, direction: i8) -> Self {
        let index = Self::ALL
            .iter()
            .position(|kind| *kind == self)
            .expect("all row kinds must be listed") as i8;
        let index = (index + direction).rem_euclid(Self::ALL.len() as i8) as usize;
        Self::ALL[index]
    }
}

#[derive(Clone, Copy)]
enum DebugFlag {
    Network,
    Ecb,
    StageCollision,
    AnimationBones,
    AttackHitboxes,
    Camera,
    DrawThroughGeometry,
}

#[derive(Clone, Copy)]
enum DebugMenuEntry {
    Toggle {
        label: &'static str,
        flag: DebugFlag,
    },
    Submenu {
        label: &'static str,
        entries: &'static [DebugMenuEntry],
    },
    PlayerSlots {
        label: &'static str,
    },
    MotionSampling {
        label: &'static str,
    },
}

const FIGHTER_MENU: &[DebugMenuEntry] = &[
    DebugMenuEntry::PlayerSlots {
        label: "Player windows",
    },
    DebugMenuEntry::Toggle {
        label: "ECB",
        flag: DebugFlag::Ecb,
    },
    DebugMenuEntry::Toggle {
        label: "Animation bones",
        flag: DebugFlag::AnimationBones,
    },
    DebugMenuEntry::Toggle {
        label: "Camera",
        flag: DebugFlag::Camera,
    },
    DebugMenuEntry::Toggle {
        label: "Attack hitboxes",
        flag: DebugFlag::AttackHitboxes,
    },
];

const WORLD_MENU: &[DebugMenuEntry] = &[DebugMenuEntry::Toggle {
    label: "Stage collision",
    flag: DebugFlag::StageCollision,
}];

const DIAGNOSTICS_MENU: &[DebugMenuEntry] = &[DebugMenuEntry::Toggle {
    label: "Network stats",
    flag: DebugFlag::Network,
}];

const RENDERING_MENU: &[DebugMenuEntry] = &[
    DebugMenuEntry::MotionSampling {
        label: "Motion sampling",
    },
    DebugMenuEntry::Toggle {
        label: "Draw through geometry",
        flag: DebugFlag::DrawThroughGeometry,
    },
];

const ROOT_MENU: &[DebugMenuEntry] = &[
    DebugMenuEntry::Submenu {
        label: "Fighter",
        entries: FIGHTER_MENU,
    },
    DebugMenuEntry::Submenu {
        label: "World",
        entries: WORLD_MENU,
    },
    DebugMenuEntry::Submenu {
        label: "Diagnostics",
        entries: DIAGNOSTICS_MENU,
    },
    DebugMenuEntry::Submenu {
        label: "Rendering",
        entries: RENDERING_MENU,
    },
];

#[derive(Clone, Copy)]
enum DebugMenuPage {
    Static(&'static [DebugMenuEntry]),
    PlayerList,
    PlayerRows(usize),
}

#[derive(Resource)]
pub struct DebugSettings {
    pub menu_open: bool,
    controller: Option<Entity>,
    pages: Vec<DebugMenuPage>,
    selections: Vec<usize>,
    pub player_rows: HashMap<usize, [FighterDebugRowKind; PLAYER_DEBUG_ROW_COUNT]>,
    pub network: bool,
    pub ecb: bool,
    pub stage_collision: bool,
    pub animation_bones: bool,
    pub attack_hitboxes: bool,
    pub camera: bool,
    pub draw_through_geometry: bool,
    pub motion_sampling: PresentationMotionMode,
}

impl Default for DebugSettings {
    fn default() -> Self {
        Self {
            menu_open: false,
            controller: None,
            pages: vec![DebugMenuPage::Static(ROOT_MENU)],
            selections: vec![0],
            player_rows: HashMap::new(),
            network: false,
            ecb: false,
            stage_collision: false,
            animation_bones: false,
            attack_hitboxes: false,
            camera: false,
            draw_through_geometry: false,
            motion_sampling: PresentationMotionMode::default(),
        }
    }
}

impl DebugSettings {
    pub fn player_rows(
        &self,
        handle: usize,
    ) -> Option<&[FighterDebugRowKind; PLAYER_DEBUG_ROW_COUNT]> {
        self.player_rows.get(&handle)
    }

    fn reset_navigation(&mut self) {
        self.pages.truncate(1);
        self.selections.truncate(1);
        self.selections[0] = 0;
    }

    fn page(&self) -> DebugMenuPage {
        *self
            .pages
            .last()
            .expect("debug menu always has a root page")
    }

    fn selected(&self) -> usize {
        *self
            .selections
            .last()
            .expect("debug menu always has a root selection")
    }

    fn selected_mut(&mut self) -> &mut usize {
        self.selections
            .last_mut()
            .expect("debug menu always has a root selection")
    }

    fn push_page(&mut self, page: DebugMenuPage) {
        self.pages.push(page);
        self.selections.push(0);
    }

    fn pop_page(&mut self) -> bool {
        if self.pages.len() == 1 {
            return false;
        }
        self.pages.pop();
        self.selections.pop();
        true
    }

    fn flag(&self, flag: DebugFlag) -> bool {
        match flag {
            DebugFlag::Network => self.network,
            DebugFlag::Ecb => self.ecb,
            DebugFlag::StageCollision => self.stage_collision,
            DebugFlag::AnimationBones => self.animation_bones,
            DebugFlag::AttackHitboxes => self.attack_hitboxes,
            DebugFlag::DrawThroughGeometry => self.draw_through_geometry,
            DebugFlag::Camera => self.camera,
        }
    }

    fn toggle_flag(&mut self, flag: DebugFlag) {
        match flag {
            DebugFlag::Network => self.network = !self.network,
            DebugFlag::Ecb => self.ecb = !self.ecb,
            DebugFlag::StageCollision => self.stage_collision = !self.stage_collision,
            DebugFlag::AnimationBones => self.animation_bones = !self.animation_bones,
            DebugFlag::AttackHitboxes => self.attack_hitboxes = !self.attack_hitboxes,
            DebugFlag::DrawThroughGeometry => {
                self.draw_through_geometry = !self.draw_through_geometry
            }
            DebugFlag::Camera => self.camera = !self.camera,
        }
    }

    fn cycle_motion_sampling(&mut self) {
        self.motion_sampling.cycle();
    }

    fn rows_mut(&mut self, handle: usize) -> &mut [FighterDebugRowKind; PLAYER_DEBUG_ROW_COUNT] {
        self.player_rows
            .entry(handle)
            .or_insert([FighterDebugRowKind::None; PLAYER_DEBUG_ROW_COUNT])
    }
}

pub struct DebugToolsPlugin;

impl Plugin for DebugToolsPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<DebugSettings>()
            .add_systems(
                Update,
                (handle_palette_input, sync_gizmo_depth_bias).chain(),
            )
            .add_systems(EguiPrimaryContextPass, draw_palette);
    }
}

fn player_handles(players: &Query<&Player>) -> Vec<usize> {
    let mut handles = players
        .iter()
        .map(|player| player.handle)
        .collect::<Vec<_>>();
    handles.sort_unstable();
    handles.dedup();
    handles
}

fn page_len(page: DebugMenuPage, player_handles: &[usize]) -> usize {
    match page {
        DebugMenuPage::Static(entries) => entries.len(),
        DebugMenuPage::PlayerList => player_handles.len(),
        DebugMenuPage::PlayerRows(_) => PLAYER_DEBUG_ROW_COUNT,
    }
}

fn handle_palette_input(
    keyboard: Res<ButtonInput<KeyCode>>,
    gamepads: Query<(Entity, &Gamepad)>,
    players: Query<&Player>,
    mut settings: ResMut<DebugSettings>,
) {
    if keyboard.just_pressed(KeyCode::KeyT) {
        settings.toggle_flag(DebugFlag::DrawThroughGeometry);
    }

    if !settings.menu_open {
        for (entity, gamepad) in &gamepads {
            if gamepad.just_pressed(GamepadButton::Start) && gamepad.pressed(GamepadButton::DPadUp)
            {
                settings.menu_open = true;
                settings.controller = Some(entity);
                settings.reset_navigation();
                return;
            }
        }
        return;
    }

    let Some(controller) = settings.controller else {
        settings.menu_open = false;
        return;
    };
    let Ok((_, gamepad)) = gamepads.get(controller) else {
        settings.menu_open = false;
        settings.controller = None;
        return;
    };
    let player_handles = player_handles(&players);
    let page = settings.page();
    let page_len = page_len(page, &player_handles);

    if page_len != 0 && settings.selected() >= page_len {
        *settings.selected_mut() = 0;
    }

    if gamepad.just_pressed(GamepadButton::DPadUp) && page_len != 0 {
        let selected = settings.selected_mut();
        *selected = selected.checked_sub(1).unwrap_or(page_len - 1);
    }
    if gamepad.just_pressed(GamepadButton::DPadDown) && page_len != 0 {
        let selected = settings.selected_mut();
        *selected = (*selected + 1) % page_len;
    }
    if matches!(page, DebugMenuPage::PlayerRows(_)) {
        let direction = if gamepad.just_pressed(GamepadButton::DPadLeft) {
            Some(-1)
        } else if gamepad.just_pressed(GamepadButton::DPadRight) {
            Some(1)
        } else {
            None
        };
        if let Some(direction) = direction {
            let DebugMenuPage::PlayerRows(handle) = page else {
                unreachable!("player row navigation must have a player row page")
            };
            let selected = settings.selected();
            let rows = settings.rows_mut(handle);
            rows[selected] = rows[selected].cycle(direction);
        }
    }
    if let DebugMenuPage::Static(entries) = page
        && matches!(
            entries.get(settings.selected()),
            Some(DebugMenuEntry::MotionSampling { .. })
        )
        && (gamepad.just_pressed(GamepadButton::DPadLeft)
            || gamepad.just_pressed(GamepadButton::DPadRight))
    {
        settings.cycle_motion_sampling();
    }
    if gamepad.just_pressed(GamepadButton::South) {
        match page {
            DebugMenuPage::Static(entries) if page_len != 0 => match entries[settings.selected()] {
                DebugMenuEntry::Toggle { flag, .. } => settings.toggle_flag(flag),
                DebugMenuEntry::Submenu { entries, .. } => {
                    settings.push_page(DebugMenuPage::Static(entries));
                }
                DebugMenuEntry::PlayerSlots { .. } => settings.push_page(DebugMenuPage::PlayerList),
                DebugMenuEntry::MotionSampling { .. } => settings.cycle_motion_sampling(),
            },
            DebugMenuPage::PlayerList if page_len != 0 => {
                let handle = player_handles[settings.selected()];
                settings.rows_mut(handle);
                settings.push_page(DebugMenuPage::PlayerRows(handle));
            }
            _ => {}
        }
    }
    if gamepad.just_pressed(GamepadButton::West) && !settings.pop_page() {
        settings.menu_open = false;
        settings.controller = None;
    }
}

fn sync_gizmo_depth_bias(settings: Res<DebugSettings>, mut configs: ResMut<GizmoConfigStore>) {
    if !settings.is_changed() {
        return;
    }
    let depth_bias = if settings.draw_through_geometry {
        -1.0
    } else {
        0.0
    };
    for (_, config, _) in configs.iter_mut() {
        config.depth_bias = depth_bias;
    }
}

fn draw_palette(
    mut contexts: EguiContexts,
    settings: Res<DebugSettings>,
    players: Query<&Player>,
) -> Result {
    if !settings.menu_open {
        return Ok(());
    }

    let player_handles = player_handles(&players);
    let page = settings.page();
    let selected = settings.selected();
    egui::Window::new("Debug")
        .anchor(egui::Align2::RIGHT_TOP, egui::vec2(-12.0, 12.0))
        .resizable(false)
        .show(contexts.ctx_mut()?, |ui| {
            ui.style_mut().override_font_id = Some(egui::FontId::proportional(20.0));
            match page {
                DebugMenuPage::Static(entries) => {
                    for (index, entry) in entries.iter().enumerate() {
                        let marker = if index == selected { ">" } else { " " };
                        match entry {
                            DebugMenuEntry::Toggle { label, flag } => {
                                let state = if settings.flag(*flag) { "on" } else { "off" };
                                ui.label(format!("{marker} {label}: {state}"));
                            }
                            DebugMenuEntry::Submenu { label, .. }
                            | DebugMenuEntry::PlayerSlots { label } => {
                                ui.label(format!("{marker} {label} >"));
                            }
                            DebugMenuEntry::MotionSampling { label } => {
                                ui.label(format!(
                                    "{marker} {label}: {}",
                                    settings.motion_sampling.label()
                                ));
                            }
                        }
                    }
                }
                DebugMenuPage::PlayerList => {
                    for (index, handle) in player_handles.iter().enumerate() {
                        let marker = if index == selected { ">" } else { " " };
                        ui.label(format!("{marker} Player {} >", handle + 1));
                    }
                    if player_handles.is_empty() {
                        ui.label("No players in match");
                    }
                }
                DebugMenuPage::PlayerRows(handle) => {
                    let rows = settings
                        .player_rows(handle)
                        .copied()
                        .unwrap_or([FighterDebugRowKind::None; PLAYER_DEBUG_ROW_COUNT]);
                    ui.label(format!("Player {}", handle + 1));
                    for (index, row) in rows.iter().enumerate() {
                        let marker = if index == selected { ">" } else { " " };
                        ui.label(format!("{marker} Row {}: {}", index + 1, row.label()));
                    }
                }
            }
            ui.separator();
            let controls = if matches!(page, DebugMenuPage::PlayerRows(_)) {
                "D-pad: select/change   B: back"
            } else {
                "D-pad: select   A: enter/toggle   Left/right: change   B: back"
            };
            ui.label(controls);
        });
    Ok(())
}

pub fn fighter_info_enabled(settings: Res<DebugSettings>) -> bool {
    settings
        .player_rows
        .values()
        .any(|rows| rows.iter().any(|row| *row != FighterDebugRowKind::None))
}

pub fn network_enabled(settings: Res<DebugSettings>) -> bool {
    settings.network
}

pub fn ecb_enabled(settings: Res<DebugSettings>) -> bool {
    settings.ecb
}

pub fn stage_collision_enabled(settings: Res<DebugSettings>) -> bool {
    settings.stage_collision
}

pub fn animation_bones_enabled(settings: Res<DebugSettings>) -> bool {
    settings.animation_bones
}

pub fn attack_hitboxes_enabled(settings: Res<DebugSettings>) -> bool {
    settings.attack_hitboxes
}

pub fn camera_debug_enabled(settings: Res<DebugSettings>) -> bool {
    settings.camera
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn player_rows_default_to_none_and_wrap() {
        let mut settings = DebugSettings::default();
        let rows = settings.rows_mut(2);

        assert_eq!(*rows, [FighterDebugRowKind::None; PLAYER_DEBUG_ROW_COUNT]);
        rows[0] = rows[0].cycle(-1);
        assert_eq!(rows[0], FighterDebugRowKind::Facing);
        rows[0] = rows[0].cycle(1);
        assert_eq!(rows[0], FighterDebugRowKind::None);
    }

    #[test]
    fn nested_navigation_preserves_parent_selection() {
        let mut settings = DebugSettings::default();
        *settings.selected_mut() = 2;
        settings.push_page(DebugMenuPage::Static(DIAGNOSTICS_MENU));

        assert!(settings.pop_page());
        assert_eq!(settings.selected(), 2);
        assert!(!settings.pop_page());
    }

    #[test]
    fn player_row_configuration_is_slot_specific() {
        let mut settings = DebugSettings::default();
        settings.rows_mut(0)[0] = FighterDebugRowKind::CurrentState;

        assert_eq!(
            settings.player_rows(0).unwrap()[0],
            FighterDebugRowKind::CurrentState
        );
        assert!(settings.player_rows(1).is_none());
    }

    #[test]
    fn presentation_motion_mode_cycles_between_choices() {
        let mut mode = PresentationMotionMode::Interpolation;

        mode.cycle();
        assert_eq!(mode, PresentationMotionMode::Extrapolation);
        mode.cycle();
        assert_eq!(mode, PresentationMotionMode::Interpolation);
    }
}
