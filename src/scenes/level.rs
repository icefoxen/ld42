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
    sprite: warmy::Res<resources::Image>,
    dispatcher: specs::Dispatcher<'static, 'static>,
    collided: bool,
    player_entity: specs::Entity,
    planet_entity: specs::Entity,
}

impl LevelScene {
    pub fn new(ctx: &mut ggez::Context, world: &mut World) -> Result<Self, Err> {
        let done = false;
        let sprite = world
            .assets
            .get::<_, resources::Image>(&warmy::FSKey::new("/images/kiwi.png"), ctx)
            .unwrap();

        let dispatcher = Self::register_systems();

        let planet_entity = Self::create_planet(ctx, world)?;
        let player_entity = Self::create_player(ctx, world)?;

        Ok(LevelScene {
            done,
            sprite,
            dispatcher,
            player_entity,
            planet_entity,
            collided: false,
        })
    }

    fn register_systems() -> specs::Dispatcher<'static, 'static> {
        let gravity = GravitySystem {
            position: Point2::new(100.0, 100.0),
            force: 5.0,
        };
        specs::DispatcherBuilder::new()
            .with(gravity, "sys_gravity", &[])
            // .with(NCollideMotionSystem {}, "sys_motion", &[])
            // .with(DebugPrinterSystem {}, "sys_debugprint", &[])
            .build()
    }

    fn create_player(ctx: &mut ggez::Context, world: &mut World) -> Result<specs::Entity, Err> {
        // Make the player entity
        let entity = world
            .specs_world
            .create_entity()
            .with(Player {
                on_ground: false,
                jumping: false,
                walk_direction: 0.0,
                walk_force: 0.1,
            })
            .with(Motion {
                velocity: Vector2::new(1.5, 0.0),
                acceleration: Vector2::new(0.0, 0.0),
            })
            .with(Mass {})
            .with(Mesh {
                mesh: graphics::MeshBuilder::default()
                    .circle(
                        graphics::DrawMode::Line(2.0),
                        graphics::Point2::new(0.0, 0.0),
                        10.0,
                        0.5,
                    )
                    .build(ctx)?,
            })
            .build();

        // Player collision info
        let ball = nc::shape::Ball::new(10.0);
        let mut player_collide_group = nc::world::CollisionGroups::new();
        player_collide_group.set_membership(&[2]);
        let query_type = nc::world::GeometricQueryType::Contacts(0.0, 0.0);

        let player_collider = {
            let mut collide_world = world.specs_world.write_resource::<CollisionWorld>();
            let player_handle = collide_world.add(
                na::Isometry2::new(na::Vector2::new(220.0, 120.0), na::zero()),
                nc::shape::ShapeHandle::new(ball.clone()),
                player_collide_group,
                query_type,
                entity,
            );

            Collider {
                object_handle: player_handle,
            }
        };
        // Insert the collider.
        world.specs_world.write_storage::<Collider>().insert(entity, player_collider)?;
        Ok(entity)
    }

    fn create_planet(ctx: &mut ggez::Context, world: &mut World) -> Result<specs::Entity, Err> {
        // Make the world entity
        let entity = world
            .specs_world
            .create_entity()
            .with(Mesh {
                mesh: graphics::MeshBuilder::default()
                    .circle(
                        graphics::DrawMode::Fill,
                        graphics::Point2::new(0.0, 0.0),
                        100.0,
                        2.0,
                    )
                    .build(ctx)?,
            })
            .build();

        // Planet collision info
        let ball = nc::shape::Ball::new(100.0);
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
                entity,
            );

            Collider {
                object_handle: planet_handle,
            }
        };
        // Insert the collider.
        world.specs_world.write_storage::<Collider>().insert(entity, planet_collider)?;
        Ok(entity)
    }

    fn handle_contact_events(&mut self, gameworld: &mut World) {
        let mut collide_world = gameworld.specs_world.write_resource::<CollisionWorld>();
        collide_world.update();
        let mut player_storage = gameworld.specs_world.write_storage::<Player>();

        // Save and reuse the same vec each run of the loop so we only allocate once.
        let contacts_list = &mut Vec::new();
        for e in collide_world.contact_events() {
            contacts_list.clear();
            match e {
                nc::events::ContactEvent::Started(cobj_handle1, cobj_handle2) => {
                    self.collided = true;
                    // It's apparently possible for the collision pair to have
                    // no contacts...
                    // Possibly if one object is entirely inside another?
                    if let Some(pair) = (&*collide_world).contact_pair(*cobj_handle1, *cobj_handle2) {
                        pair.contacts(contacts_list);
                        let cobj1 = collide_world.collision_object(*cobj_handle1)
                            .expect("Invalid collision object handle?");
                        let cobj2 = collide_world.collision_object(*cobj_handle2)
                            .expect("Invalid collision object handle?");

                        // Get the entities out of the collision data
                        let e1 = cobj1.data();
                        let e2 = cobj2.data();
                        for e in &[e1, e2] {
                            if let Some(player) = player_storage.get_mut(**e) {
                                player.on_ground = true;
                            }
                        }
                    }
                }
                nc::events::ContactEvent::Stopped(cobj_handle1, cobj_handle2) => {
                    self.collided = false;

                    if let Some(pair) = (&*collide_world).contact_pair(*cobj_handle1, *cobj_handle2) {
                        pair.contacts(contacts_list);
                        let cobj1 = collide_world.collision_object(*cobj_handle1)
                            .expect("Invalid collision object handle?");
                        let cobj2 = collide_world.collision_object(*cobj_handle2)
                            .expect("Invalid collision object handle?");

                        // Get the entities out of the collision data
                        let e1 = cobj1.data();
                        let e2 = cobj2.data();
                        for e in &[e1, e2] {
                            if let Some(player) = player_storage.get_mut(**e) {
                                player.on_ground = false;
                            }
                        }
                    }
                }
            }
        }
    }


    /// This is really hard to express as a specs System so we roll our own.
    fn run_player_motion(&mut self, world: &mut World) {
        if let Some(player) = world.specs_world.write_storage::<Player>().get_mut(self.player_entity) {
            let mut colliders = world.specs_world.write_storage::<Collider>();
            let mut motions = world.specs_world.write_storage::<Motion>();
            let mut ncollide_world = world.specs_world.write_resource::<CollisionWorld>();

            let player_motion = motions.get_mut(self.player_entity)
                .expect("Player w/o motion?");
            let player_collider = colliders.get(self.player_entity)
                .expect("Player w/o motion?");
            let (player_position, player_rotation) = collision_object_position(&*ncollide_world, &player_collider);
            let planet_collider = colliders.get(self.planet_entity)
                .expect("Planet w/o collider?");
            let (planet_position, _planet_rotation) = collision_object_position(&*ncollide_world, planet_collider);

            let offset = player_position - planet_position;
            let normal = offset / na::norm(&offset);
            if player.on_ground {
                // We only want to zero the Y component
                // of the velocity... that is, the portion
                // towards the planet.
                // So we take the projection of velocity onto
                // the toward-the-planet offset vector.
                let projection = na::dot(&player_motion.velocity, &(offset / na::norm(&offset)));
                // debug!("Projection is {:?}, offset is {:?}, velocity is {}", projection, offset, player_motion.velocity);

                player_motion.velocity -= na::normalize(&offset) * projection;

                player_motion.acceleration = na::zero();
                // Jump
                if player.jumping {
                    player_motion.acceleration += normal;
                    player.on_ground = false;
                }

                // Walk
                use std::f32;
                let rot = na::Rotation2::new(f32::consts::PI / 2.0);
                let walk_direction = rot * (normal * player.walk_direction);
                player_motion.acceleration += walk_direction * player.walk_force;
            }
            player_motion.velocity += player_motion.acceleration;
            player_motion.acceleration = na::zero();

            let new_position = {
                let collision_obj = ncollide_world
                    .collision_object(player_collider.object_handle)
                    .expect(
                        "Invalid collision object; was it removed from ncollide but not specs?",
                    );
                let mut new_position = collision_obj.position().clone();
                new_position.append_translation_mut(&na::Translation::from_vector(player_motion.velocity));
                new_position
            };
            ncollide_world.set_position(player_collider.object_handle, new_position);
        }
    }
}

/// Takes a collision object handle and returns the location and orientation
/// of the object.
fn collision_object_position(
    ncollide_world: &CollisionWorld,
    collider: &Collider,
) -> (Point2, f32) {
    let collision_object = ncollide_world
        .collision_object(collider.object_handle)
        .expect("Invalid collision object; was it removed from ncollide but not specs?");
    let isometry = collision_object.position();
    let annoying_new_pos =
        Point2::new(isometry.translation.vector.x, isometry.translation.vector.y);
    let annoying_new_angle = isometry.rotation.angle();
    (annoying_new_pos, annoying_new_angle)
}



/// augh
///
/// Mainly used for drawing, so it returns ggez's Point type rather than ncollide's.
fn ggez_collision_object_position(
    ncollide_world: &CollisionWorld,
    collider: &Collider,
) -> (graphics::Point2, f32) {
    let (point, rotation) = collision_object_position(ncollide_world, collider);
    (graphics::Point2::new(point.x, point.y), rotation)
}


impl scene::Scene<World, input::InputEvent> for LevelScene {
    fn update(&mut self, gameworld: &mut World) -> FSceneSwitch {
        self.run_player_motion(gameworld);
        self.dispatcher.dispatch(&mut gameworld.specs_world.res);

        self.handle_contact_events(gameworld);
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
            let (pos, angle) = ggez_collision_object_position(&*ncollide_world, c);
            graphics::draw_ex(
                ctx,
                &(self.sprite.borrow().0),
                graphics::DrawParam {
                    dest: pos,
                    rotation: angle,
                    ..graphics::DrawParam::default()
                },
            )?;
        }

        for (c, mesh) in (&collider, &mesh).join() {
            let (pos, angle) = ggez_collision_object_position(&*ncollide_world, c);
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
        if gameworld.input.get_button_pressed(input::Button::Menu) {
            gameworld.quit = true;
        }
        if let Some(player) = gameworld.specs_world.write_storage::<Player>().get_mut(self.player_entity) {
            player.jumping = gameworld.input.get_button_pressed(input::Button::Jump);
            player.walk_direction = gameworld.input.get_axis(input::Axis::Horz);
        }
    }
}
