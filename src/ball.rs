use raylib::prelude::*;

use crate::BALL_MASS;

#[derive(Debug, Copy, Clone)]
pub struct Ball {
    pub position: Vector2,
    pub last_position: Vector2,
    pub acceleration: Vector2,
    pub radius: f32,
    pub color: Color,
    pub mass: f32,
    pub coeff_restitution: f32,
    pub friction: f32,
}

impl Ball {
    pub fn get_velocity(&self) -> Vector2 {
        return self.position - self.last_position;
    }

    pub fn get_position(&self) -> Vector2 {
        return self.position;
    }

    pub fn set_position(&mut self, new_pos: Vector2) -> &mut Ball {
        self.last_position = self.position;
        self.position = new_pos;
        return self;
    }

    pub fn apply(&mut self, dt: f32) {
        let last_update_move = self.position - self.last_position;
        let new_position = self.position
            + last_update_move
            + (self.acceleration - last_update_move * 40.0) * (dt * dt);
        self.last_position = self.position;
        self.position = new_position;
        self.acceleration = Vector2 { x: 0.0, y: 0.0 };
    }

    pub fn stop(&mut self)
    {
        self.last_position = self.position;
    }

    pub fn slowdown(&mut self ,  ratio : f32)
    {
        self.last_position = self.last_position +  (self.position - self.last_position) * ratio ;
    }

    pub fn  add_velocity(&mut self,  v: Vector2)
    {
        self.last_position -= v;
    }

    pub fn  set_position_same_speed(&mut self, new_position : Vector2)
    {
        let to_last = self.last_position - self.position;
        self.position           = new_position;
        self.last_position      = self.position + to_last;
    }

    pub fn moove(&mut self,  v : Vector2)
    {
        self.position += v;
    }

    pub fn apply_velocity(&mut self, dt: f32) -> &mut Ball {
        self.last_position.x = self.position.x;
        self.last_position.y = self.position.y;
        self.acceleration =
            (Vector2 { x: 0.0, y: 9.8 } + self.acceleration + self.acceleration) / 3.0;
        self.position.x += self.acceleration.x * dt * BALL_MASS;
        self.position.y += self.acceleration.y * dt * BALL_MASS;
        return self;
    }



    pub fn get_drawable(&self) -> (f32, f32, f32, Color) {
        return (self.position.x, self.position.y, self.radius, self.color);
    }

    pub fn get_radius(&self) -> f32 {
        return self.radius;
    }

    pub fn get_mass(&self) -> f32 {
        return self.mass;
    }
}
