use bevy::{
    log::tracing_subscriber::field::debug,
    math::ops::{cos, sin},
};
use macroquad::prelude::*;
use std::cmp::Ordering;

const PLAYER_COLOR: Color = GRAY;
const PLAYER_SPEED: f32 = 0.01;

const ASTEROID_COLOR: Color = WHITE;
const ASTEROID_RADIUS: f32 = 10.0;
const ASTEROID_THICKNESS: f32 = 2.0;
const ASTEROID_SPEED: f32 = 0.5;

const BULLET_SIZE: f32 = 2.0;
const BULLET_COLOR: Color = GREEN;

#[macroquad::main("Main")]
async fn main() {
    let mut num_lives: u16 = 100;
    let mut player = Player::default_player();
    clear_background(BLACK);
    let mut bullets: Vec<Bullet> = Vec::new();
    let mut asteroids: Vec<Asteroid> = Vec::new();

    loop {
        let player_position = player.parse_x();
        if num_lives == 0 {
            print!("Game over");
        }
        if is_key_pressed(KeyCode::Space) {
            let mut bullet = Bullet {
                x: player.x.x,
                y: player.x.y,
            };
            bullets.push(bullet);
        }
        if is_key_pressed(KeyCode::LeftControl) {
            let mut asteroid = Asteroid {
                x: screen_height() / 2.0,
                y: screen_width() / 2.0,
                live: true,
            };
            asteroids.push(asteroid);
        }

        if is_key_down(KeyCode::Right) {
            player = player.rotate_cw();
        }
        if is_key_down(KeyCode::Left) {
            player = player.rotate_ccw();
        }
        let direction = Bullet::direction(player.center(), player.x);
        let mut bullet_position = Point { x: 0.0, y: 0.0 };
        for bullet in &mut bullets {
            bullet.x += direction.x;
            bullet.y += direction.y;
            bullet.new(); 
            bullet_position = bullet.parse();
        }
        let mut asteroid_position = Point { x: 0.0, y: 0.0 };
        for asteroid in &mut asteroids {
            asteroid.move_to_player(player.parse_x());
            asteroid.new();
            asteroid_position = asteroid.parse();
            if (asteroid_position.x - bullet_position.x) < 1.0
                && (asteroid_position.y - bullet_position.y) < 1.0
            {
                asteroid.live = false;
            }

            if (asteroid.parse().x - player_position.x) < 1.0
                && (asteroid.parse().y - player_position.y) < 1.0
            {
                //num_lives -= 1;
                //print!("HIT, Lives remaining {}", num_lives);
                asteroid.live = false;
            }
        }

        player.draw_player();
        next_frame().await;
    }
}
struct Player {
    x: Vec2,
    y: Vec2,
    z: Vec2,
    angle: f32,
}
#[derive(Debug)]
struct Point {
    x: f32,
    y: f32,
}

struct BetterPlayer {
    base: [(f32, f32); 3],
    angle: f32,
}

trait Thing {
    fn new(); // diff implementations
    fn parse(); // 2 identical implementations, 1 different
}

impl Player  {
    fn default_player() -> Self {
        Player {
            x: Vec2 {
                x: screen_width() / 2.0 - 15.0,
                y: screen_height() / 2.0 - 30.0,
            },
            y: Vec2 {
                x: screen_width() / 2.0 - 15.0,
                y: screen_height() / 2.0 + 15.0,
            },
            z: Vec2 {
                x: screen_width() / 2.0 + 15.0,
                y: screen_height() / 2.0,
            },
            angle: PLAYER_SPEED,
        }
    }

    fn parse_x(&self) -> Point {
        // can also iter and give all but only need x so.. too bad!
        let x = self.x.x;
        let y = self.x.y;
        Point { x, y }
    }

    fn center(&self) -> Point {
        let cx = (self.x.x + self.y.x + self.z.x) / 3.0;
        let cy = (self.x.y + self.y.y + self.z.y) / 3.0;

        Point { x: cx, y: cy }
    }
    fn iter(&self) -> impl Iterator<Item = &Vec2> {
        [&self.x, &self.y, &self.z].into_iter()
    }

    fn iter_mut(&mut self) -> impl Iterator<Item = &mut Vec2> {
        [&mut self.x, &mut self.y, &mut self.z].into_iter()
    }

    fn rotate(mut self, theta: f32) -> Self {
        let cx = (self.x.x + self.y.x + self.z.x) / 3.0;
        let cy = (self.x.y + self.y.y + self.z.y) / 3.0;
        let cos = cos(self.angle);
        let sin = sin(self.angle);

        for curr in self.iter_mut() {
            let (dx, dy) = (curr.x - cx, curr.y - cy);
            let nx = cx + dx * cos - dy * sin;
            let ny = cy + dx * sin + dy * cos;
            curr.x = nx;
            curr.y = ny;
        }

        self
    }
    fn rotate_cw(mut self) -> Self {
        self.rotate(1.0)
    }
    fn rotate_ccw(mut self) -> Self {
        self.rotate(-1.0)
    }

    fn draw_player(&self) {
        draw_triangle(self.x, self.y, self.z, PLAYER_COLOR);
    }
}
struct Asteroid {
    x: f32,
    y: f32,
    live: bool,
}

impl Asteroid {
    fn parse(&self) -> Point {
        Point {
            x: self.x,
            y: self.y,
        }
    }

    fn new(&mut self) {
        if self.live {
            draw_circle_lines(
                self.x,
                self.y,
                ASTEROID_RADIUS,
                ASTEROID_THICKNESS,
                ASTEROID_COLOR,
            );
        }
    }

    fn move_to_player(&mut self, target: Point) {
        if self.x > target.x {
            self.x -= ASTEROID_SPEED;
        } else {
            self.x += ASTEROID_SPEED;
        }
        if self.y > target.y {
            self.y -= ASTEROID_SPEED;
        } else {
            self.y += ASTEROID_SPEED;
        }
    }
}

struct Bullet {
    x: f32,
    y: f32,
}

impl Bullet {
    fn new(&mut self) {
        self.x += 1.0;
        draw_circle(self.x, self.y, BULLET_SIZE, BULLET_COLOR);
    }
    fn parse(&self) -> Point {
        Point {
            x: self.x,
            y: self.y,
        }
    }
    fn direction(center: Point, player: Vec2) -> Point {
        let mut dir: Point = Point { x: 0.0, y: 0.0 };
        let cx = center.x;
        let px = player.x;
        let cy = center.y;
        let py = player.y;

        match cx.partial_cmp(&px) {
            Some(Ordering::Greater) => dir.x = px - cx,
            Some(Ordering::Less) => dir.x = px - cx,
            Some(Ordering::Equal) => dir.x = px,
            None => {}
        }
        match cy.partial_cmp(&py) {
            Some(Ordering::Greater) => dir.y = py - cy,
            Some(Ordering::Less) => dir.y = py - cy,
            Some(Ordering::Equal) => dir.y = py,
            None => {}
        }

        dir
    }
}
