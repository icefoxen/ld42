//! specs systems.
use nalgebra as na;
use specs::{self, Join};
use util::*;

use components::*;

pub struct GravitySystem {}

impl<'a> specs::System<'a> for GravitySystem {
    type SystemData = (
        specs::WriteStorage<'a, Motion>,
        specs::ReadStorage<'a, Gravity>,
        specs::ReadStorage<'a, Collider>,
        specs::ReadStorage<'a, Mass>,
        specs::Read<'a, CollisionWorld, specs::shred::PanicHandler>,
    );

    fn run(&mut self, (mut motion, gravity, collider, mass, ncollide_world): Self::SystemData) {
        // I know we'll only ever have one gravity source in the game,
        // shut up.
        let mut gravity_sources: Vec<(Point2, f32)> = Vec::new();
        for (collider, gravity) in (&collider, &gravity).join() {
            let grav_position = {
                let collision_obj = ncollide_world
                    .collision_object(collider.object_handle)
                    .expect(
                        "Invalid collision object; was it removed from ncollide but not specs?",
                    );
                Point2 {
                    coords: collision_obj.position().translation.vector,
                }
            };
            gravity_sources.push((grav_position, gravity.force));
        }

        for (motion, collider, _mass) in (&mut motion, &collider, &mass).join() {
            let other_position = {
                let collision_obj = ncollide_world
                    .collision_object(collider.object_handle)
                    .expect(
                        "Invalid collision object; was it removed from ncollide but not specs?",
                    );
                Point2 {
                    coords: collision_obj.position().translation.vector,
                }
            };

            for (grav_position, grav_force) in &gravity_sources {
                let offset = grav_position - other_position;
                let distance = na::norm(&offset);
                // avoid punishingly small distances
                if !distance.is_nan() && distance > 0.1 {
                    motion.acceleration += offset * (grav_force / (distance * distance));
                } else {
                    debug!(
                        "Something horrible happened in GravitySystem: distance {}",
                        distance
                    );
                }
            }
        }
    }
}

/// Makes the player tumble and slow down after they've
/// hit something.
pub struct PlayerTumbleSystem {}

impl<'a> specs::System<'a> for PlayerTumbleSystem {
    type SystemData = (specs::WriteStorage<'a, Player>,);

    fn run(&mut self, (mut player,): Self::SystemData) {
        for (player,) in (&mut player,).join() {
            if player.tumbling_timer > 0.0 {
                println!("Tumbling!");
                // BUGGO: TODO: Aieeee, time
                player.tumbling_timer -= 0.1;
                player.friction = 0.1;
            } else {
                player.friction = 0.0;
            }
        }
    }
}

pub struct NCollideMotionSystem {}

impl<'a> specs::System<'a> for NCollideMotionSystem {
    type SystemData = (
        specs::WriteStorage<'a, Collider>,
        specs::WriteStorage<'a, Motion>,
        // Gotta use the panic handler here 'cause there is no default
        // we can provide for CollisionWorld, I guess.
        specs::Write<'a, CollisionWorld, specs::shred::PanicHandler>,
    );

    fn run(&mut self, (mut collider, mut motion, mut ncollide_world): Self::SystemData) {
        for (collider, motion) in (&mut collider, &mut motion).join() {
            motion.velocity += motion.acceleration;
            motion.acceleration = na::zero();

            let new_position = {
                let collision_obj = ncollide_world
                    .collision_object(collider.object_handle)
                    .expect(
                        "Invalid collision object; was it removed from ncollide but not specs?",
                    );
                let mut new_position = collision_obj.position().clone();
                new_position.append_translation_mut(&na::Translation::from_vector(motion.velocity));
                new_position
            };
            ncollide_world.set_position(collider.object_handle, new_position);
        }
    }
}

/*
#[allow(dead_code)]
pub struct PlayerMotionSystem {}

impl<'a> specs::System<'a> for PlayerMotionSystem {
    type SystemData = (
        specs::ReadStorage<'a, Player>,
        specs::WriteStorage<'a, Collider>,
        specs::WriteStorage<'a, Motion>,
        // Gotta use the panic handler here 'cause there is no default
        // we can provide for CollisionWorld, I guess.
        specs::Write<'a, CollisionWorld, specs::shred::PanicHandler>,
    );

    fn run(&mut self, (player, mut colliders, mut motions, mut ncollide_world): Self::SystemData) {
        for (player, collider, motion) in (&player, &mut colliders, &mut motions).join() {
            let planet_collider = colliders.get(player.planet_entity);
            if player.on_ground {
                motion.velocity = na::zero();
                motion.acceleration = na::zero();
                if player.jumping {
                    motion.velocity = Vector2::new(1.0, 0.0);
                }
            }
            motion.velocity += motion.acceleration;
            motion.acceleration = na::zero();

            let new_position = {
                let collision_obj = ncollide_world
                    .collision_object(collider.object_handle)
                    .expect(
                        "Invalid collision object; was it removed from ncollide but not specs?",
                    );
                let mut new_position = collision_obj.position().clone();
                new_position.append_translation_mut(&na::Translation::from_vector(motion.velocity));
                new_position
            };
            ncollide_world.set_position(collider.object_handle, new_position);
        }
    }
}
*/

#[allow(dead_code)]
pub struct DebugPrinterSystem {}

impl<'a> specs::System<'a> for DebugPrinterSystem {
    type SystemData = (
        specs::ReadStorage<'a, Motion>,
        specs::ReadStorage<'a, Collider>,
        specs::Read<'a, CollisionWorld, specs::shred::PanicHandler>,
    );

    fn run(&mut self, (motion, collider, ncollide_world): Self::SystemData) {
        for (motion, collider) in (&motion, &collider).join() {
            let collision_obj = ncollide_world
                .collision_object(collider.object_handle)
                .expect("Invalid collision object; was it removed from ncollide but not specs?");
            let new_position = collision_obj.position().clone();
            debug!(
                "Object position {:?}, velocity <{},{}>",
                new_position, motion.velocity.x, motion.velocity.y
            );
        }
    }
}
