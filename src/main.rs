use std::{borrow::BorrowMut, cell::RefCell, rc::Rc};

use raylib::prelude::*;

const HEIGHT: usize = 480;
const WIDTH: usize = 640;
const GRAVITY: f32 = 2.0;
const BALL_RADIUS: f32 = 10.0;
const FRICTION: f32 = 0.99;

fn main() {
    let (mut rl, thread) = raylib::init()
        .size(WIDTH as i32, HEIGHT as i32)
        .title("Hello, World")
        .build();

    // limit the framerate to 60
    rl.set_target_fps(60);

    let mut matrix: Vec<Vec<Option<Ball>>> = vec![];

    // initialize rows

    for _ in 0..((HEIGHT as f32 / BALL_RADIUS) as usize) {
        let mut row: Vec<Option<Ball>> = vec![];
        for _ in 0..((WIDTH as f32 / BALL_RADIUS) as usize) {
            row.push(None);
        }
        matrix.push(row);
    }

    // create a game state
    let grid: Grid = Grid {
        width: WIDTH,
        height: HEIGHT,
        rows: (HEIGHT as f32 / BALL_RADIUS) as usize,
        cols: (WIDTH as f32 / BALL_RADIUS) as usize,
        spacing: 10.0,
        // create an empty matrix with rows= (HEIGHT as f32 / BALL_RADIUS) & cols= (WIDTH as f32 / BALL_RADIUS)
        // fill the matric of None
        matrix: matrix,
    };
    
    let mut game_state = GameState {
        gravity: GRAVITY,
        grid: grid,
        moves: vec![],
        collisions: vec![],
    };
    
    let mut ctx = 0;


    while !rl.window_should_close() {
        let mut d = rl.begin_drawing(&thread);
        d.clear_background(Color::BLACK);
        d.draw_fps(0, 0);
        ctx = ctx + 1;

        game_state
            .apply_gravity()
            .update_position()
            .apply_moves()
            .check_for_collisions()
            .apply_collisions()
            .add_ball(250.0 , 50.0, BALL_RADIUS, Color::RED, ctx)
            .draw_balls(&mut d);

     

        //game_state.monitor_drift();
    }
}

// make a struct for the game state
struct GameState {
    gravity: f32,
    grid: Grid,
    moves: Vec<Move>,
    collisions: Vec<(Ball, Ball)>
}

#[derive(Debug, Copy, Clone)]
struct Ball  {
    x: f32,
    y: f32,
    radius: f32,
    color: Color,
}

impl Ball {
    fn Clone(&self) -> Ball {
        return Ball {
            x: self.x,
            y: self.y,
            radius: self.radius,
            color: self.color,
        };
    }

    fn debug(&self) -> String {
        return format!(
            "Ball {{ x: {}, y: {}, radius: {}}}",
            self.x, self.y, self.radius
        );
    }

}

impl GameState {
    fn add_ball(&mut self, x: f32, y: f32, radius: f32, color: Color, ctx : i32) -> &mut GameState {
        if (ctx % 100 != 0) {
            return self;
        }
        self.grid.matrix[0][0] = Some(Ball {
            x: x,
            y: y,
            radius: radius,
            color: color,
        });

        return self;
    }

    fn apply_gravity(&mut self) -> &mut GameState {
        for vec in &mut self.grid.matrix {
            for ball_option in vec {
                match ball_option {
                    Some(ball) => {
                        // check for out of bounds
                        if ball.y + ball.radius >= self.grid.height as f32 {
                            continue;
                        }
                        ball.y += self.gravity;
                    }
                    None => (),
                }
            }
        }
        return self;
    }

    fn update_position(&mut self) -> &mut GameState {
        // update the position of the ball and put them in the corresponding cell
        let mut res: Vec<Move> = vec![];
        for (idx, vec) in self.grid.matrix.iter().enumerate() {
            // get the indices of the ball
            for (jdx, ball_option) in vec.iter().enumerate() {
                match ball_option {
                    Some(ball) => {
                        let row = (ball.y / BALL_RADIUS) as usize;
                        let col = (ball.x / BALL_RADIUS) as usize;

                        // check if the ball is out of the grid
                        if row >= self.grid.rows as usize || col >= self.grid.cols as usize {
                            continue;
                        }

                        // check if the cell is empty
                        if self.grid.matrix[row][col].is_none() {
                            res.push(Move {
                                from: (idx, jdx),
                                to: (row, col),
                            });
                        }
                    }
                    None => (),
                }
            }
        }
        self.moves = res;
        return self;
    }

    fn draw_balls(&mut self, d: &mut RaylibDrawHandle)  -> &mut GameState{
        for vec in &self.grid.matrix {
            for ball_option in vec {
                match ball_option {
                    Some(ball) => {
                        d.draw_circle(ball.x as i32, ball.y as i32, ball.radius, ball.color)
                    }
                    None => (),
                }
            }
        }
        return self;
    }

    fn apply_moves(&mut self) -> &mut GameState {
        for mov in &self.moves {
            let (from_row, from_col) = mov.from;
            let (to_row, to_col) = mov.to;

            let rest = std::mem::replace(&mut self.grid.matrix[from_row][from_col], None);
            self.grid.matrix[to_row][to_col] = rest;
        }
        return self;
    }

    fn monitor_drift(&self) {
        // print the values of the balls and their grid position to check for drift
        for (idx, vec) in self.grid.matrix.iter().enumerate() {
            for (jdx, ball_option) in vec.iter().enumerate() {
                match ball_option {
                    Some(ball) => {
                        println!("drifto: {} {} {}", ball.debug(), idx, jdx);
                    }
                    None => (),
                }
            }
        }
    }

    fn check_for_collisions(&mut self) -> &mut GameState {
        for (idx, vec) in (&self.grid.matrix).iter().enumerate() {
            // get the indices of the ball
            for (jdx, ball_option) in vec.iter().enumerate() {
                match ball_option {
                    Some(ball) => {
                        let row = (ball.y / BALL_RADIUS) as usize;
                        let col = (ball.x / BALL_RADIUS) as usize;

                        // check the adjacent cells
                        for i in -1..2 {
                            for j in -1..2 {
                                let new_row = row as i32 + i;
                                let new_col = col as i32 + j;
                                if new_row < 0 || new_col < 0 {
                                    continue;
                                }
                                let new_row = new_row as usize;
                                let new_col = new_col as usize;
                                if new_row >= self.grid.rows || new_col >= self.grid.cols {
                                    continue;
                                }
                                match &self.grid.matrix[new_row][new_col] {
                                    Some(other_ball) => {
                                        if other_ball.x != ball.x || other_ball.y != ball.y {
                                            let distance = ((ball.x - other_ball.x).powi(2)
                                                + (ball.y - other_ball.y).powi(2))
                                            .sqrt();
                                            if distance <= (2.0 * BALL_RADIUS) {
                                                self.collisions.push((*ball, *other_ball));
                                                println!(
                                                    "collision: {} {} {} {}",
                                                    ball.debug(),
                                                    idx,
                                                    jdx,
                                                    self.grid.matrix[new_row][new_col]
                                                        .as_ref()
                                                        .unwrap()
                                                        .debug()
                                                );
                                            }
                                        }
                                    }

                                    _ => (),
                                }
                            }
                        }
                    }
                    _ => (),
                }
            }
        }
        // stop borrowing self
        return self;
    }

    fn apply_collisions(&mut self) -> &mut GameState {
        for (ball, other_ball) in &self.collisions {
            // move the ball away from each center
            let distance =
                ((ball.x - other_ball.x).powi(2) + (ball.y - other_ball.y).powi(2)).sqrt();
            let x_diff = (ball.x - other_ball.x) / distance;
            let y_diff = (ball.y - other_ball.y) / distance;
            let x_move = x_diff * BALL_RADIUS;
            let y_move = y_diff * BALL_RADIUS;
            
            let mut ball = match self.grid.matrix[(ball.y / BALL_RADIUS) as usize]
                [(ball.x / BALL_RADIUS) as usize]
                .as_mut()
                {
                    Some(ball) => ball,
                    None => continue,
                };
            
            
            ball.x += x_move;
            ball.y += y_move;

            let mut other_ball = match  self.grid.matrix[(other_ball.y / BALL_RADIUS) as usize]
                [(other_ball.x / BALL_RADIUS) as usize]
                .as_mut()
                {
                    Some(other_ball) => other_ball,
                    None => continue,
                };

            other_ball.x -= x_move;
            other_ball.y -= y_move;
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
    matrix: Vec<Vec<Option<Ball>>>,
}

#[derive(Debug)]
struct Move {
    from: (usize, usize),
    to: (usize, usize),
}
