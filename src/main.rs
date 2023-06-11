use macroquad::prelude::*;

/// screen width
const WIDTH: i32 = 800;
/// screen height
const HEIGHT: i32 = 800;

/// board width
const BOARD_WIDTH: usize = 200;
/// board height
const BOARD_HEIGHT: usize = 200;

/// FPS for the simulation
const FPS: f32 = 2.0;
/// target time in seconds 
/// between each simulation frame
const FPS_TIME: f32 = 1.0 / FPS;

fn window_conf() -> Conf {
    Conf {
        window_title: "Wireworld".to_owned(),
        window_width: WIDTH,
        window_height: HEIGHT,
        window_resizable: false,
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

#[macroquad::main(window_conf)]
async fn main() {
    let mut board: [[Cell; BOARD_WIDTH]; BOARD_HEIGHT] = [[Cell::Empty; BOARD_WIDTH]; BOARD_HEIGHT];

    let mut board_image = Image::gen_image_color(BOARD_WIDTH as u16, BOARD_HEIGHT as u16, BLACK);
    let board_texture = Texture2D::from_image(&board_image);
    board_texture.set_filter(FilterMode::Nearest);

    // scales the screen to the smaller dimension
    let mut scale = if WIDTH < HEIGHT {
        WIDTH as f32 / BOARD_WIDTH as f32
    } else {
        HEIGHT as f32 / BOARD_HEIGHT as f32
    };

    let mut offset_x = 0.0;
    let mut offset_y = 0.0;
    
    // keeps track of time passed since last simulation
    let mut elapsed = 0.0;

    let mut paused = false;

    loop {
        clear_background(BLACK);

        // sets the current board state into the board image
        for (y, row) in board.iter().enumerate() {
            for (x, cell) in row.iter().enumerate() {
                board_image.set_pixel(x as u32, y as u32, get_cell_color(cell));
            }
        }
        // uploads the new board state to the texture
        board_texture.update(&board_image);

        // update mouse position
        let (m_x, m_y) = mouse_position();

        // handle user input and check to make sure the 
        // mouse isnt on the edge, which can cause issues
        if (m_x >= 0.0 && m_y >= 0.0) && (m_x < WIDTH as f32 && m_y < HEIGHT as f32) {
            let (board_x, board_y) = screen_to_board(m_x, m_y, scale, offset_x, offset_y);
            if board_x < BOARD_WIDTH && board_y < BOARD_HEIGHT {
                if is_mouse_button_down(MouseButton::Left) {
                    board[board_y][board_x] = Cell::Conductor;
                } else if is_mouse_button_down(MouseButton::Right) {
                    board[board_y][board_x] = Cell::Empty;
                } else if is_mouse_button_down(MouseButton::Middle) {
                    board[board_y][board_x] = Cell::Head;
                }
            }
        }

        // handles pan and zoom operations

        if is_key_down(KeyCode::A) {
            offset_x -= 1.0;
        } else if is_key_down(KeyCode::D) {
            offset_x += 1.0;
        }
        if is_key_down(KeyCode::W) {
            offset_y -= 1.0;
        } else if is_key_down(KeyCode::S) {
            offset_y += 1.0;
        }
        
        if is_key_down(KeyCode::Q) {
            scale *= 0.99;
        } else if is_key_down(KeyCode::E) {
            scale *= 1.01;
        }


        if is_key_pressed(KeyCode::Space) {
            paused = !paused;
        }
        
        // only updates the board the request amount per second
        if elapsed >= FPS_TIME && !paused {
            board = next_generation(&board);
            elapsed = 0.0;
        }

        draw_texture_ex(
            board_texture,
            -offset_x * scale,
            -offset_y * scale,
            WHITE,
            DrawTextureParams {
                dest_size: Some(vec2(BOARD_WIDTH as f32 * scale, BOARD_HEIGHT as f32 * scale)),
                ..Default::default()
            }
        );

        elapsed += get_frame_time();

        next_frame().await
    }
}

/// takes a cell and returns the color
/// that visually represents it
fn get_cell_color(cell: &Cell) -> Color {
    match cell {
        Cell::Empty => BLACK,
        Cell::Head => BLUE,
        Cell::Tail => RED,
        Cell::Conductor => YELLOW,
    }
}

/// takes an x and y in screen space
/// and converts it to board space
fn screen_to_board(x: f32, y: f32, scale: f32, x_off: f32, y_off: f32) -> (usize, usize) {
    let scaled_x = x / scale + x_off;
    let scaled_y = y / scale + y_off;

    if scaled_x < 0.0 || scaled_y < 0.0 {
        return (BOARD_WIDTH, BOARD_HEIGHT);
    }

    (scaled_x as usize, scaled_y as usize)
}

/// returns the next generation of any given board state
fn next_generation(board: &[[Cell; BOARD_WIDTH]; BOARD_HEIGHT]) -> [[Cell; BOARD_WIDTH]; BOARD_HEIGHT] {
    let mut next_board = [[Cell::Empty; BOARD_WIDTH]; BOARD_HEIGHT];

    for y in 0..board.len() {
        for x in 0..board[0].len() {
            next_board[y][x] = next_state(board, x, y);
        }
    }

    next_board
}

/// returns the next state of a cell in any given position
fn next_state(board: &[[Cell; BOARD_WIDTH]; BOARD_HEIGHT], x: usize, y: usize) -> Cell {
    let cell = board[y][x];

    match cell {
        Cell::Empty => Cell::Empty,
        Cell::Head => Cell::Tail,
        Cell::Tail => Cell::Conductor,
        Cell::Conductor => {

            let mut neighbour_head = 0;

            // loops through the 8 surrounding cells to see how many are head cells.
            // has checks in place for edge cells to avoid leaving the bounds.
            for c_y in (if y == 0 {0} else {y - 1})..=(if y > BOARD_HEIGHT - 2 {BOARD_HEIGHT - 1} else {y + 1}) {
                for c_x in (if x == 0 {0} else {x - 1})..=(if x > BOARD_WIDTH - 2 {BOARD_WIDTH - 1} else {x + 1}) {
                    if board[c_y][c_x] == Cell::Head {
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
