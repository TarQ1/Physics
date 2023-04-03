use raylib::prelude::*;

#[derive(Debug, Copy, Clone)]
pub struct Ball {
    pub position: Vector2,
    pub last_position: Vector2,
    pub velocity: Vector2,
    pub radius: f32,
    pub color: Color,
    pub mass: f32,
    pub coeff_restitution: f32,
    pub friction: f32,
}

impl Ball {
    pub fn get_velocity(&self) -> Vector2 {
        return self.velocity;
    }

    pub fn set_velocity(&mut self, velocity: Vector2) -> &mut Ball {
        self.velocity = velocity;
        return self;
    }

    pub fn get_position(&self) -> Vector2 {
        return self.position;
    }

    pub fn set_position(&mut self,  new_pos : Vector2) -> &mut Ball {
        self.last_position.x = self.position.x;
        self.last_position.y = self.position.y;
        self.position.x = new_pos.x;
        self.position.y = new_pos.y;
        return self;
    }

    pub fn apply_velocity(&mut self) -> &mut Ball {
        self.last_position.x = self.position.x;
        self.last_position.y = self.position.y;
        self.position.x += self.velocity.x;
        self.position.y += self.velocity.y;
        self.set_velocity(self.velocity * self.friction);
        return self;
    }

    // apply gravity to the ball using Verlet integration
    pub fn apply_gravity(&mut self, gravity: f32) -> &mut Ball {
        self.velocity.y += gravity;
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
