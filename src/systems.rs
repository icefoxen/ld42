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
        specs::ReadStorage<'a, Position>,
        specs::ReadStorage<'a, Mass>,
    );

    fn run(&mut self, (mut motion, position, mass): Self::SystemData) {
        for (motion, position, _mass) in (&mut motion, &position, &mass).join() {
            let offset = self.position - position.position;
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


    use ncollide2d as nc;

impl<'a> specs::System<'a> for NCollideMotionSystem {
    type SystemData = (
        specs::WriteStorage<'a, Position>,
        specs::WriteStorage<'a, Collider>,
        specs::WriteStorage<'a, Motion>,
        specs::ReadStorage<'a, Player>,
        // specs::Write<'a, nc::world::CollisionWorld<f32,()>>,
    );

    fn run(&mut self, (mut pos, mut collider, mut motion, mut ncollide_world): Self::SystemData) {
        // let mut ncollide_world =
        for (pos, collider, motion) in (&mut pos, &mut collider, &mut motion).join() {
            motion.velocity += motion.acceleration;
            pos.position += motion.velocity;
            motion.acceleration = na::zero();

            // ncollide_world.set_position(collider.object, na::Isometry::new(pos.0, na::zero()))
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