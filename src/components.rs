//use ggez::nalgebra as na;
use ggez::graphics::*;
use specs::*;

//use util::*;

/// ///////////////////////////////////////////////////////////////////////
/// Components
/// ///////////////////////////////////////////////////////////////////////
#[derive(Clone, Debug, Component)]
#[component(VecStorage)]
pub struct Position(pub Point2);

#[derive(Clone, Debug, Component)]
#[component(VecStorage)]
pub struct Motion {
    pub velocity: Vector2,
    pub acceleration: Vector2,
}

/// Objects without one won't get affected by the `Gravity` system.
#[derive(Clone, Debug, Component)]
#[component(VecStorage)]
pub struct Mass {
}

// Just a marker that a particular entity is the player.
#[derive(Clone, Debug, Default, Component)]
#[component(NullStorage)]
pub struct Player;
