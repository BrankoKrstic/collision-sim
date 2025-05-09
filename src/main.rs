use macroquad::{prelude::*, rand::RandGenerator};

static RNG: RandGenerator = RandGenerator::new();

const RADIUS_MIN: f32 = 6.0;
const RADIUS_MAX: f32 = 12.0;

const VELOCITY_MIN: f32 = -20.0;
const VELOCITY_MAX: f32 = 20.0;

const MARGIN: f32 = 50.0;
const WIDTH: f32 = 1800.0;
const HEIGHT: f32 = 950.0;

const FRAME_TIME: f32 = 1.0 / 60.0;

const FLIP_Y: Mat2 = Mat2::from_cols(Vec2::new(1.0, 0.0), Vec2::new(0.0, -1.0));
const FLIP_X: Mat2 = Mat2::from_cols(Vec2::new(-1.0, 0.0), Vec2::new(0.0, 1.0));

struct World {
    balls: Vec<Ball>,
}

impl World {
    fn new() -> Self {
        Self { balls: vec![] }
    }
    fn add_ball(&mut self, b: Ball) {
        self.balls.push(b);
    }
    fn tick(&mut self) {
        for b in &mut self.balls {
            b.compute_collision();
        }
        let mut possible_collisions = self
            .balls
            .iter()
            .enumerate()
            .map(|(i, b)| (i, b.position.x - b.radius, b.position.x + b.radius))
            .collect::<Vec<_>>();
        possible_collisions.sort_by(|b1, b2| b1.1.partial_cmp(&b2.1).unwrap());

        for i in 0..possible_collisions.len() - 1 {
            'inner: for j in (i + 1)..possible_collisions.len() {
                if possible_collisions[i].2 < possible_collisions[j].1 {
                    break 'inner;
                } else {
                    let i = possible_collisions[i].0;
                    let j = possible_collisions[j].0;

                    let (left_i, right_i) = if i < j { (i, j) } else { (j, i) };
                    let (head, tail) = self.balls.split_at_mut(left_i + 1);

                    if let Some(cur) = head.last_mut() {
                        let b = &mut tail[right_i - left_i - 1];
                        cur.collide(b);
                    }
                }
            }
        }

        for b in &mut self.balls {
            b.update(FRAME_TIME);
        }
    }
    fn draw(&self) {
        clear_background(BLACK);
        draw_rectangle_lines(MARGIN, MARGIN, WIDTH, HEIGHT, 2.0, BLUE);
        for b in &self.balls {
            b.draw();
        }
    }
}

#[derive(Debug, PartialEq)]
struct Ball {
    radius: f32,
    position: Vec2,
    velocity: Vec2,
    color: Color,
}

impl Ball {
    fn new(position: Vec2, velocity: Vec2, radius: f32, color: Color) -> Self {
        Self {
            radius,
            position,
            velocity,
            color,
        }
    }
    fn random() -> Self {
        let radius = RNG.gen_range(RADIUS_MIN, RADIUS_MAX).floor();
        let x = RNG.gen_range(MARGIN + radius, WIDTH + MARGIN - radius);
        let y = RNG.gen_range(MARGIN + radius, HEIGHT + MARGIN - radius);
        let vx = RNG.gen_range::<f32>(VELOCITY_MIN, VELOCITY_MAX);
        let vy = RNG.gen_range::<f32>(VELOCITY_MIN, VELOCITY_MAX);
        let color = Color::new(
            RNG.gen_range(0.0, 1.0),
            RNG.gen_range(0.0, 1.0),
            RNG.gen_range(0.0, 1.0),
            1.0,
        );
        Ball::new(Vec2::new(x, y), Vec2::new(vx, vy), radius, color)
    }
    fn collides_with(&self, b: &Ball) -> bool {
        self.position.distance(b.position) < self.radius + b.radius
    }
    fn collide(&mut self, b: &mut Ball) {
        if !self.collides_with(b) {
            return;
        }
        let m1 = self.radius * self.radius;
        let m2: f32 = b.radius * b.radius;
        let v1 = self.velocity;
        let v2 = b.velocity;
        let p1 = self.position;
        let p2 = b.position;

        self.velocity = compute_velocity_after_collision(v1, v2, m1, m2, p1, p2);
        b.velocity = compute_velocity_after_collision(v2, v1, m2, m1, p2, p1);
    }
    fn update(&mut self, dt: f32) {
        // self.speed *= 0.999;
        // let acceleration = if self.position.y + self.radius < 500.0 {
        //     Vec2::new(0.0, 981.0)
        // } else {
        //     Vec2::default()
        // };
        // self.speed += acceleration * dt;
        self.position += self.velocity * dt;
    }
    fn compute_collision(&mut self) {
        if (self.position.x - self.radius < MARGIN && self.velocity.x < 0.0)
            || (self.position.x + self.radius > WIDTH + MARGIN && self.velocity.x > 0.0)
        {
            self.velocity = FLIP_X * self.velocity;
        }

        if (self.position.y - self.radius < MARGIN && self.velocity.y < 0.0)
            || (self.position.y + self.radius > HEIGHT + MARGIN && self.velocity.y > 0.0)
        {
            self.velocity = FLIP_Y * self.velocity;
        }
    }
    fn draw(&self) {
        draw_circle(self.position.x, self.position.y, self.radius, self.color);
    }
}

fn compute_velocity_after_collision(
    v1: Vec2,
    v2: Vec2,
    m1: f32,
    m2: f32,
    p1: Vec2,
    p2: Vec2,
) -> Vec2 {
    let dpos = p1 - p2;

    v1 - (2.0 * m2 / (m1 + m2) * (v1 - v2).dot(dpos) / dpos.length_squared() * dpos)
}

fn window_conf() -> Conf {
    Conf {
        window_title: "Collider".to_owned(),
        fullscreen: true,
        ..Default::default()
    }
}
#[macroquad::main(window_conf)]
async fn main() {
    let mut world = World::new();
    for _ in 0..1000 {
        let mut new_ball = Ball::random();
        while world.balls.iter().any(|b| b.collides_with(&new_ball)) {
            new_ball = Ball::random();
        }
        world.add_ball(new_ball);
    }
    let mut total_time = 0.0;
    loop {
        total_time += get_frame_time();
        while total_time >= FRAME_TIME {
            total_time -= FRAME_TIME;
            world.tick();
            world.draw();
            next_frame().await;
        }
    }
}
