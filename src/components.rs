use ggez;
use ncollide2d as nc;
use specs::*;
use warmy;
use resources;
use util::*;

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

/// Just a marker that a particular entity is the player.
#[derive(Clone, Debug, Default, Component)]
#[component(NullStorage)]
pub struct Player;

/// NCollide collision object handle
#[derive(Clone, Debug, Component)]
#[component(VecStorage)]
pub struct Collider {
    pub object: nc::world::CollisionObjectHandle,
}

/// Sprite marker.
/// Should someday say something about what sprite to draw.
#[derive(Clone, Debug, Default, Component)]
#[component(NullStorage)]
pub struct Sprite {
    //image: warmy::Res<resources::Image>,
}

/// Mesh
#[derive(Clone, Debug, Component)]
#[component(NullStorage)]
pub struct Mesh {
    pub mesh: ggez::graphics::Mesh,
}