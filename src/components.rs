use ggez;
use ncollide2d as nc;
use specs::*;
use util::*;

/// ///////////////////////////////////////////////////////////////////////
/// Components
/// ///////////////////////////////////////////////////////////////////////
#[derive(Clone, Debug, Component)]
#[storage(VecStorage)]
pub struct Motion {
    pub velocity: Vector2,
    pub acceleration: Vector2,
}

/// Objects without one won't get affected by the `Gravity` system.
#[derive(Clone, Debug, Component)]
#[storage(VecStorage)]
pub struct Mass {}

/// Just a marker that a particular entity is the player.
#[derive(Clone, Debug, Component)]
#[storage(VecStorage)]
pub struct Player;

/// NCollide collision object handle.
/// This also stores position and orientation info.
#[derive(Clone, Debug, Component)]
#[storage(VecStorage)]
pub struct Collider {
    pub object_handle: nc::world::CollisionObjectHandle,
}

/// Sprite marker.
/// Should someday say something about what sprite to draw.
#[derive(Clone, Debug, Component)]
#[storage(VecStorage)]
pub struct Sprite {
    //image: warmy::Res<resources::Image>,
}

/// Mesh
#[derive(Clone, Debug, Component)]
#[storage(VecStorage)]
pub struct Mesh {
    pub mesh: ggez::graphics::Mesh,
}
