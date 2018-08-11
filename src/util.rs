//! Random utility functions go here.
//!
//! Every project ends up with a few.

// I wanna use ncollide2d and that means using nalgebra 0.15
// which means using a different version than is in ggez 0.4.3
use nalgebra as na;
use ncollide2d as nc;
pub type Point2 = na::Point2<f32>;
pub type Vector2 = na::Vector2<f32>;
pub type CollisionWorld = nc::world::CollisionWorld<f32,()>;