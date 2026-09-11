use std::{cell::RefCell, collections::HashMap, rc::Rc};

use rhai::EvalAltResult;

use crate::{
    fighter::attack::{self, AttackAngle, FighterDamage, Knockback, KnockbackType},
    math::{int::FGi32, vec::FGVec2, vec3::FGVec3},
    scripting::FighterAttackScript,
};

#[derive(Debug, Default)]
struct MoveCompiler {
    hitboxes: HashMap<u32, attack::AttackHitbox>,
    active_hitboxes: Vec<u32>,
    wont_autocancel_window: Option<(u32, u32)>,
    iasa_frame: Option<u32>,
    cursor: u32,
}

impl MoveCompiler {
    fn frame(&mut self, new_frame: u32) -> Result<(), Box<EvalAltResult>> {
        if new_frame < self.cursor {
            return Err(format!(
                "Tried to go back from frame {} -> {}, rolling back is not supported",
                self.cursor, new_frame
            )
            .into());
        }

        self.cursor = new_frame;

        Ok(())
    }

    fn remove_hitbox(&mut self, id: u32) -> Result<(), Box<EvalAltResult>> {
        let hb = self
            .hitboxes
            .get_mut(&id)
            .ok_or(format!("Can't remove hitbox with {id}: Hitbox not found"))?;

        let active_hbox_position =
            self.active_hitboxes
                .iter()
                .position(|i| *i == id)
                .ok_or(format!(
                    "Can't remove hitbox with ID {id}: Hitbox was not active"
                ))?;

        hb.end_frame = self.cursor;

        self.active_hitboxes.remove(active_hbox_position);

        Ok(())
    }
    fn remove_all_hitboxes(&mut self) -> Result<(), Box<EvalAltResult>> {
        // We are gonna modify the array, so take a copy...
        for hitbox_id in self.active_hitboxes.clone() {
            self.remove_hitbox(hitbox_id)?;
        }

        self.active_hitboxes.clear();

        Ok(())
    }

    fn set_wont_autocancel_window(
        &mut self,
        start: u32,
        end: u32,
    ) -> Result<(), Box<EvalAltResult>> {
        if end < start {
            return Err(format!(
                "Couldn't set won't autocancel window, end frame {} is smaller than start frame {}",
                end, start
            )
            .into());
        }

        self.wont_autocancel_window = Some((start, end));

        Ok(())
    }

    fn set_iasa_frame(&mut self, iasa_frame: u32) {
        self.iasa_frame = Some(iasa_frame);
    }

    fn into_attack_script(mut self) -> Result<FighterAttackScript, String> {
        self.remove_all_hitboxes().map_err(|err| err.to_string())?;
        let hitboxes = self
            .hitboxes
            .into_iter()
            .map(|(_, hitbox)| hitbox)
            .collect();
        let iasa_frame = self.iasa_frame.unwrap_or(100000);
        let wont_autocancel_window = self.wont_autocancel_window.unwrap_or((0, 100000));
        Ok(FighterAttackScript {
            hitboxes,
            iasa_frame,
            wont_autocancel_window,
        })
    }
}

// scripting functions

pub fn register_common_types(engine: &mut rhai::Engine) {
    engine
        .register_type_with_name::<KnockbackType>("KnockbackType")
        .register_fn("knockback_normal", KnockbackType::make_normal)
        .register_fn("knockback_fixed", KnockbackType::make_fixed)
        .register_type_with_name::<AttackAngle>("Angle")
        .register_fn("angle", AttackAngle::make_normal)
        .register_fn("angle_sakurai", AttackAngle::make_sakurai)
        .register_type_with_name::<Knockback>("Knockback")
        .register_fn("knockback", Knockback::lit)
        .register_type_with_name::<FGi32>("FGi32")
        .register_fn("fg32", FGi32::lit)
        .register_type_with_name::<FGVec2>("Vec2")
        .register_fn("vec2", FGVec2::lit)
        .register_type_with_name::<FGVec3>("Vec3")
        .register_fn("vec3", FGVec3::lit)
        .register_type_with_name::<FighterDamage>("FighterDamage")
        .register_fn("damage", FighterDamage::lit);
}

pub fn compile_script(text: &str) -> Result<FighterAttackScript, String> {
    let mc = Rc::new(RefCell::new(MoveCompiler {
        cursor: 0,
        ..Default::default()
    }));
    let result = {
        let mut engine = rhai::Engine::new();

        register_common_types(&mut engine);

        {
            let mc = mc.clone();
            engine.register_fn(
                "frame",
                move |frame: i64| -> Result<(), Box<EvalAltResult>> {
                    let mut b = mc.borrow_mut();
                    b.frame(frame as u32)
                },
            );
        }

        {
            let mc = mc.clone();
            engine.register_fn(
                "remove_all_hitboxes",
                move || -> Result<(), Box<EvalAltResult>> {
                    let mut b = mc.borrow_mut();
                    b.remove_all_hitboxes()
                },
            );
        }

        {
            let mc = mc.clone();
            engine.register_fn(
                "remove_all_hitboxes",
                move |id: i64| -> Result<(), Box<EvalAltResult>> {
                    let mut b = mc.borrow_mut();
                    b.remove_hitbox(id as u32)
                },
            );
        }

        {
            let mc = mc.clone();
            engine.register_fn(
                "set_wont_autocancel_window",
                move |start: i64, end: i64| -> Result<(), Box<EvalAltResult>> {
                    let mut b = mc.borrow_mut();
                    b.set_wont_autocancel_window(start as u32, end as u32)
                },
            );
        }

        {
            let mc = mc.clone();
            engine.register_fn("set_iasa_frame", move |frame: i64| {
                let mut b = mc.borrow_mut();
                b.set_iasa_frame(frame as u32);
            });
        }

        {
            let mc = mc.clone();
            engine.register_fn(
                "hitbox",
                move |id: i64,
                      bone: &str,
                      damage: FighterDamage,
                      offset: FGVec3,
                      radius: FGi32,
                      angle: AttackAngle,
                      knockback_type: KnockbackType,
                      knockback: Knockback,
                      knockback_growth: FGi32|
                      -> Result<(), Box<EvalAltResult>> {
                    let mut b = mc.borrow_mut();

                    // Rhai does not have first class u32 support, so we have to do this.
                    let id = id as u32;

                    if b.hitboxes.contains_key(&id) {
                        return Err(format!("Hitbox with ID {} already existed!", id).into());
                    }

                    let hitbox = attack::AttackHitbox {
                        id,
                        bone: bone.to_owned(),
                        damage,
                        offset,
                        radius,
                        angle,
                        knockback_type,
                        knockback,
                        knockback_growth,
                        start_frame: b.cursor,
                        end_frame: b.cursor,
                    };

                    b.hitboxes.insert(id, hitbox);
                    b.active_hitboxes.push(id);

                    Ok(())
                },
            );
        }
        engine.run(&text)
    };

    result
        .map(|_| {
            Rc::into_inner(mc)
                .unwrap()
                .into_inner()
                .into_attack_script()
        })
        .map_err(|err| err.to_string())?
}
