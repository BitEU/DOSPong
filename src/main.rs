use crossterm::{
    cursor::{Hide, MoveTo, Show},
    event::{self, Event, KeyCode, KeyEvent, KeyModifiers},
    execute, queue,
    style::{Color, Print, ResetColor, SetForegroundColor},
    terminal::{self, EnterAlternateScreen, LeaveAlternateScreen},
};
use rand::Rng;
use std::io::{self, Write};
use std::time::{Duration, Instant};

const PADDLE_HEIGHT: u16 = 5;
const BALL_SPEED: f32 = 0.75;
const PADDLE_SPEED: i16 = 1;
const POWERUP_SPAWN_CHANCE: f32 = 0.002;
const POWERUP_DURATION: Duration = Duration::from_secs(10);
const POWERUP_SIZE: u16 = 5;

#[derive(Clone, Copy, PartialEq)]
enum PowerUpType {
    DoublePaddle,
    CenterWall,
    TwoSmallWalls,
    BentPaddle,
    SplitBall,
}

struct PowerUp {
    x: u16,
    y: u16,
    ptype: PowerUpType,
}

struct ActivePowerUp {
    ptype: PowerUpType,
    player: u8,
    end_time: Instant,
}

struct Ball {
    x: f32,
    y: f32,
    vx: f32,
    vy: f32,
}

struct Game {
    width: u16,
    height: u16,
    p1_y: i16,
    p2_y: i16,
    p1_second_y: Option<i16>,
    p2_second_y: Option<i16>,
    p1_bent: bool,
    p2_bent: bool,
    balls: Vec<Ball>,
    p1_score: u16,
    p2_score: u16,
    powerups: Vec<PowerUp>,
    active_powerups: Vec<ActivePowerUp>,
    center_wall: bool,
    two_small_walls: bool,
    last_frame: Instant,
    buffer: Vec<Vec<char>>,
    color_buffer: Vec<Vec<Color>>,
}

impl Game {
    fn new(width: u16, height: u16) -> Self {
        let mut game = Game {
            width,
            height,
            p1_y: (height / 2) as i16,
            p2_y: (height / 2) as i16,
            p1_second_y: None,
            p2_second_y: None,
            p1_bent: false,
            p2_bent: false,
            balls: vec![Ball {
                x: (width / 2) as f32,
                y: (height / 2) as f32,
                vx: BALL_SPEED,
                vy: BALL_SPEED * 0.5,
            }],
            p1_score: 0,
            p2_score: 0,
            powerups: Vec::new(),
            active_powerups: Vec::new(),
            center_wall: false,
            two_small_walls: false,
            last_frame: Instant::now(),
            buffer: vec![vec![' '; width as usize]; height as usize],
            color_buffer: vec![vec![Color::White; width as usize]; height as usize],
        };
        game.reset_ball();
        game
    }

    fn reset_ball(&mut self) {
        self.balls.clear();
        let mut rng = rand::thread_rng();
        let vx = if rng.gen_bool(0.5) {
            BALL_SPEED
        } else {
            -BALL_SPEED
        };
        let vy = rng.gen_range(-BALL_SPEED..BALL_SPEED);
        self.balls.push(Ball {
            x: (self.width / 2) as f32,
            y: (self.height / 2) as f32,
            vx,
            vy,
        });
    }

    fn update(&mut self, dt: f32) {
        // Spawn powerups
        let mut rng = rand::thread_rng();
        if rng.gen::<f32>() < POWERUP_SPAWN_CHANCE && self.powerups.len() < 2 {
            let powerup_types = [
                PowerUpType::DoublePaddle,
                PowerUpType::CenterWall,
                PowerUpType::TwoSmallWalls,
                PowerUpType::BentPaddle,
                PowerUpType::SplitBall,
            ];
            self.powerups.push(PowerUp {
                x: rng.gen_range(self.width / 4..3 * self.width / 4),
                y: rng.gen_range(2..self.height - 2),
                ptype: powerup_types[rng.gen_range(0..powerup_types.len())],
            });
        }

        // Extract data needed for collision checks
        let width = self.width;
        let height = self.height;
        let p1_y = self.p1_y;
        let p2_y = self.p2_y;
        let p1_second_y = self.p1_second_y;
        let p2_second_y = self.p2_second_y;
        let p1_bent = self.p1_bent;
        let p2_bent = self.p2_bent;
        let center_wall = self.center_wall;
        let two_small_walls = self.two_small_walls;

        // Update balls
        let mut new_balls = Vec::new();
        let mut scored = false;
        let mut score_player = 0;

        for ball in &mut self.balls {
            ball.x += ball.vx * dt * 60.0;
            ball.y += ball.vy * dt * 60.0;

            // Top/bottom collision
            if ball.y <= 0.0 || ball.y >= (height - 1) as f32 {
                ball.vy = -ball.vy;
                ball.y = ball.y.clamp(0.0, (height - 1) as f32);
            }

            // Check center wall collision
            if center_wall {
                let wall_x = width / 2;
                if (ball.x as u16) == wall_x && ball.vx.abs() > 0.0 {
                    ball.vx = -ball.vx;
                }
            }

            // Check two small walls collision
            if two_small_walls {
                let wall_x = width / 2;
                let wall1_start = height / 4;
                let wall1_end = wall1_start + height / 6;
                let wall2_start = 3 * height / 4 - height / 6;
                let wall2_end = 3 * height / 4;

                if (ball.x as u16) == wall_x {
                    let by = ball.y as u16;
                    if (by >= wall1_start && by < wall1_end)
                        || (by >= wall2_start && by < wall2_end)
                    {
                        ball.vx = -ball.vx;
                    }
                }
            }

            // P1 paddle collision
            let hit_p1 = Self::check_paddle_collision_static(ball, 2, p1_y, p1_bent)
                || p1_second_y
                    .map(|y| Self::check_paddle_collision_static(ball, 2, y, false))
                    .unwrap_or(false);

            if hit_p1 && ball.vx < 0.0 {
                ball.vx = -ball.vx * 1.05;
                let paddle_center = p1_y as f32 + PADDLE_HEIGHT as f32 / 2.0;
                ball.vy = (ball.y - paddle_center) * 0.15;
            }

            // P2 paddle collision
            let hit_p2 = Self::check_paddle_collision_static(ball, width - 3, p2_y, p2_bent)
                || p2_second_y
                    .map(|y| Self::check_paddle_collision_static(ball, width - 3, y, false))
                    .unwrap_or(false);

            if hit_p2 && ball.vx > 0.0 {
                ball.vx = -ball.vx * 1.05;
                let paddle_center = p2_y as f32 + PADDLE_HEIGHT as f32 / 2.0;
                ball.vy = (ball.y - paddle_center) * 0.15;
            }

            // Scoring
            if ball.x <= 0.0 {
                scored = true;
                score_player = 2;
            } else if ball.x >= (width - 1) as f32 {
                scored = true;
                score_player = 1;
            }

            // Clamp ball speed
            ball.vx = ball.vx.clamp(-1.0, 1.0);
            ball.vy = ball.vy.clamp(-0.8, 0.8);
        }

        // Collect ball positions for powerup collision check
        let ball_positions: Vec<(f32, f32)> = self.balls.iter().map(|b| (b.x, b.y)).collect();
        
        // Now handle powerup collisions with mutable access
        for (ball_x, ball_y) in ball_positions {
            let bx = ball_x as u16;
            let by = ball_y as u16;
            let player = if bx < self.width / 2 { 1 } else { 2 };

            self.powerups.retain(|p| {
                // Check collision with 3x3 powerup area
                let hit = (p.x as i16 - bx as i16).abs() <= (POWERUP_SIZE / 2) as i16 
                       && (p.y as i16 - by as i16).abs() <= (POWERUP_SIZE / 2) as i16;
                
                if hit {
                    match p.ptype {
                        PowerUpType::SplitBall => {
                            // Split into 3 balls - use the original ball data
                            let mut rng = rand::thread_rng();
                            for _ in 0..2 {
                                new_balls.push(Ball {
                                    x: ball_x,
                                    y: ball_y,
                                    vx: if bx < self.width / 2 { BALL_SPEED } else { -BALL_SPEED },
                                    vy: rng.gen_range(-BALL_SPEED..BALL_SPEED),
                                });
                            }
                        }
                        _ => {
                            self.active_powerups.push(ActivePowerUp {
                                ptype: p.ptype,
                                player,
                                end_time: Instant::now() + POWERUP_DURATION,
                            });
                        }
                    }
                    false
                } else {
                    true
                }
            });
        }

        self.balls.append(&mut new_balls);

        if scored {
            if score_player == 1 {
                self.p1_score += 1;
            } else {
                self.p2_score += 1;
            }
            self.reset_ball();
            self.center_wall = false;
            self.two_small_walls = false;
        }

        // Update active powerups
        let now = Instant::now();
        self.active_powerups.retain(|p| now < p.end_time);

        // Reset powerup effects
        self.p1_second_y = None;
        self.p2_second_y = None;
        self.p1_bent = false;
        self.p2_bent = false;
        self.center_wall = false;
        self.two_small_walls = false;

        // Apply active powerup effects
        for powerup in &self.active_powerups {
            match powerup.ptype {
                PowerUpType::DoublePaddle => {
                    if powerup.player == 1 {
                        self.p1_second_y = Some(self.p1_y + PADDLE_HEIGHT as i16 + 2);
                    } else {
                        self.p2_second_y = Some(self.p2_y + PADDLE_HEIGHT as i16 + 2);
                    }
                }
                PowerUpType::BentPaddle => {
                    if powerup.player == 1 {
                        self.p1_bent = true;
                    } else {
                        self.p2_bent = true;
                    }
                }
                PowerUpType::CenterWall => {
                    self.center_wall = true;
                }
                PowerUpType::TwoSmallWalls => {
                    self.two_small_walls = true;
                }
                _ => {}
            }
        }
    }

    fn check_paddle_collision_static(ball: &Ball, paddle_x: u16, paddle_y: i16, bent: bool) -> bool {
        let bx = ball.x as u16;
        let by = ball.y as u16;

        if bent {
            // Bent paddle shape: <>
            for i in 0..PADDLE_HEIGHT {
                let py = (paddle_y + i as i16) as u16;
                if by == py {
                    let offset = if i < PADDLE_HEIGHT / 2 { i } else { PADDLE_HEIGHT - i - 1 };
                    if bx == paddle_x + offset as u16 {
                        return true;
                    }
                }
            }
        } else {
            // Normal paddle
            if bx == paddle_x && by >= paddle_y as u16 && by < (paddle_y + PADDLE_HEIGHT as i16) as u16 {
                return true;
            }
        }
        false
    }

    fn move_paddle(&mut self, player: u8, direction: i16) {
        let paddle_y = if player == 1 {
            &mut self.p1_y
        } else {
            &mut self.p2_y
        };

        *paddle_y += direction * PADDLE_SPEED;
        *paddle_y = (*paddle_y).clamp(0, self.height as i16 - PADDLE_HEIGHT as i16);
    }

    fn render(&mut self, stdout: &mut io::Stdout) -> io::Result<()> {
        // Clear buffers
        for row in &mut self.buffer {
            row.fill(' ');
        }
        for row in &mut self.color_buffer {
            row.fill(Color::White);
        }

        // Draw borders
        for x in 0..self.width {
            self.buffer[0][x as usize] = '─';
            self.buffer[(self.height - 1) as usize][x as usize] = '─';
        }

        // Draw center line
        for y in 0..self.height {
            if y % 2 == 0 {
                self.buffer[y as usize][(self.width / 2) as usize] = '┊';
                self.color_buffer[y as usize][(self.width / 2) as usize] = Color::DarkGrey;
            }
        }

        // Draw center wall
        if self.center_wall {
            for y in 1..(self.height - 1) {
                self.buffer[y as usize][(self.width / 2) as usize] = '█';
                self.color_buffer[y as usize][(self.width / 2) as usize] = Color::Yellow;
            }
        }

        // Draw two small walls
        if self.two_small_walls {
            let wall_x = (self.width / 2) as usize;
            let wall1_start = self.height / 4;
            let wall1_end = wall1_start + self.height / 6;
            let wall2_start = 3 * self.height / 4 - self.height / 6;
            let wall2_end = 3 * self.height / 4;

            for y in wall1_start..wall1_end {
                self.buffer[y as usize][wall_x] = '█';
                self.color_buffer[y as usize][wall_x] = Color::Cyan;
            }
            for y in wall2_start..wall2_end {
                self.buffer[y as usize][wall_x] = '█';
                self.color_buffer[y as usize][wall_x] = Color::Cyan;
            }
        }

        // Draw P1 paddle
        self.draw_paddle(2, self.p1_y, self.p1_bent, Color::Blue);
        if let Some(y) = self.p1_second_y {
            self.draw_paddle(2, y, false, Color::Cyan);
        }

        // Draw P2 paddle
        self.draw_paddle(self.width - 3, self.p2_y, self.p2_bent, Color::Red);
        if let Some(y) = self.p2_second_y {
            self.draw_paddle(self.width - 3, y, false, Color::Magenta);
        }

        // Draw balls
        for ball in &self.balls {
            let x = ball.x as usize;
            let y = ball.y as usize;
            if y < self.height as usize && x < self.width as usize {
                self.buffer[y][x] = '●';
                self.color_buffer[y][x] = Color::White;
            }
        }

        // Draw powerups (3x3 size)
        for powerup in &self.powerups {
            let symbol = match powerup.ptype {
                PowerUpType::DoublePaddle => '║',
                PowerUpType::CenterWall => '█',
                PowerUpType::TwoSmallWalls => '▓',
                PowerUpType::BentPaddle => '⟨',
                PowerUpType::SplitBall => '✦',
            };
            let color = match powerup.ptype {
                PowerUpType::DoublePaddle => Color::Cyan,
                PowerUpType::CenterWall => Color::Yellow,
                PowerUpType::TwoSmallWalls => Color::Magenta,
                PowerUpType::BentPaddle => Color::Green,
                PowerUpType::SplitBall => Color::White,
            };
            
            // Draw 3x3 powerup
            for dy in -(POWERUP_SIZE as i16 / 2)..=(POWERUP_SIZE as i16 / 2) {
                for dx in -(POWERUP_SIZE as i16 / 2)..=(POWERUP_SIZE as i16 / 2) {
                    let px = (powerup.x as i16 + dx) as usize;
                    let py = (powerup.y as i16 + dy) as usize;
                    if py < self.height as usize && px < self.width as usize {
                        self.buffer[py][px] = symbol;
                        self.color_buffer[py][px] = color;
                    }
                }
            }
        }

        // Build complete frame in a single string buffer before outputting
        // This prevents tearing and ensures atomic screen updates
        let mut frame_buffer = String::with_capacity((self.width as usize + 10) * self.height as usize);
        
        for y in 0..self.height {
            frame_buffer.push_str(&format!("\x1b[{};{}H", y + 1, 1)); // Move to position
            let mut current_color = Color::White;
            for x in 0..self.width {
                let ch = self.buffer[y as usize][x as usize];
                let color = self.color_buffer[y as usize][x as usize];
                if color != current_color {
                    frame_buffer.push_str(&Self::color_to_ansi(color));
                    current_color = color;
                }
                frame_buffer.push(ch);
            }
        }

        // Draw score
        frame_buffer.push_str(&format!(
            "\x1b[{};{}H\x1b[37mP1: {}  P2: {}\x1b[0m",
            1,
            self.width / 2 - 9,
            self.p1_score,
            self.p2_score
        ));

        // Write entire frame at once
        write!(stdout, "{}", frame_buffer)?;
        stdout.flush()?;

        Ok(())
    }

    fn color_to_ansi(color: Color) -> String {
        match color {
            Color::Black => "\x1b[30m".to_string(),
            Color::DarkGrey => "\x1b[90m".to_string(),
            Color::Red => "\x1b[31m".to_string(),
            Color::DarkRed => "\x1b[91m".to_string(),
            Color::Green => "\x1b[32m".to_string(),
            Color::DarkGreen => "\x1b[92m".to_string(),
            Color::Yellow => "\x1b[33m".to_string(),
            Color::DarkYellow => "\x1b[93m".to_string(),
            Color::Blue => "\x1b[34m".to_string(),
            Color::DarkBlue => "\x1b[94m".to_string(),
            Color::Magenta => "\x1b[35m".to_string(),
            Color::DarkMagenta => "\x1b[95m".to_string(),
            Color::Cyan => "\x1b[36m".to_string(),
            Color::DarkCyan => "\x1b[96m".to_string(),
            Color::White => "\x1b[37m".to_string(),
            Color::Grey => "\x1b[97m".to_string(),
            _ => "\x1b[37m".to_string(),
        }
    }

    fn draw_paddle(&mut self, x: u16, y: i16, bent: bool, color: Color) {
        if bent {
            // Bent paddle: <>
            for i in 0..PADDLE_HEIGHT {
                let py = y + i as i16;
                if py >= 0 && py < self.height as i16 {
                    let offset = if i < PADDLE_HEIGHT / 2 { i } else { PADDLE_HEIGHT - i - 1 };
                    let px = x + offset as u16;
                    if px < self.width {
                        self.buffer[py as usize][px as usize] = '█';
                        self.color_buffer[py as usize][px as usize] = color;
                    }
                }
            }
        } else {
            // Normal paddle
            for i in 0..PADDLE_HEIGHT {
                let py = y + i as i16;
                if py >= 0 && py < self.height as i16 {
                    self.buffer[py as usize][x as usize] = '█';
                    self.color_buffer[py as usize][x as usize] = color;
                }
            }
        }
    }
}

fn main() -> io::Result<()> {
    let mut stdout = io::stdout();

    // Setup terminal
    execute!(stdout, EnterAlternateScreen, Hide)?;
    terminal::enable_raw_mode()?;

    let (width, height) = terminal::size()?;
    let mut game = Game::new(width, height.saturating_sub(1));

    let mut p1_up = false;
    let mut p1_down = false;
    let mut p2_up = false;
    let mut p2_down = false;
    let mut running = true;

    // Game loop
    while running {
        let now = Instant::now();
        let dt = now.duration_since(game.last_frame).as_secs_f32();
        game.last_frame = now;

        // Handle input (non-blocking)
        while event::poll(Duration::from_millis(0))? {
            let event = event::read()?;
            match event {
                Event::Key(KeyEvent {
                    code,
                    modifiers,
                    kind: event::KeyEventKind::Press,
                    ..
                }) => {
                    match code {
                        KeyCode::Char('a') | KeyCode::Char('A') => p1_up = true,
                        KeyCode::Char('d') | KeyCode::Char('D') => p1_down = true,
                        KeyCode::Char('4') => p2_up = true,
                        KeyCode::Char('6') => p2_down = true,
                        KeyCode::Char('q') | KeyCode::Char('Q') => {
                            if modifiers.contains(KeyModifiers::CONTROL) {
                                running = false;
                            }
                        }
                        KeyCode::Esc => running = false,
                        _ => {}
                    }
                }
                Event::Key(KeyEvent {
                    code,
                    kind: event::KeyEventKind::Release,
                    ..
                }) => {
                    match code {
                        KeyCode::Char('a') | KeyCode::Char('A') => p1_up = false,
                        KeyCode::Char('d') | KeyCode::Char('D') => p1_down = false,
                        KeyCode::Char('4') => p2_up = false,
                        KeyCode::Char('6') => p2_down = false,
                        _ => {}
                    }
                }
                _ => {}
            }
        }

        // Update paddle positions
        if p1_up {
            game.move_paddle(1, -1);
        }
        if p1_down {
            game.move_paddle(1, 1);
        }
        if p2_up {
            game.move_paddle(2, -1);
        }
        if p2_down {
            game.move_paddle(2, 1);
        }

        // Update game state
        game.update(dt);

        // Render
        game.render(&mut stdout)?;

        // Cap framerate to ~60 FPS
        std::thread::sleep(Duration::from_millis(16));
    }

    // Cleanup
    execute!(stdout, LeaveAlternateScreen, Show)?;
    terminal::disable_raw_mode()?;

    Ok(())
}