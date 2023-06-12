use std::{time::Instant, collections::HashSet};

use macroquad::prelude::*;

/// FPS for the simulation
const FPS: f32 = 2.0;
/// target time in seconds 
/// between each simulation frame
const FPS_TIME: f32 = 1.0 / FPS;

fn window_conf() -> Conf {
    Conf {
        window_title: "Wireworld".to_owned(),
        window_width: 800,
        window_height: 800,
        window_resizable: true,
        fullscreen: false,
        ..Default::default()
    }
}

#[derive(Clone, Copy, PartialEq, Eq)]
enum Cell {
    Empty,
    Head,
    Tail,
    Conductor,
}

impl Cell {
    /// returns the color of the cell
    pub fn get_cell_color(&self) -> Color {
        match self {
            Cell::Empty => BLACK,
            Cell::Head => BLUE,
            Cell::Tail => RED,
            Cell::Conductor => YELLOW,
        }
    }
}

struct Wireworld {
    board: Vec<Vec<Cell>>,
    width: usize,
    height: usize,
    updates: HashSet<(usize, usize)>,

    x_offset: f32,
    y_offset: f32,
    scale: f32,

    paused: bool,

    board_image: Image,
    board_texture: Texture2D,

    elapsed: Instant,

}

impl Wireworld {
    pub fn new(width: usize, height: usize) -> Self {
        let board_image = Image::gen_image_color(width as u16, height as u16, BLACK);
        let board_texture = Texture2D::from_image(&board_image);
        board_texture.set_filter(FilterMode::Nearest);

        Self {
            board: vec![vec![Cell::Empty; width]; height],
            width,
            height,
            updates: HashSet::new(),
            x_offset: 0.0,
            y_offset: 0.0,
            scale: screen_width() / 30.0,
            paused: false,
            board_image,
            board_texture,
            elapsed: Instant::now(),
        }
    }

    /// calculates the next generation of the world
    fn next_generation(&mut self) {
        let mut next_board = self.board.clone();
        let mut next_updates = HashSet::new();

        for pos in &self.updates {
            let cell_next = self.next_state(pos.0, pos.1);
            if cell_next != next_board[pos.1][pos.0] {
                next_board[pos.1][pos.0] = cell_next;
                next_updates.insert(*pos);
                for x in ((pos.0 as isize - 1).max(0) as usize)..=(pos.0 + 1).min(self.width - 1) {
                    for y in ((pos.1 as isize - 1).max(0) as usize)..=(pos.1 + 1).min(self.height - 1) {
                        next_updates.insert((x, y));
                    }
                }
            }
        }

        self.board = next_board;
        self.updates = next_updates;
    }

    /// returns the next state of a cell in any given position
    fn next_state(&self, x: usize, y: usize) -> Cell {
        let cell = self.board[y][x];

        match cell {
            Cell::Empty => Cell::Empty,
            Cell::Head => Cell::Tail,
            Cell::Tail => Cell::Conductor,
            Cell::Conductor => {

                let mut neighbour_head = 0;

                // loops through the 8 surrounding cells to see how many are head cells.
                // has checks in place for edge cells to avoid leaving the bounds.
                for c_y in (if y == 0 {0} else {y - 1})..=(if y > self.height - 2 {self.height - 1} else {y + 1}) {
                    for c_x in (if x == 0 {0} else {x - 1})..=(if x > self.width - 2 {self.width - 1} else {x + 1}) {
                        if self.board[c_y][c_x] == Cell::Head {
                            neighbour_head += 1;
                        }
                    }
                }

                // will only become a head cell if there
                // are 1 or 2 surrounding head cells
                if neighbour_head == 1 || neighbour_head == 2 {
                    Cell::Head
                } else {
                    Cell::Conductor
                }
            }
        }
    }

    /// handles the input for panning and zooming
    fn handle_pan_and_zoom(&mut self) {
        if is_key_down(KeyCode::A) {
            self.x_offset -= 10.0 / self.scale;
        } else if is_key_down(KeyCode::D) {
            self.x_offset += 10.0 / self.scale;
        }
        if is_key_down(KeyCode::W) {
            self.y_offset -= 10.0 / self.scale;
        } else if is_key_down(KeyCode::S) {
            self.y_offset += 10.0 / self.scale;
        }

        if is_key_down(KeyCode::Q) {
            self.scale *= 0.99;
        } else if is_key_down(KeyCode::E) {
            self.scale *= 1.01;
        }
    }

    /// handles any input from the mouse
    fn handle_mouse_input(&mut self) {
        let (m_x, m_y) = mouse_position();

        if is_mouse_button_down(MouseButton::Left) {
            let (board_x, board_y) = self.screen_to_board(m_x, m_y);
            if board_x >= 0 && board_x < self.width as isize && board_y >= 0 && board_y < self.height as isize {
                self.board[board_y as usize][board_x as usize] = Cell::Conductor;
                self.updates.insert((board_x as usize, board_y as usize));
                for x in ((board_x - 1).max(0) as usize)..=((board_x + 1) as usize).min(self.width - 1) {
                    for y in ((board_y - 1).max(0) as usize)..=((board_y + 1) as usize).min(self.height - 1) {
                        self.updates.insert((x, y));
                    }
                }
            }
        } else if is_mouse_button_down(MouseButton::Right) {
            let (board_x, board_y) = self.screen_to_board(m_x, m_y);
            if board_x >= 0 && board_x < self.width as isize && board_y >= 0 && board_y < self.height as isize {
                self.board[board_y as usize][board_x as usize] = Cell::Empty;
                self.updates.insert((board_x as usize, board_y as usize));
                for x in ((board_x - 1).max(0) as usize)..=((board_x + 1) as usize).min(self.width - 1) {
                    for y in ((board_y - 1).max(0) as usize)..=((board_y + 1) as usize).min(self.height - 1) {
                        self.updates.insert((x, y));
                    }
                }
            }
        } else if is_mouse_button_down(MouseButton::Middle) {
            let (board_x, board_y) = self.screen_to_board(m_x, m_y);
            if board_x >= 0 && board_x < self.width as isize && board_y >= 0 && board_y < self.height as isize {
                self.board[board_y as usize][board_x as usize] = Cell::Head;
                self.updates.insert((board_x as usize, board_y as usize));
                for x in ((board_x - 1).max(0) as usize)..=((board_x + 1) as usize).min(self.width - 1) {
                    for y in ((board_y - 1).max(0) as usize)..=((board_y + 1) as usize).min(self.height - 1) {
                        self.updates.insert((x, y));
                    }
                }
            }
        }
    }

    /// handles all forms of input the user can give
    fn handle_input(&mut self) {
        self.handle_mouse_input();
        self.handle_pan_and_zoom();

        if is_key_pressed(KeyCode::Space) {
            self.paused = !self.paused;
        }
    }

    /// takes an x and y in screen space
    /// and converts it to board space
    fn screen_to_board(&self, x: f32, y: f32) -> (isize, isize) {
        let scaled_x = x / self.scale + self.x_offset;
        let scaled_y = y / self.scale + self.y_offset;

        (scaled_x as isize, scaled_y as isize)
    }

    /// draws the board and and grid
    fn draw_board(&mut self) {
        draw_texture_ex(
            self.board_texture,
            -self.x_offset * self.scale,
            -self.y_offset * self.scale,
            WHITE,
            DrawTextureParams {
                dest_size: Some(vec2(self.width as f32 * self.scale, self.height as f32 * self.scale)),
                ..Default::default()
            },
        );

        for i in (self.x_offset.clamp(0.0, self.width as f32) as usize)..=(((screen_width() - self.x_offset) * self.scale) as usize).min(self.width) {
            let x = (i as f32 - self.x_offset) * self.scale;
            let top_y = (-self.y_offset * self.scale).max(0.0);
            let bottom_y = ((self.height as f32 - self.y_offset) * self.scale).min(screen_height());
            
            draw_line(x, top_y, x, bottom_y, 0.5, GRAY);
        }
        
        for i in (self.y_offset.clamp(0.0, self.height as f32) as usize)..=(((screen_height() - self.y_offset) * self.scale) as usize).min(self.height) {
            let y = (i as f32 - self.y_offset) * self.scale;
            let left_x = (-self.x_offset * self.scale).max(0.0);
            let right_x = ((self.width as f32 - self.x_offset) * self.scale).min(screen_width());

            draw_line(left_x.round(), y.round(), right_x.round(), y.round(), 0.5, GRAY);
        }
    }

    /// updates the state of the world
    pub fn update(&mut self) {
        // sets the current board state into the board image
        for (y, row) in self.board.iter().enumerate() {
            for (x, cell) in row.iter().enumerate() {
                self.board_image.set_pixel(x as u32, y as u32, cell.get_cell_color());
            }
        }
        // uploads the new board state to the texture
        self.board_texture.update(&self.board_image);

        self.handle_input();

        if self.elapsed.elapsed().as_secs_f32() >= FPS_TIME && !self.paused {
            self.next_generation();
            self.elapsed = Instant::now();
        }

        self.draw_board();
    }
}

#[macroquad::main(window_conf)]
async fn main() {

    let mut world = Wireworld::new(200, 200);

    loop {
        clear_background(BLACK);

        world.update();
        draw_text(format!("FPS: {}", get_fps()).as_str(), 10.0, 50.0, 50.0, WHITE);

        next_frame().await
    }
}
