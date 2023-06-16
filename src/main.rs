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

        let (before_x, before_y) = self.screen_to_board(screen_width() / 2.0, screen_height() / 2.0);
        // let before_x = (screen_width() / 2.0) / self.scale + self.x_offset;
        // let before_y = (screen_height() / 2.0) / self.scale + self.y_offset;

        if is_key_down(KeyCode::Q) {
            self.scale *= 0.99;
        } else if is_key_down(KeyCode::E) {
            self.scale *= 1.01;
        }

        let (after_x, after_y) = self.screen_to_board(screen_width() / 2.0, screen_height() / 2.0);
        // let after_x = (screen_width() / 2.0) / self.scale + self.x_offset;
        // let after_y = (screen_height() / 2.0) / self.scale + self.y_offset;

        self.x_offset += before_x - after_x;
        self.y_offset += before_y - after_y;
    }

    /// handles any input from the mouse
    fn handle_mouse_input(&mut self) {
        let (m_x, m_y) = mouse_position();

        if is_mouse_button_down(MouseButton::Left) {
            let (board_x, board_y) = self.screen_to_board_rounded(m_x, m_y);
            if board_x >= 0 && board_x < self.width as isize && board_y >= 0 && board_y < self.height as isize {
                self.insert_cell(Cell::Conductor, board_x as usize, board_y as usize);
            }
        } else if is_mouse_button_down(MouseButton::Right) {
            let (board_x, board_y) = self.screen_to_board_rounded(m_x, m_y);
            if board_x >= 0 && board_x < self.width as isize && board_y >= 0 && board_y < self.height as isize {
                self.insert_cell(Cell::Empty, board_x as usize, board_y as usize);
            }
        } else if is_mouse_button_down(MouseButton::Middle) {
            let (board_x, board_y) = self.screen_to_board_rounded(m_x, m_y);
            if board_x >= 0 && board_x < self.width as isize && board_y >= 0 && board_y < self.height as isize {
                self.insert_cell(Cell::Head, board_x as usize, board_y as usize);
            }
        }
    }

    fn insert_cell(&mut self, cell: Cell, x: usize, y: usize) {
        self.board[y][x] = cell;
        self.updates.insert((x, y));
        for n_x in ((x as isize - 1).max(0) as usize)..=(x + 1).min(self.width - 1) {
            for n_y in ((y as isize - 1).max(0) as usize)..=(y + 1).min(self.height - 1) {
                self.updates.insert((n_x, n_y));
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
    fn screen_to_board(&self, x: f32, y: f32) -> (f32, f32) {
        let board_x = x / self.scale + self.x_offset;
        let board_y = y / self.scale + self.y_offset;

        (board_x, board_y)
    }

    /// helper function to make converting to integer board space easier
    fn screen_to_board_rounded(&self, x: f32, y: f32) -> (isize, isize) {
        let (board_x, board_y) = self.screen_to_board(x, y);

        (board_x as isize, board_y as isize)
    }

    fn board_to_screen(&self, x: usize, y: usize) -> (f32, f32) {
        let screen_x = (x as f32 - self.x_offset) * self.scale;
        let screen_y = (y as f32 - self.y_offset) * self.scale;

        (screen_x, screen_y)
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

        // gets the top-left and bottom-right of the screen in board space
        // and clamps them to the board
        let tl = {
            let (x, y) = self.screen_to_board_rounded(0.0, 0.0);
            (x.clamp(0, self.width as isize) as usize, y.clamp(0, self.height as isize) as usize)
        };

        let br = {
            let (x, y) = self.screen_to_board_rounded(screen_width() - 1.0, screen_height() - 1.0);
            (x.clamp(0, self.width as isize) as usize, y.clamp(0, self.height as isize) as usize)
        };
        
        // fades the grid out the farther you are zoomed out
        let grid_weight = (self.scale - 10.0).clamp(0.0, 1.0);

        if grid_weight > 0.0 {
            for i in tl.0..=br.0 {
                let top = self.board_to_screen(i, tl.1);
                let bottom = self.board_to_screen(i, (br.1 + 1).min(self.height));
                
                draw_line(top.0, top.1, bottom.0, bottom.1, grid_weight, GRAY);
            }
            
            for i in tl.1..=br.1 {
                let left = self.board_to_screen(tl.0, i);
                let right = self.board_to_screen((br.0 + 1).min(self.width), i);

                draw_line(left.0, left.1, right.0, right.1, grid_weight, GRAY);
            }
        }
        
    }

    /// updates the state of the world
    pub fn update(&mut self) {

        // loops through all of the updated pixels and uploads the changes to the image
        let mut changed = false;
        for pos in &self.updates {
            self.board_image.set_pixel(pos.0 as u32, pos.1 as u32, self.board[pos.1][pos.0].get_cell_color());
            changed = true;
        }
        // uploads the new board state to the texture only if the image changed
        if changed {
            self.board_texture.update(&self.board_image);
        }

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

    let mut world = Wireworld::new(500, 500);

    loop {
        clear_background(Color::new(0.05, 0.05, 0.05, 1.0));

        world.update();
        draw_text(format!("FPS: {}", get_fps()).as_str(), 10.0, 50.0, 50.0, WHITE);

        next_frame().await
    }
}
