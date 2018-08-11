//! specs systems.
use specs::{self, Join};
use nalgebra as na;
use util::*;

use components::*;

pub struct MovementSystem;

impl<'a> specs::System<'a> for MovementSystem {
    type SystemData = (
        specs::WriteStorage<'a, Position>,
        specs::WriteStorage<'a, Motion>,
    );

    fn run(&mut self, (mut pos, mut motion): Self::SystemData) {
        for (pos, motion) in (&mut pos, &mut motion).join() {
            motion.velocity += motion.acceleration;
            pos.position += motion.velocity;
            motion.acceleration = na::zero();
        }
    }
}


pub struct GravitySystem {
    pub position: Point2,
    pub force: f32,
}

impl<'a> specs::System<'a> for GravitySystem {
    type SystemData = (
        specs::WriteStorage<'a, Motion>,
        specs::ReadStorage<'a, Collider>,
        specs::ReadStorage<'a, Mass>,
        specs::Read<'a, CollisionWorld, specs::shred::PanicHandler>,
    );

    fn run(&mut self, (mut motion, collider, mass, ncollide_world): Self::SystemData) {
        for (motion, collider, _mass) in (&mut motion, &collider, &mass).join() {
            let other_position = {
                let collision_obj = ncollide_world.collision_object(collider.object_handle)
                .expect("Invalid collision object; was it removed from ncollide but not specs?");
                na::Point2{ coords: collision_obj.position().translation.vector }
            };

            let offset = self.position - other_position;
            let distance = na::norm(&offset);
            // avoid punishingly small distances
            if !distance.is_nan() && distance > 0.1 {
                motion.acceleration += offset * (self.force / (distance * distance));
            } else {
                debug!("Something horrible happened in GravitySystem: distance {}", distance);
            }
        }
    }
}


pub struct NCollideMotionSystem {
}

impl<'a> specs::System<'a> for NCollideMotionSystem {
    type SystemData = (
        specs::WriteStorage<'a, Collider>,
        specs::WriteStorage<'a, Motion>,
        // Gotta use the panic handler here 'cause there is no default
        // we can provide for CollisionWorld, I guess.
        specs::Write<'a, CollisionWorld, specs::shred::PanicHandler>,
    );

    fn run(&mut self, (mut collider, mut motion, mut ncollide_world): Self::SystemData) {
        // let mut ncollide_world =
        for (collider, motion) in (&mut collider, &mut motion).join() {
            motion.velocity += motion.acceleration;
            // pos.position += motion.velocity;
            motion.acceleration = na::zero();

            let new_position = {
                let collision_obj = ncollide_world.collision_object(collider.object_handle)
                .expect("Invalid collision object; was it removed from ncollide but not specs?");
                let mut new_position = collision_obj.position().clone();
                new_position.append_translation_mut(&na::Translation::from_vector(motion.velocity));
                new_position
            };
            ncollide_world.set_position(collider.object_handle, new_position);
        }
    }
}


pub struct DebugPrinterSystem {

}



impl<'a> specs::System<'a> for DebugPrinterSystem {
    type SystemData = (
        specs::WriteStorage<'a, Motion>,
        specs::ReadStorage<'a, Position>,
    );

    fn run(&mut self, (mut motion, position): Self::SystemData) {
        for (motion, position) in (&mut motion, &position).join() {
            debug!("Object position <{},{}>, velocity <{},{}>",
                position.position.x, position.position.y, motion.velocity.x, motion.velocity.y
            );
        }
    }
}