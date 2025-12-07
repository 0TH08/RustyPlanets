use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::atomic::{AtomicU64, Ordering};

static NEXT_ASTEROID_ID: AtomicU64 = AtomicU64::new(1);

#[derive(Clone, Copy, Debug, Serialize, Deserialize, PartialEq)]
pub struct Vec2 {
    pub x: f32,
    pub y: f32,
}

impl Default for Vec2 {
    fn default() -> Self {
        Self { x: 0.0, y: 0.0 }
    }
}

impl Vec2 {
    pub fn zero() -> Self {
        Self { x: 0.0, y: 0.0 }
    }
    pub fn new(x: f32, y: f32) -> Self {
        Self { x, y }
    }
    pub fn length(&self) -> f32 {
        (self.x * self.x + self.y * self.y).sqrt()
    }
    pub fn normalized(&self) -> Self {
        let l = self.length();
        if l == 0.0 {
            Self::zero()
        } else {
            Self {
                x: self.x / l,
                y: self.y / l,
            }
        }
    }
    pub fn dot(&self, other: &Self) -> f32 {
        self.x * other.x + self.y * other.y
    }
    pub fn scale(&self, s: f32) -> Self {
        Self {
            x: self.x * s,
            y: self.y * s,
        }
    }
    pub fn add(&self, other: &Self) -> Self {
        Self {
            x: self.x + other.x,
            y: self.y + other.y,
        }
    }
    pub fn sub(&self, other: &Self) -> Self {
        Self {
            x: self.x - other.x,
            y: self.y - other.y,
        }
    }
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Asteroid {
    pub id: u64,
    pub position: Vec2,
    pub velocity: Vec2,
    pub radius: f32,
    pub mass: f32,
    pub rotation: f32,
    pub angular_velocity: f32,
    pub integrity: f32,
    pub resources: HashMap<String, u32>,
    pub active: bool,
}

impl Asteroid {
    pub fn new(position: Vec2, velocity: Vec2, radius: f32, mass: f32) -> Self {
        let id = NEXT_ASTEROID_ID.fetch_add(1, Ordering::SeqCst);
        let integrity = (radius * radius * std::f32::consts::PI * 0.5).max(1.0);
        Self {
            id,
            position,
            velocity,
            radius,
            mass: if mass > 0.0 { mass } else { radius * radius },
            rotation: 0.0,
            angular_velocity: 0.0,
            integrity,
            resources: HashMap::new(),
            active: true,
        }
    }
    pub fn with_resources(mut self, resources: HashMap<String, u32>) -> Self {
        self.resources = resources;
        self
    }
    pub fn update(&mut self, dt: f32) {
        if !self.active {
            return;
        }
        self.position = self.position.add(&self.velocity.scale(dt));
        self.rotation += self.angular_velocity * dt;
    }
    pub fn apply_impulse(&mut self, impulse: Vec2) {
        if self.mass <= 0.0 {
            return;
        }
        let dv = impulse.scale(1.0 / self.mass);
        self.velocity = self.velocity.add(&dv);
    }
    pub fn apply_torque(&mut self, torque: f32) {
        if self.mass <= 0.0 {
            return;
        }
        let inertia = self.mass * self.radius * self.radius * 0.5;
        if inertia > 0.0 {
            self.angular_velocity += torque / inertia;
        }
    }
    pub fn contains_point(&self, point: Vec2) -> bool {
        self.position.sub(&point).length() <= self.radius
    }
    pub fn collides_with(&self, other: &Asteroid) -> bool {
        let dist = self.position.sub(&other.position).length();
        dist <= (self.radius + other.radius)
    }
    pub fn resolve_collision(a: &mut Asteroid, b: &mut Asteroid) {
        if !a.active || !b.active {
            return;
        }
        let delta = b.position.sub(&a.position);
        let dist = delta.length().max(1e-6);
        let normal = delta.scale(1.0 / dist);
        let rel_vel = b.velocity.sub(&a.velocity);
        let vel_along_normal = rel_vel.dot(&normal);
        if vel_along_normal > 0.0 {
            return;
        }
        let restitution = 0.6;
        let inv_mass_a = if a.mass > 0.0 { 1.0 / a.mass } else { 0.0 };
        let inv_mass_b = if b.mass > 0.0 { 1.0 / b.mass } else { 0.0 };
        let j = -(1.0 + restitution) * vel_along_normal / (inv_mass_a + inv_mass_b);
        let impulse = normal.scale(j);
        a.velocity = a.velocity.sub(&impulse.scale(inv_mass_a));
        b.velocity = b.velocity.add(&impulse.scale(inv_mass_b));
        let penetration = (a.radius + b.radius) - dist;
        if penetration > 0.0 {
            let total_inv_mass = inv_mass_a + inv_mass_b;
            if total_inv_mass > 0.0 {
                a.position = a
                    .position
                    .sub(&normal.scale(penetration * (inv_mass_a / total_inv_mass)));
                b.position = b
                    .position
                    .add(&normal.scale(penetration * (inv_mass_b / total_inv_mass)));
            }
        }
        let damage = j.abs() * 0.02;
        a.integrity = (a.integrity - damage).max(0.0);
        b.integrity = (b.integrity - damage).max(0.0);
        if a.integrity <= 0.0 {
            a.active = false;
        }
        if b.integrity <= 0.0 {
            b.active = false;
        }
    }
    pub fn split(&mut self) -> Vec<Asteroid> {
        let mut out = Vec::new();
        if self.integrity > self.radius * 0.5 || self.radius <= 4.0 {
            return out;
        }
        let pieces = 2 + ((self.radius / 6.0).floor() as usize);
        let mut total_resource = HashMap::new();
        for (k, v) in &self.resources {
            total_resource.insert(k.clone(), *v);
        }
        for i in 0..pieces {
            let factor = 0.5 + (i as f32) * (0.5 / pieces as f32);
            let child_radius = (self.radius * factor).max(1.0);
            let child_mass =
                self.mass * (child_radius * child_radius) / (self.radius * self.radius);
            let angle = (i as f32) * (std::f32::consts::PI * 2.0 / pieces as f32);
            let vel = Vec2::new(
                self.velocity.x + angle.cos() * 0.5,
                self.velocity.y + angle.sin() * 0.5,
            );
            let mut child = Asteroid::new(self.position, vel, child_radius, child_mass);
            child.rotation = self.rotation + angle;
            child.angular_velocity =
                self.angular_velocity + (i as f32 - (pieces as f32 / 2.0)) * 0.1;
            for (k, v) in &total_resource {
                let share = (*v as f32 * (1.0 / pieces as f32)).round() as u32;
                if share > 0 {
                    child.resources.insert(k.clone(), share);
                }
            }
            out.push(child);
        }
        self.resources.clear();
        self.active = false;
        out
    }
    pub fn damage(&mut self, amount: f32) {
        self.integrity = (self.integrity - amount).max(0.0);
        if self.integrity <= 0.0 {
            self.active = false;
        }
    }
    pub fn harvest(&mut self, resource: &str, amount: u32) -> u32 {
        if let Some(val) = self.resources.get_mut(resource) {
            let take = amount.min(*val);
            *val -= take;
            if *val == 0 {
                self.resources.remove(resource);
            }
            take
        } else {
            0
        }
    }
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct AsteroidSnapshot {
    pub id: u64,
    pub position: Vec2,
    pub velocity: Vec2,
    pub radius: f32,
    pub rotation: f32,
    pub angular_velocity: f32,
    pub integrity: f32,
    pub active: bool,
}

impl From<&Asteroid> for AsteroidSnapshot {
    fn from(a: &Asteroid) -> Self {
        Self {
            id: a.id,
            position: a.position,
            velocity: a.velocity,
            radius: a.radius,
            rotation: a.rotation,
            angular_velocity: a.angular_velocity,
            integrity: a.integrity,
            active: a.active,
        }
    }
}
