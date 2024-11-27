//! Use Composition over inheritance in Rust !

use crate::{infos, vector_improver::EguiConvertCompatibility, world::World};
use rapier2d::prelude::*;
use nalgebra::Vector2;

//////////// FUNCTIONS

fn draw_circular_generic(
    painter: &egui::Painter,
    pos: egui::Pos2,
    radius: f32,
    color: egui::Color32,
    offset: egui::Vec2,
    scale: f32,
) {
    painter.circle_filled((pos * scale) + offset, radius * scale, color);
}

//////////// CIRCULARMOVEBASE

pub struct CircularMoveBuilder {
    pub position: Vector2<f32>,
    pub velocity: Vector2<f32>,
    pub friction: f32,
    pub mass: f32,
    pub radius: f32,
}

impl CircularMoveBuilder {
    fn base() -> Self {
        CircularMoveBuilder {
            position: Vector2::zeros(),
            velocity: Vector2::zeros(), //TODO
            friction: 0.0,
            mass: 10.0,
            radius: 10.0,
        }
    }

    fn build(self, world: &mut World) -> (RigidBodyHandle, ColliderHandle) {
        let body = RigidBodyBuilder::dynamic()
            .translation(self.position)
            .build();
        let handle = world.rigid_body_set.insert(body);
        let collider = ColliderBuilder::ball(self.radius)
            .active_events(ActiveEvents::COLLISION_EVENTS)
            .mass(self.mass)
            .friction(self.friction)
            .build();
        let collider_handle = world.collider_set.insert_with_parent(collider, handle, &mut world.rigid_body_set);
        (handle, collider_handle) //TODO: refactorisation
    }
}

////////////  ROBOT

#[derive(Debug)]
pub struct Robot {
    pub handle: RigidBodyHandle,
    pub collider_handle: ColliderHandle,
    pub color: egui::Color32,
}

impl Robot {
    pub fn new(world: &mut World, position: Vector2<f32>, color: egui::Color32, mass: f32) -> Self {
        let (handle, collider_handle) = CircularMoveBuilder {
            position,
            mass,
            radius: infos::ROBOT_RADIUS,
            ..CircularMoveBuilder::base()
        }.build(world);
        Self {
            handle,
            collider_handle,
            color,
        }
    }

    pub fn draw(&self, world: &World, painter: &egui::Painter, offset: egui::Vec2, scale: f32) {
        draw_circular_generic(
            painter,
            world.rigid_body_set[self.handle].position().translation.vector.to_egui_pos2(),
            infos::ROBOT_RADIUS,
            self.color,
            offset,
            scale,
        );
    }

    pub fn collide_with(&self, other: &Robot) {
        println!("{:?} collides with {:?}", self, other);
    }
}

////////////  BALL

#[derive(Debug)]
pub struct Ball {
    pub handle: RigidBodyHandle,
    pub collider_handle: ColliderHandle,
    pub color: egui::Color32,
}

impl Ball {
    pub fn new(world: &mut World, position: Vector2<f32>, color: egui::Color32, mass: f32) -> Self {
        let (handle, collider_handle) = CircularMoveBuilder {
            position,
            mass,
            radius: infos::BALL_RADIUS,
            ..CircularMoveBuilder::base()
        }.build(world);
        Self {
            handle,
            collider_handle,
            color,
        }
    }

    pub fn draw(&self, world: &World, painter: &egui::Painter, offset: egui::Vec2, scale: f32) {
        draw_circular_generic(
            painter,
            world.rigid_body_set[self.handle].position().translation.vector.to_egui_pos2(),
            infos::BALL_RADIUS,
            self.color,
            offset,
            scale,
        )
    }
}
