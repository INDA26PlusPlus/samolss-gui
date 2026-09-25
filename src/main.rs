use std::{error::Error, ops::Bound};

use chess_library::Board;
use ggez::{
    Context, GameError, GameResult, event,
    glam::*,
    graphics::{self, Color, DrawParam},
    winit::event::MouseButton,
};
use std::cmp;

const ASSET_SIDE: f32 = 128.0;
const POPUP_W: f32 = 400.0;
const POPUP_H: f32 = 400.0;

const BLACK_SQUARE_COLOR: graphics::Color = graphics::Color {
    r: 115.0 / 255.0,
    g: 149.0 / 255.0,
    b: 82.0 / 255.0,
    a: 1.0,
};

const WHITE_SQUARE_COLOR: graphics::Color = graphics::Color {
    r: 235.0 / 255.0,
    g: 236.0 / 255.0,
    b: 208.0 / 255.0,
    a: 1.0,
};
struct MainState {
    square_side: u32,
    board_x: u32,
    board_y: u32,

    board: Board,
    promoting: bool,
    promoting_square: usize,
    piece_assets: [graphics::Image; 12],
    clicked_piece: Option<usize>,
    legal_moves: [u64; 64],
    white_win: bool,
    black_win: bool,
    draw: bool,
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
            square_side: 0,
            board_x: 0,
            board_y: 0,
            board,
            piece_assets,
            clicked_piece: None,
            legal_moves,
            white_win: false,
            black_win: false,
            draw: false,
            promoting: false,
            promoting_square: 64,
        })
    }
    fn draw_board(&mut self, ctx: &mut Context, canvas: &mut graphics::Canvas) {
        let screen = canvas.screen_coordinates().expect("No screen?");
        let smallest_side = cmp::min(screen.w as u32, screen.h as u32);
        self.square_side = smallest_side / 10;
        self.board_x = (screen.w as u32 - 8 * self.square_side) / 2;
        self.board_y = (screen.h as u32 - 8 * self.square_side) / 2;

        let scalar = self.square_side as f32 / ASSET_SIDE;

        for i in 0..64 {
            let bounds = graphics::Rect {
                x: (self.board_x as usize + self.square_side as usize * (i % 8)) as f32,
                y: (self.board_y + self.square_side * (i as u32 / 8)) as f32,
                w: self.square_side as f32,
                h: self.square_side as f32,
            };

            let color = if self.clicked_piece == Some(i) {
                Color::GREEN
            } else if self.clicked_piece.is_some()
                && self.legal_moves[self.clicked_piece.expect("???")] >> i & 1 == 1
            {
                Color::RED
            } else if i % 2 == (i / 8 % 2) {
                WHITE_SQUARE_COLOR
            } else {
                BLACK_SQUARE_COLOR
            };

            let square =
                graphics::Mesh::new_rectangle(ctx, graphics::DrawMode::fill(), bounds, color)
                    .expect("FFFFFFFUUUUUCKKKK");

            canvas.draw(&square, Vec2::new(0 as f32, 0 as f32));

            let piece_type = chess_library::Board::piece_type_on_position(&self.board, i);
            if piece_type >= 0 {
                let x = (self.board_x + self.square_side * (i as u32 % 8)) as f32;
                let y = (self.board_y + self.square_side * (i as u32 / 8)) as f32;
                let draw_params = DrawParam::new()
                    .dest(vec2(x, y))
                    .scale(vec2(scalar, scalar));
                canvas.draw(&self.piece_assets[piece_type as usize], draw_params);
            }
        }
    }

    fn draw_hud(&mut self, ctx: &mut Context, canvas: &mut graphics::Canvas) {
        let screen = canvas.screen_coordinates().expect("No screen?");
        if self.white_win || self.black_win || self.draw {
            let bounds = graphics::Rect {
                x: (screen.w - POPUP_W) / 2.0,
                y: (screen.h - POPUP_H) / 2.0,
                w: POPUP_W,
                h: POPUP_H,
            };

            let color = Color::MAGENTA;
            let finish_text = if self.white_win {
                graphics::Text::new("White won!")
            } else if self.black_win {
                graphics::Text::new("Black won!")
            } else {
                graphics::Text::new("Draw")
            };

            let popup =
                graphics::Mesh::new_rectangle(ctx, graphics::DrawMode::fill(), bounds, color)
                    .expect("Fuck");

            canvas.draw(&popup, Vec2::new(0 as f32, 0 as f32));
            canvas.draw(
                &finish_text,
                DrawParam::new().dest(Vec2::new(
                    (screen.w - POPUP_W) / 2.0,
                    (screen.h - POPUP_H) / 2.0,
                )),
            )
        } else if self.promoting {
            let bounds = graphics::Rect {
                x: (screen.w - POPUP_W) / 2.0,
                y: (screen.h - POPUP_H) / 2.0,
                w: POPUP_W,
                h: POPUP_H,
            };

            let color = Color::MAGENTA;

            let promotable_asset = if self.board.white_turn {
                [5, 1, 2, 3]
            } else {
                [11, 7, 8, 9]
            };

            // let promotable_type if self.board.white_turn {
            //     [chess_library::Board::W_QUEENS]
            // }v

            let popup =
                graphics::Mesh::new_rectangle(ctx, graphics::DrawMode::fill(), bounds, color)
                    .expect("Fuck");

            canvas.draw(&popup, Vec2::new(0 as f32, 0 as f32));

            for i in 0..4 {
                let x =
                    ((screen.w - POPUP_W) as u32 / 2 + POPUP_W as u32 / 2 * (i as u32 % 2)) as f32;
                let y =
                    ((screen.h - POPUP_H) as u32 / 2 + POPUP_H as u32 / 2 * (i as u32 / 2)) as f32;
                let scalar = (POPUP_H / 2.0) / ASSET_SIDE;
                let draw_params = DrawParam::new()
                    .dest(vec2(x, y))
                    .scale(vec2(scalar, scalar));

                canvas.draw(&self.piece_assets[promotable_asset[i]], draw_params);
            }
        }
    }
}

impl event::EventHandler for MainState {
    fn update(&mut self, _ctx: &mut Context) -> GameResult {
        Ok(())
    }

    fn draw(&mut self, ctx: &mut Context) -> GameResult {
        let mut canvas =
            graphics::Canvas::from_frame(ctx, graphics::Color::from([0.1, 0.2, 0.3, 1.0]));

        let screen = canvas.screen_coordinates().expect("No screeen!");

        self.draw_board(ctx, &mut canvas);
        self.draw_hud(ctx, &mut canvas);
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

        if self.promoting {
            let screen = _ctx.gfx.drawable_size();
            let popup_x = (screen.0 - POPUP_W) / 2.0;
            let popup_y = (screen.1 - POPUP_H) / 2.0;
            let promotable_asset = if self.board.white_turn {
                [5, 1, 2, 3]
            } else {
                [11, 7, 8, 9]
            };

            let piece_index_clicked = ((x as u32 - popup_x as u32) / (POPUP_W as u32 / 2)
                + 2 * ((y as u32 - popup_y as u32) / (POPUP_H as u32 / 2)))
                as usize;

            if self.clicked_piece.is_some() {
                let valid_move = chess_library::Board::move_piece(
                    &mut self.board,
                    self.clicked_piece.expect("???"),
                    self.promoting_square as u64,
                    Some(promotable_asset[piece_index_clicked] as usize),
                );
                if valid_move {
                    self.legal_moves = chess_library::Board::get_all_legal_moves(&self.board);
                }
                self.clicked_piece = None;
                self.promoting = false;
                self.promoting_square = 64;

                let is_mate_white = chess_library::Board::is_mate_white(&self.board);
                self.black_win = is_mate_white;

                let is_mate_black = chess_library::Board::is_mate_black(&self.board);
                self.white_win = is_mate_black;

                return Ok(());
            }
        }
        let square_x = (x as u32 - self.board_x) / self.square_side;
        if square_x > 7 {
            return Ok(());
        }

        let square_y = (y as u32 - self.board_y) / self.square_side;
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

            // Piece clicked this time (square_index) doesnt match turn
            if piece_type > 5 && self.board.white_turn {
                return Ok(());
            } else if !self.board.white_turn && piece_type < 6 {
                return Ok(());
            }
        }

        if self.clicked_piece.is_some() {
            let piece_type = chess_library::Board::piece_type_on_position(
                &self.board,
                self.clicked_piece.expect("???"),
            );
            if piece_type == 0 || piece_type == 6 {
                // New square on last or first rank
                if square_index <= 7 || square_index >= 56 {
                    self.promoting = true;
                    self.promoting_square = square_index;
                    return Ok(());
                }
            }

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

            let is_mate_white = chess_library::Board::is_mate_white(&self.board);
            self.black_win = is_mate_white;

            let is_mate_black = chess_library::Board::is_mate_black(&self.board);
            self.white_win = is_mate_black;

            println!("white mate: {is_mate_white}");
            println!("black mate: {is_mate_black}");
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
