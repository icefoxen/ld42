use ggez;
use ggez::graphics;
use ggez_goodies::scene;
use nalgebra as na;
use ncollide2d as nc;
use specs::{self, Builder, Join};
use warmy;

use components::*;
use error::Err;
use input;
use resources;
use scenes::*;
use systems::*;
use util::*;
use world::World;

pub struct LevelScene {
    done: bool,
    kiwi: warmy::Res<resources::Image>,
    dispatcher: specs::Dispatcher<'static, 'static>,
    collided: bool,
}

impl LevelScene {
    pub fn new(ctx: &mut ggez::Context, world: &mut World) -> Result<Self, Err> {
        let done = false;
        let kiwi = world
            .assets
            .get::<_, resources::Image>(&warmy::FSKey::new("/images/kiwi.png"), ctx)
            .unwrap();

        let dispatcher = Self::register_systems();

        Self::create_player(world);
        Self::create_planet(ctx, world)?;

        Ok(LevelScene {
            done,
            kiwi,
            dispatcher,
            collided: false,
        })
    }

    fn register_systems() -> specs::Dispatcher<'static, 'static> {
        let gravity = GravitySystem {
            position: Point2::new(100.0, 100.0),
            force: 1.0,
        };
        specs::DispatcherBuilder::new()
            .with(NCollideMotionSystem {}, "sys_movement", &[])
            .with(gravity, "sys_gravity", &["sys_movement"])
            // .with(DebugPrinterSystem {}, "sys_debugprint", &[])
            .build()
    }

    fn create_player(world: &mut World) {
        // Player collision info
        let ball = nc::shape::Ball::new(10.0);
        let mut player_collide_group = nc::world::CollisionGroups::new();
        player_collide_group.set_membership(&[2]);
        let query_type = nc::world::GeometricQueryType::Contacts(0.0, 0.0);

        let player_collider = {
            let mut collide_world = world.specs_world.write_resource::<CollisionWorld>();
            let player_handle = collide_world.add(
                na::Isometry2::new(na::Vector2::new(110.0, 110.0), na::zero()),
                nc::shape::ShapeHandle::new(ball.clone()),
                player_collide_group,
                query_type,
                (),
            );

            Collider {
                object_handle: player_handle,
            }
        };

        // Make the player entity
        world
            .specs_world
            .create_entity()
            .with(Motion {
                velocity: Vector2::new(1.5, 0.0),
                acceleration: Vector2::new(0.0, 0.0),
            })
            .with(Mass {})
            .with(Sprite {})
            .with(player_collider)
            .build();
    }

    fn create_planet(ctx: &mut ggez::Context, world: &mut World) -> Result<(), Err> {
        // Planet collision info
        let ball = nc::shape::Ball::new(10.0);
        let mut terrain_collide_group = nc::world::CollisionGroups::new();
        terrain_collide_group.set_membership(&[1]);
        let query_type = nc::world::GeometricQueryType::Contacts(0.0, 0.0);

        let planet_collider = {
            let mut collide_world = world.specs_world.write_resource::<CollisionWorld>();
            let planet_handle = collide_world.add(
                na::Isometry2::new(Vector2::new(100.0, 100.0), na::zero()),
                nc::shape::ShapeHandle::new(ball.clone()),
                terrain_collide_group,
                query_type,
                (),
            );

            Collider {
                object_handle: planet_handle,
            }
        };

        // Make the world entity
        world
            .specs_world
            .create_entity()
            .with(Mesh {
                mesh: graphics::MeshBuilder::default()
                    .circle(
                        graphics::DrawMode::Fill,
                        graphics::Point2::new(0.0, 0.0),
                        10.0,
                        1.0,
                    )
                    .build(ctx)?,
            })
            .with(planet_collider)
            .build();
        Ok(())
    }
}

/// Takes a collision object handle and returns the location and orientation
/// of the object.
///
/// Mainly used for drawing, so it returns ggez's Point type rather than ncollide's.
fn collision_object_position(
    ncollide_world: &CollisionWorld,
    collider: &Collider,
) -> (graphics::Point2, f32) {
    let collision_object = ncollide_world
        .collision_object(collider.object_handle)
        .expect("Invalid collision object; was it removed from ncollide but not specs?");
    let isometry = collision_object.position();
    let annoying_new_pos =
        graphics::Point2::new(isometry.translation.vector.x, isometry.translation.vector.y);
    let annoying_new_angle = isometry.rotation.angle();
    (annoying_new_pos, annoying_new_angle)
}

impl scene::Scene<World, input::InputEvent> for LevelScene {
    fn update(&mut self, gameworld: &mut World) -> FSceneSwitch {
        self.dispatcher.dispatch(&mut gameworld.specs_world.res);

        let mut collide_world = gameworld.specs_world.write_resource::<CollisionWorld>();
        collide_world.update();
        for e in collide_world.contact_events() {
            match e {
                nc::events::ContactEvent::Started(_, _) => {
                    self.collided = true;
                }
                nc::events::ContactEvent::Stopped(_, _) => {
                    self.collided = false;
                }
            }
        }

        if self.done {
            scene::SceneSwitch::Pop
        } else {
            scene::SceneSwitch::None
        }
    }

    fn draw(&mut self, gameworld: &mut World, ctx: &mut ggez::Context) -> ggez::GameResult<()> {
        let sprite = gameworld.specs_world.read_storage::<Sprite>();
        let mesh = gameworld.specs_world.read_storage::<Mesh>();
        let collider = gameworld.specs_world.read_storage::<Collider>();
        let ncollide_world = gameworld.specs_world.read_resource::<CollisionWorld>();
        for (c, _) in (&collider, &sprite).join() {
            let (pos, angle) = collision_object_position(&*ncollide_world, c);
            graphics::draw_ex(
                ctx,
                &(self.kiwi.borrow().0),
                graphics::DrawParam {
                    dest: pos,
                    rotation: angle,
                    ..graphics::DrawParam::default()
                },
            )?;
        }

        for (c, mesh) in (&collider, &mesh).join() {
            let (pos, angle) = collision_object_position(&*ncollide_world, c);
            graphics::draw_ex(
                ctx,
                &mesh.mesh,
                graphics::DrawParam {
                    dest: pos,
                    rotation: angle,
                    ..graphics::DrawParam::default()
                },
            )?;
        }

        if self.collided {
            let t = ggez::graphics::TextCached::new("Collision")?;
            t.queue(ctx, graphics::Point2::new(100.0, 200.0), None);
        } else {
            let t = ggez::graphics::TextCached::new("No collision")?;
            t.queue(ctx, graphics::Point2::new(100.0, 200.0), None);
        }
        graphics::TextCached::draw_queued(ctx, graphics::DrawParam::default())?;
        Ok(())
    }

    fn name(&self) -> &str {
        "LevelScene"
    }

    fn input(&mut self, gameworld: &mut World, ev: input::InputEvent, _started: bool) {
        debug!("Input: {:?}", ev);
        if gameworld.input.get_button_pressed(input::Button::Menu) {
            gameworld.quit = true;
        }
    }
}
