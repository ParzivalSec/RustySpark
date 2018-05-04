use std::mem;
use calx_ecs::Entity;

const ENTITY_NUM: usize = 10_000;
const ENTITY_NUM_LARGE: usize = 100_000;

#[derive(Copy, Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct Position {
    pub x: f32,
    pub y: f32,
    pub z: f32,
}

impl Position {
    fn new(x: f32, y: f32, z: f32) -> Self {
        Position { x, y, z}
    }
}

#[derive(Copy, Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct Velocity {
    pub dx: f32,
    pub dy: f32,
}

impl Velocity {
    fn new(dx: f32, dy: f32) -> Self {
        Velocity { dx, dy }
    }
}

Ecs! {
    pos: Position,
    vel: Velocity,
}

pub fn ecs_create_10000_with_pos() {
    let mut ecs = Ecs::new();
    let mut entities: [Entity; ENTITY_NUM] = unsafe { mem::uninitialized() };

    for idx in 0 .. ENTITY_NUM {
        let entity = ecs.make();
        ecs.pos.insert(entity, Position::new(1.0, 2.0, 3.0));
        entities[idx] = entity;
    }
}

pub fn ecs_create_100000_with_pos() {
    let mut ecs = Ecs::new();
    let mut entities: [Entity; ENTITY_NUM_LARGE] = unsafe { mem::uninitialized() };

    for idx in 0 .. ENTITY_NUM_LARGE {
        let entity = ecs.make();
        ecs.pos.insert(entity, Position::new(1.0, 2.0, 3.0));
        entities[idx] = entity;
    }
}

pub fn ecs_create_10000_with_pos_vel() {
    let mut ecs = Ecs::new();
    let mut entities: [Entity; ENTITY_NUM] = unsafe { mem::uninitialized() };

    for idx in 0 .. ENTITY_NUM {
        let entity = ecs.make();
        ecs.pos.insert(entity, Position::new(1.0, 2.0, 3.0));
        ecs.vel.insert(entity, Velocity::new(10.0, 10.0));
        entities[idx] = entity;
    }
}

pub fn ecs_create_100000_with_pos_vel() {
    let mut ecs = Ecs::new();
    let mut entities: [Entity; ENTITY_NUM_LARGE] = unsafe { mem::uninitialized() };

    for idx in 0 .. ENTITY_NUM_LARGE {
        let entity = ecs.make();
        ecs.pos.insert(entity, Position::new(1.0, 2.0, 3.0));
        ecs.vel.insert(entity, Velocity::new(10.0, 10.0));
        entities[idx] = entity;
    }
}

pub fn ecs_iterate_10000_pos() {
    let mut ecs = Ecs::new();
    let mut entities: [Entity; ENTITY_NUM] = unsafe { mem::uninitialized() };

    for idx in 0 .. ENTITY_NUM {
        let entity = ecs.make();
        ecs.pos.insert(entity, Position::new(1.0, 2.0, 3.0));
        ecs.vel.insert(entity, Velocity::new(10.0, 10.0));
        entities[idx] = entity;
    }

    let with_pos: Vec<Entity> = ecs.pos.ent_iter().cloned().collect();
    for e_idx in 0 .. with_pos.len() {
       ecs.pos.get_mut(with_pos[e_idx]).unwrap().x += 10.0;
    }
}

pub fn ecs_iterate_100000_pos() {
    let mut ecs = Ecs::new();
    let mut entities: [Entity; ENTITY_NUM_LARGE] = unsafe { mem::uninitialized() };

    for idx in 0 .. ENTITY_NUM_LARGE {
        let entity = ecs.make();
        ecs.pos.insert(entity, Position::new(1.0, 2.0, 3.0));
        ecs.vel.insert(entity, Velocity::new(10.0, 10.0));
        entities[idx] = entity;
    }

    let with_pos: Vec<Entity> = ecs.pos.ent_iter().cloned().collect();
    for e_idx in 0 .. with_pos.len() {
       ecs.pos.get_mut(with_pos[e_idx]).unwrap().x += 10.0;
    }
}

pub fn ecs_remove_5000_pos() {
    let mut ecs = Ecs::new();
    let mut entities: [Entity; ENTITY_NUM] = unsafe { mem::uninitialized() };

    for idx in 0 .. ENTITY_NUM {
        let entity = ecs.make();
        ecs.pos.insert(entity, Position::new(1.0, 2.0, 3.0));
        ecs.vel.insert(entity, Velocity::new(10.0, 10.0));
        entities[idx] = entity;
    }

    for idx in 0 .. ENTITY_NUM / 2 {
        ecs.remove(entities[idx]);
    }
}

pub fn ecs_remove_50000_pos() {
    let mut ecs = Ecs::new();
    let mut entities: [Entity; ENTITY_NUM_LARGE] = unsafe { mem::uninitialized() };

    for idx in 0 .. ENTITY_NUM_LARGE {
        let entity = ecs.make();
        ecs.pos.insert(entity, Position::new(1.0, 2.0, 3.0));
        ecs.vel.insert(entity, Velocity::new(10.0, 10.0));
        entities[idx] = entity;
    }

    for idx in 0 .. ENTITY_NUM_LARGE / 2 {
        ecs.remove(entities[idx]);
    }
}