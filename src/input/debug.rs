use bevy_egui::prelude::*;

use crate::input::FighterInput;
use bevy::prelude::*;

pub fn input_debug(mut contexts: EguiContexts, query: Query<&FighterInput>) -> Result {
    for (i, input) in query.iter().enumerate() {
        const STICK_SIZE: f32 = 50.0;
        const STICK_SIZE_FRACTION: f32 = 0.75;
        let last_frame = input.get_last_frame();
        let movement = egui::Vec2::new(last_frame.movement.x.to_num(), -last_frame.movement.y.to_num::<f32>());
        egui::Window::new(format!("Input {}", i)).show(contexts.ctx_mut()?, |ui| {
            let (_, rect) = ui.allocate_space(egui::Vec2::new(STICK_SIZE*(1.0 + STICK_SIZE_FRACTION), STICK_SIZE*(1.0 + STICK_SIZE_FRACTION)));
            let painter = ui.painter();
            let button_bg = egui::Rgba::WHITE * egui::Rgba::from_white_alpha(0.25);
            painter.circle_filled(rect.center(), STICK_SIZE * 0.5, button_bg);
            painter.circle_filled(
                rect.center() + STICK_SIZE * 0.5 * movement,
                STICK_SIZE * STICK_SIZE_FRACTION * 0.5,
                button_bg,
            );
            ui.label(format!("X: {:.2}\nY:{:.2}", last_frame.movement.x, last_frame.movement.y));
        });
    }

    Ok(())
}
