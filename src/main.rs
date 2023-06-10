use macroquad::prelude::*;

const WIDTH: i32 = 800;
const HEIGHT: i32 = 800;

const BOARD_WIDTH: usize = 50;
const BOARD_HEIGHT: usize = 50;

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

#[derive(Clone, Copy)]
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

    loop {
        clear_background(BLACK);

        // sets the new board state into the board image
        for (y, row) in board.iter().enumerate() {
            for (x, cell) in row.iter().enumerate() {
                board_image.set_pixel(x as u32, y as u32, get_cell_color(cell));
            }
        }
        // uploads the new board state to the texture
        board_texture.update(&board_image);
        
        draw_texture_ex(
            board_texture,
            0.0,
            0.0,
            WHITE,
            DrawTextureParams {
                dest_size: Some(vec2(WIDTH as f32, HEIGHT as f32)),
                ..Default::default()
            }
        );

        next_frame().await
    }
}

fn get_cell_color(cell: &Cell) -> Color {
    match cell {
        Cell::Empty => BLACK,
        Cell::Head => BLUE,
        Cell::Tail => RED,
        Cell::Conductor => YELLOW,
    }
}
