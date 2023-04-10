#![recursion_limit = "20000"]

use std::rc::Rc;

pub mod ball;
pub use ball::Ball;

use generational_arena::Arena;
use raylib::prelude::*;

const HEIGHT: isize = 480;
const WIDTH: isize = 640;
const GRAVITY: Vector2 = Vector2{ x: 0.0, y: 30.0};
const BALL_RADIUS: f32 = 10.0;
const FRICTION: f32 = 0.80;
const ACCELERATION: f32 = 0.5;
const MAX_SPEED: f32 = 20.0;
const BALL_MASS: f32 = 10.0;
const BALL_COEFF_RESTITUTION: f32 = 0.10;
const TARGET_FPS: u32 = 60;
const MARGIN: f32 = 3.0;

fn main() {
    let (mut rl, thread) = raylib::init()
        .size(WIDTH as i32, HEIGHT as i32)
        .title("Hello, World")
        .build();

    // limit the framerate to 60
    rl.set_target_fps(TARGET_FPS);

    let mut arena: Arena<Ball> = Arena::new();

    let mut color_generator = ColorGenerator::new();

    let mut game_state = GameState {
        moves: vec![],
        arena: arena,
        collisions: vec![],
        balls: vec![],
        color_generator: color_generator,
    };

    let mut ctx = 0;
    let mut nb_ball = 0;
    let dt = 1.0 / TARGET_FPS as f32;

    while !rl.window_should_close() {
        let mut d = rl.begin_drawing(&thread);
        d.clear_background(Color::BLACK);
        d.draw_fps(0, 0);
        ctx = ctx + 1;

        d.draw_text(
            &format!("balls : {}", nb_ball),
            0,
            20,
            20,
            Color::new(255, 255, 255, 255),
        );
        if ctx == 100 {
            nb_ball = nb_ball + 1;
            game_state.add_ball(
                (250.0 + ctx as f32) % WIDTH as f32,
                55.0 + (ctx % 4) as f32,
                BALL_RADIUS,
            );
        }
        game_state
            // .handle_collisions(dt)
            .apply_velocity(dt)
            .update_balls()
            .draw_balls(&mut d);

        //game_state.monitor_drift();
    }
}

// make a struct for the game state
struct GameState {
    moves: Vec<Move>,
    arena: Arena<Ball>,
    collisions: Vec<(generational_arena::Index, generational_arena::Index)>,
    balls: Vec<Rc<Ball>>,
    color_generator: ColorGenerator,
}

struct ColorGenerator {
    index: u8,
}

impl ColorGenerator {
    fn new() -> ColorGenerator {
        ColorGenerator { index: 0 }
    }

    fn consumme(&mut self) -> Color {
        self.index = ((self.index as usize + 11) % 254) as u8;
        if (self.index as usize) > 255 {
            self.index = 0;
        }
        let c = match self.index % 6 {
            0 => Color::new(255, 0, self.index, 255),
            1 => Color::new(255, self.index, self.index, 255),
            2 => Color::new(self.index, 255, 0, 255),
            3 => Color::new(0, 255, self.index, 255),
            4 => Color::new(0, self.index, 255, 255),
            5 => Color::new(self.index, 0, 255, 255),
            _ => Color::new(0, 0, 0, 255),
        };

        return c;
    }
}

impl GameState {
    fn add_ball(&mut self, x: f32, y: f32, radius: f32) -> &mut GameState {
        let color = self.color_generator.consumme();
        self.arena.insert(Ball {
            position: Vector2::new(x, y),
            last_position: Vector2::new( x - 1.0, y),
            acceleration: Vector2::new(1.0, 0.0),
            radius: radius,
            color: color,
            mass: BALL_MASS,
            coeff_restitution: BALL_COEFF_RESTITUTION,
            friction: FRICTION,
        });
        return self;
    }

    fn apply_velocity(&mut self, dt: f32) -> &mut GameState {
        for (_, ball) in self.arena.iter_mut() {
            ball.acceleration += GRAVITY;
            ball.apply(dt);
        }
        self.handle_oob();
        return self;
    }

    fn draw_balls(&mut self, d: &mut RaylibDrawHandle) -> &mut GameState {
        for (_, ball) in self.arena.iter() {
            let values = ball.get_drawable();
            d.draw_circle(values.0 as i32, values.1 as i32, values.2, values.3);
        }
        return self;
    }

    fn handle_collisions(&mut self, dt: f32) -> &mut GameState {
        // make 8 substeps

        let nb_iter: u32 = 8;
        // calculate the time step
        let dt = 1.0 / (TARGET_FPS as f32 * nb_iter as f32);

        for i in 1..nb_iter {
            // self.check_for_collisions()
            //     .solve_collisions(dt)
            
        }

        return self;
    }

    fn check_for_collisions(&mut self) -> &mut GameState {
        for (idx, ball) in self.arena.iter() {
            for (jdx, ball2) in self.arena.iter() {
                if idx == jdx || jdx > idx {
                    continue;
                }
                let ball_a = ball.get_position();
                let ball_b = ball2.get_position();
                let distance =
                    ((ball_a.x - ball_b.x).powi(2) + (ball_a.y - ball_b.y).powi(2)).sqrt();
                if distance <= (2.0 * BALL_RADIUS) {
                    self.collisions.push((idx, jdx));
                }
            }
        }
        return self;
    }

    /*
    fn linear_impulse_resolution(&mut self) -> &mut GameState {
        for (idx, jdx) in &self.collisions {
            let (ball_a, ball_b) = self.arena.get2_mut(*idx, *jdx);

            let mut ball_a = ball_a.unwrap();s
            let mut ball_b = ball_b.unwrap();

            let e = ball_a.coeff_restitution.min(ball_b.coeff_restitution);
            let numerator =
                -(1.0 + e) * ball_a.velocity.relative_velocity(&ball_b.velocity).nomal();
            let denominator = 1.0 / ball_a.mass + 1.0 / ball_b.mass;
            let impulse = numerator / denominator;
            let impulse = impulse / 2.0 ;
            ball_a.velocity.x += impulse / ball_a.mass * -1.0;
            ball_b.velocity.x += impulse / ball_b.mass * 1.0;

            ball_a.velocity.y += impulse / ball_a.mass * -1.0;
            ball_b.velocity.y += impulse / ball_b.mass * 1.0;

        self.collisions.clear();

        return self;
    }
    */

    // fn solve_collisions(&mut self, dt: f32) -> &mut GameState {
    //     for (idx, jdx) in &self.collisions {
    //         let (ball_a, ball_b) = self.arena.get2_mut(*idx, *jdx);

    //         let response_coef = BALL_COEFF_RESTITUTION;

    //         let mut ball_a = ball_a.unwrap();
    //         let mut ball_b = ball_b.unwrap();
    //         let min_dist: f32 = ball_a.get_radius() + ball_b.get_radius();

    //         let ball_a_pos = ball_a.get_position();
    //         let ball_b_pos = ball_b.get_position();

    //         let o2_o1 = Vector2::new(ball_a_pos.x - ball_b_pos.x, ball_a_pos.y - ball_b_pos.y);
    //         let dist2 = o2_o1.x.powi(2) + o2_o1.y.powi(2);
    //         let dist = dist2.sqrt();

    //         let delta = 0.5 * response_coef * (dist - min_dist);
    //         let col_vec = o2_o1 / dist * delta;

    //         if dist2 > 0.2 {
    //             ball_a.set_velocity(ball_a.get_velocity() + col_vec / ball_a.get_mass());
    //             ball_b.set_velocity(ball_b.get_velocity() - col_vec / ball_b.get_mass());
    //         }
    //     }
    //     self.collisions.clear();
    //     return self;
    // }

    fn handle_oob(&mut self) -> &mut GameState {
        for (_, ball) in self.arena.iter_mut() {
            if ball.position.x + ball.radius > WIDTH as f32 - MARGIN {
                ball.position.x = WIDTH as f32 - MARGIN;
            } else if ball.position.x < MARGIN {
                ball.position.x = MARGIN;
            }
            if ball.position.y + ball.radius > HEIGHT as f32 {
                ball.position.y = HEIGHT as f32 - MARGIN - ball.radius;
            } else if ball.position.y < MARGIN {
                ball.position.y = MARGIN;
            }
        }
        return self;
    }

    fn update_balls(&mut self) -> &mut GameState {
        for (_, ball) in self.arena.iter_mut() {

            // if ball.get_speed() < 1.0 {
            //     ball.x = ball.last_position_x;
            //     ball.y = ball.last_position_y;
            // }
        }
        return self;
    }
}

// Make a grid of balls

struct Grid {
    width: usize,
    height: usize,
    rows: usize,
    cols: usize,
    spacing: f32,
    // make the matrix take an option of a reference of ball
}

#[derive(Debug)]
struct Move {
    from: (usize, usize),
    to: (usize, usize),
}
