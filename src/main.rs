use std::{error::Error, ops::Bound};

use chess_library::Board;
use ggez::{
    Context, GameError, GameResult, event,
    glam::*,
    graphics::{self, Color, DrawParam},
    winit::event::MouseButton,
};

const BOARD_X: u32 = 100;
const BOARD_Y: u32 = 100;
const SQUARE_W: u32 = 100;
const SQUARE_H: u32 = 100;

struct MainState {
    board: Board,
    piece_assets: [graphics::Image; 12],
    clicked_piece: Option<usize>,
    legal_moves: [u64; 64],
}

impl MainState {
    fn new(ctx: &mut Context) -> GameResult<MainState> {
        let initial_boards: [u64; 12] = [0; 12];
        let mut board = Board {
            boards: initial_boards,
            w_enpesant: 0,
            b_enpesant: 0,

            w_l_rook_moved: false,
            w_r_rook_moved: false,
            w_k_moved: false,

            b_l_rook_moved: false,
            b_r_rook_moved: false,
            b_k_moved: false,

            white_turn: true,
        };
        chess_library::Board::set_standard_board(&mut board);
        let piece_assets = [
            graphics::Image::from_path(ctx, "/assets/w_Pawn.png").expect("Fuck image not found"),
            graphics::Image::from_path(ctx, "/assets/w_Rook.png").expect("Fuck image not found"),
            graphics::Image::from_path(ctx, "/assets/w_Knight.png").expect("Fuck image not found"),
            graphics::Image::from_path(ctx, "/assets/w_Bishop.png").expect("Fuck image not found"),
            graphics::Image::from_path(ctx, "/assets/w_King.png").expect("Fuck image not found"),
            graphics::Image::from_path(ctx, "/assets/w_Queen.png").expect("Fuck image not found"),
            graphics::Image::from_path(ctx, "/assets/b_Pawn.png").expect("Fuck image not found"),
            graphics::Image::from_path(ctx, "/assets/b_Rook.png").expect("Fuck image not found"),
            graphics::Image::from_path(ctx, "/assets/b_Knight.png").expect("Fuck image not found"),
            graphics::Image::from_path(ctx, "/assets/b_Bishop.png").expect("Fuck image not found"),
            graphics::Image::from_path(ctx, "/assets/b_King.png").expect("Fuck image not found"),
            graphics::Image::from_path(ctx, "/assets/b_Queen.png").expect("Fuck image not found"),
        ];

        let legal_moves = chess_library::Board::get_all_legal_moves(&board);

        Ok(MainState {
            board,
            piece_assets,
            clicked_piece: None,
            legal_moves,
        })
    }
}

impl event::EventHandler for MainState {
    fn update(&mut self, _ctx: &mut Context) -> GameResult {
        Ok(())
    }

    fn draw(&mut self, ctx: &mut Context) -> GameResult {
        let mut canvas =
            graphics::Canvas::from_frame(ctx, graphics::Color::from([0.1, 0.2, 0.3, 1.0]));

        // canvas.draw(&self.circle, Vec2::new(self.pos_x, 380.0));

        draw_board(
            ctx,
            &mut canvas,
            self.board,
            &self.piece_assets,
            &self.clicked_piece,
            self.legal_moves,
        );
        canvas.finish(ctx)?;

        Ok(())
    }
    fn mouse_button_down_event(
        &mut self,
        _ctx: &mut Context,
        button: MouseButton,
        x: f32,
        y: f32,
    ) -> Result<(), GameError> {
        if !matches!(button, MouseButton::Left) {
            return Ok(());
        }

        let square_x = (x as u32 - BOARD_X) / 100;
        if square_x > 7 {
            return Ok(());
        }

        let square_y = (y as u32 - BOARD_Y) / 100;
        if square_y > 7 {
            return Ok(());
        }

        let square_index = square_x as usize + square_y as usize * 8;

        if !self.clicked_piece.is_some() {
            let piece_type =
                chess_library::Board::piece_type_on_position(&self.board, square_index);
            if piece_type == -1 {
                return Ok(());
            }
        }

        if self.clicked_piece.is_some()
            && (self.legal_moves[self.clicked_piece.expect("???")] >> square_index & 1 == 1)
        {
            let valid_move = chess_library::Board::move_piece(
                &mut self.board,
                self.clicked_piece.expect("???"),
                square_index as u64,
                None,
            );
            if valid_move {
                self.legal_moves = chess_library::Board::get_all_legal_moves(&self.board);
            }
            self.clicked_piece = None;
            return Ok(());
        }

        self.clicked_piece = Some(square_index);
        return Ok(());
    }
}

pub fn main() -> GameResult {
    let cb = ggez::ContextBuilder::new("chess", "elbjork-samolss");
    let (mut ctx, event_loop) = cb.build()?;
    ctx.fs.print_all();
    let state = MainState::new(&mut ctx)?;
    event::run(ctx, event_loop, state)
}

fn draw_board(
    ctx: &mut Context,
    canvas: &mut graphics::Canvas,
    board: Board,
    piece_assets: &[graphics::Image; 12],
    clicked_square: &Option<usize>,
    legal_moves: [u64; 64],
) {
    for i in 0..64 {
        let bounds = graphics::Rect {
            x: (BOARD_X as usize + SQUARE_W as usize * (i % 8)) as f32,
            y: (BOARD_Y + SQUARE_H * (i as u32 / 8)) as f32,
            w: SQUARE_W as f32,
            h: SQUARE_H as f32,
        };

        let color = if clicked_square == &Some(i) {
            Color::GREEN
        } else if clicked_square.is_some()
            && legal_moves[clicked_square.expect("???")] >> i & 1 == 1
        {
            Color::RED
        } else if i % 2 == (i / 8 % 2) {
            Color::WHITE
        } else {
            Color::BLUE
        };

        let square = graphics::Mesh::new_rectangle(ctx, graphics::DrawMode::fill(), bounds, color)
            .expect("FFFFFFFUUUUUCKKKK");

        canvas.draw(&square, Vec2::new(0 as f32, 0 as f32));

        let piece_type = chess_library::Board::piece_type_on_position(&board, i);
        if piece_type >= 0 {
            let x = (BOARD_X + SQUARE_W * (i as u32 % 8)) as f32;
            let y = (BOARD_Y + SQUARE_H * (i as u32 / 8)) as f32;
            let draw_params = DrawParam::new().dest(vec2(x, y)).scale(vec2(0.9, 0.9));
            canvas.draw(&piece_assets[piece_type as usize], draw_params);
        }
    }
}
