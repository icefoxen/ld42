//! specs systems.
use specs::{self, Join};
use nalgebra as na;
use util::{Point2, Vector2};

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
            pos.0 += motion.velocity;
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
            let offset = self.position - position.0;
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
                position.0.x, position.0.y, motion.velocity.x, motion.velocity.y
            );
        }
    }
}