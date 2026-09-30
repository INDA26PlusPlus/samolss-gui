use std::{error::Error, io::BufReader, net::TcpStream, ops::Bound};

use chess_library::Board;
use ggez::{
    Context, GameError, GameResult, event,
    glam::*,
    graphics::{self, Color, DrawParam},
    mint::Point2,
    winit::event::MouseButton,
};
use std::cmp;

use crate::network::{self, read_move, read_msg, send_move, send_msg};

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

const LEGAL_MOVE_COLOR: graphics::Color = graphics::Color {
    r: 111.0 / 255.0,
    g: 116.0 / 255.0,
    b: 122.0 / 255.0,
    a: 0.6,
};

const CERISE_COLOR: graphics::Color = graphics::Color {
    r: 222.0 / 255.0,
    g: 49.0 / 255.0,
    b: 99.0 / 255.0,
    a: 1.0,
};

pub struct MainState {
    square_side: u32,
    board_x: u32,
    board_y: u32,

    player_is_white: bool,
    writer: TcpStream,
    reader: BufReader<TcpStream>,
    current_buffer: Vec<u8>,

    board: Board,
    promoting: bool,
    promoting_piece: Option<usize>,
    piece_assets: [graphics::Image; 12],
    clicked_piece: Option<usize>,
    clicked_square: Option<usize>,
    legal_moves: [u64; 64],
    white_win: bool,
    black_win: bool,
    draw: bool,
}

impl MainState {
    pub fn new(
        ctx: &mut Context,
        player_is_white: bool,
        writer: TcpStream,
        reader: BufReader<TcpStream>,
    ) -> GameResult<MainState> {
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
            clicked_square: None,
            legal_moves,
            white_win: false,
            black_win: false,
            draw: false,
            promoting: false,
            promoting_piece: None,
            player_is_white,
            writer,
            reader,
            current_buffer: Vec::new(),
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
                x: (self.board_x as usize + self.square_side as usize * (7 - i % 8)) as f32,
                y: if self.player_is_white {
                    (self.board_y + self.square_side * (7 - i as u32 / 8)) as f32
                } else {
                    (self.board_y + self.square_side * (i as u32 / 8)) as f32
                },
                w: self.square_side as f32,
                h: self.square_side as f32,
            };

            let color = if self.clicked_piece == Some(i) {
                Color::GREEN
            } else if i % 2 == (i / 8 % 2) {
                WHITE_SQUARE_COLOR
            } else {
                BLACK_SQUARE_COLOR
            };

            let square =
                graphics::Mesh::new_rectangle(ctx, graphics::DrawMode::fill(), bounds, color)
                    .expect("FFFFFFFUUUUUCKKKK");

            canvas.draw(&square, Vec2::new(0 as f32, 0 as f32));

            if self.clicked_piece.is_some()
                && self.legal_moves[self.clicked_piece.expect("???")] >> i & 1 == 1
            {
                let circle_center = Point2 {
                    x: (self.board_x as usize
                        + self.square_side as usize * (7 - i % 8)
                        + self.square_side as usize / 2) as f32,
                    y: if self.player_is_white {
                        (self.board_y
                            + self.square_side * (7 - i as u32 / 8)
                            + self.square_side as u32 / 2) as f32
                    } else {
                        (self.board_y
                            + self.square_side * (i as u32 / 8)
                            + self.square_side as u32 / 2) as f32
                    },
                };
                let circle = graphics::Mesh::new_circle(
                    ctx,
                    graphics::DrawMode::fill(),
                    circle_center,
                    self.square_side as f32 / 5.0,
                    0.1,
                    LEGAL_MOVE_COLOR,
                )
                .expect("Fuuuuuck");
                canvas.draw(&circle, Vec2::new(0 as f32, 0 as f32));
            }
            let piece_type = chess_library::Board::piece_type_on_position(&self.board, i);
            if piece_type >= 0 {
                let x = (self.board_x + self.square_side * (7 - i as u32 % 8)) as f32;
                let y = if self.player_is_white {
                    (self.board_y + self.square_side * (7 - i as u32 / 8)) as f32
                } else {
                    (self.board_y + self.square_side * (i as u32 / 8)) as f32
                };
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

            let color = CERISE_COLOR;
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

            let color = CERISE_COLOR;

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

    fn handle_promotion(&mut self) -> GameResult {
        let promotable_asset = if self.board.white_turn {
            [5, 1, 2, 3]
        } else {
            [11, 7, 8, 9]
        };

        let mut temp_board = self.board.clone();

        let clicked_piece = self.clicked_piece.expect("???");
        let clicked_square = self.clicked_square.expect("???");
        let promoting_piece = self.promoting_piece.expect("???");

        let valid_temp_move = chess_library::Board::move_piece(
            &mut temp_board,
            clicked_piece,
            clicked_square as u64,
            Some(promotable_asset[promoting_piece] as usize),
        );

        if !valid_temp_move {
            self.promoting_piece = None;
            self.promoting = false;
            self.clicked_square = None;
            self.clicked_piece = None;
            return Ok(());
        }
        send_move(
            &mut self.writer,
            clicked_piece,
            clicked_square as u64,
            Some(promotable_asset[promoting_piece] as usize),
            &temp_board,
        );

        while !read_msg(&mut self.reader, &mut self.current_buffer).unwrap_or(false) {}
        let msg = std::mem::take(&mut self.current_buffer);

        let cleaned_msg = msg.strip_suffix(b"\n").unwrap_or(b"REJECT");
        match cleaned_msg {
            b"OK" => {}
            b"REJECT" => {
                self.clicked_piece = None;
                self.clicked_square = None;
                self.promoting = false;
                self.promoting_piece = None;
                return Ok(());
            }
            b"CHECKMATE" => {
                self.clicked_piece = None;
                self.clicked_square = None;
                self.promoting = false;
                self.promoting_piece = None;
                self.white_win = self.player_is_white;
                self.black_win = !self.player_is_white;
                self.draw = false;
                return Ok(());
            }
            b"STALEMATE" => {
                self.clicked_piece = None;
                self.clicked_square = None;
                self.promoting = false;
                self.promoting_piece = None;
                self.white_win = false;
                self.black_win = false;
                self.draw = true;
                return Ok(());
            }
            _ => {
                self.clicked_piece = None;
                self.clicked_square = None;
                self.promoting = false;
                self.promoting_piece = None;
                return Ok(());
            }
        }
        let valid_move = chess_library::Board::move_piece(
            &mut self.board,
            clicked_piece,
            clicked_square as u64,
            Some(promotable_asset[promoting_piece] as usize),
        );
        if valid_move {
            self.legal_moves = chess_library::Board::get_all_legal_moves(&self.board);
        }
        self.clicked_piece = None;
        self.clicked_square = None;
        self.promoting = false;
        self.promoting_piece = None;

        let is_mate_white = chess_library::Board::is_mate_white(&self.board);
        self.black_win = is_mate_white;

        let is_mate_black = chess_library::Board::is_mate_black(&self.board);
        self.white_win = is_mate_black;

        return Ok(());
    }

    fn handle_move(&mut self) -> GameResult {
        let clicked_piece = self.clicked_piece.expect("???");
        let clicked_square = self.clicked_square.expect("???");
        let mut temp_board = self.board.clone();

        let valid_temp_move = chess_library::Board::move_piece(
            &mut temp_board,
            clicked_piece,
            clicked_square as u64,
            None,
        );

        if valid_temp_move {
            send_move(
                &mut self.writer,
                clicked_piece,
                clicked_square as u64,
                None,
                &temp_board,
            );

            while !read_msg(&mut self.reader, &mut self.current_buffer).unwrap_or(false) {}
            let msg = std::mem::take(&mut self.current_buffer);

            let cleaned_msg = msg.strip_suffix(b"\n").unwrap_or(b"REJECT");
            match cleaned_msg {
                b"OK" => {}
                b"REJECT" => {
                    self.clicked_piece = None;
                    self.clicked_square = None;
                    return Ok(());
                }
                b"CHECKMATE" => {
                    println!("Received checkmate");
                    self.clicked_piece = None;
                    self.clicked_square = None;
                    self.white_win = self.player_is_white;
                    self.black_win = !self.player_is_white;
                    self.draw = false;
                    return Ok(());
                }
                b"STALEMATE" => {
                    println!("Received stalemate");
                    self.clicked_piece = None;
                    self.clicked_square = None;
                    self.white_win = false;
                    self.black_win = false;
                    self.draw = true;
                    return Ok(());
                }
                _ => {
                    self.clicked_piece = None;
                    self.clicked_square = None;
                    return Ok(());
                }
            }
        }

        let valid_move = chess_library::Board::move_piece(
            &mut self.board,
            clicked_piece,
            clicked_square as u64,
            None,
        );
        if valid_move {
            self.legal_moves = chess_library::Board::get_all_legal_moves(&self.board);
        }

        self.clicked_piece = None;
        self.clicked_square = None;

        let is_mate_white = chess_library::Board::is_mate_white(&self.board);
        self.black_win = is_mate_white;

        let is_mate_black = chess_library::Board::is_mate_black(&self.board);
        self.white_win = is_mate_black;

        if !self.white_win && !self.black_win && self.legal_moves.len() == 0 {
            self.draw = true;
        }

        return Ok(());
    }

    fn handle_opponent_move(&mut self) -> GameResult {
        let mut message: (u64, u64, String, String) = (0, 0, "".to_string(), "".to_string());
        if !read_move(
            &mut self.reader,
            &mut self.writer,
            &mut self.current_buffer,
            &mut message,
        )
        .unwrap_or(false)
        {
            return Ok(());
        }
        let (old_position, new_position, promotion_piece, received_board_state) = message;

        let move_is_legal = self.legal_moves[old_position as usize] >> new_position & 1 == 1;
        let promotion_piece_int = match promotion_piece.as_str() {
            "Q" => {
                if self.board.white_turn {
                    Some(chess_library::Board::W_QUEENS)
                } else {
                    Some(chess_library::Board::B_QUEENS)
                }
            }
            "R" => {
                if self.board.white_turn {
                    Some(chess_library::Board::W_ROOKS)
                } else {
                    Some(chess_library::Board::B_ROOKS)
                }
            }
            "B" => {
                if self.board.white_turn {
                    Some(chess_library::Board::W_BISHOPS)
                } else {
                    Some(chess_library::Board::B_BISHOPS)
                }
            }
            "N" => {
                if self.board.white_turn {
                    Some(chess_library::Board::W_KNIGHTS)
                } else {
                    Some(chess_library::Board::B_KNIGHTS)
                }
            }
            _ => None,
        };
        if !move_is_legal {
            network::send_msg(&mut self.writer, b"REJECT\n");

            return Ok(());
        }

        let mut temp_board = self.board.clone();
        chess_library::Board::move_piece(
            &mut temp_board,
            old_position as usize,
            new_position,
            promotion_piece_int,
        );

        let own_board_state = network::board_to_board_state(&temp_board);
        if own_board_state != received_board_state {
            network::send_msg(&mut self.writer, b"REJECT\n");
            return Ok(());
        }

        chess_library::Board::move_piece(
            &mut self.board,
            old_position as usize,
            new_position,
            promotion_piece_int,
        );
        self.legal_moves = chess_library::Board::get_all_legal_moves(&self.board);

        let is_mate_white = chess_library::Board::is_mate_white(&self.board);
        self.black_win = is_mate_white;

        let is_mate_black = chess_library::Board::is_mate_black(&self.board);
        self.white_win = is_mate_black;

        if (self.player_is_white && self.black_win) || (!self.player_is_white && self.white_win) {
            network::send_msg(&mut self.writer, b"CHECKMATE\n");
            self.writer.shutdown(std::net::Shutdown::Both);
            self.reader.get_ref().shutdown(std::net::Shutdown::Both);

            return Ok(());
        }

        // Elbjork library doesnt have an easy way to check for draw
        if (!self.black_win && !self.white_win)
            && chess_library::Board::num_legal_moves_for_current_color(
                &self.board,
                self.legal_moves,
            ) == 0
        {
            self.draw = true;
            println!("Sending stalemate");
            network::send_msg(&mut self.writer, b"STALEMATE\n");
            self.writer.shutdown(std::net::Shutdown::Both);
            self.reader.get_ref().shutdown(std::net::Shutdown::Both);
            return Ok(());
        }

        network::send_msg(&mut self.writer, b"OK\n");
        return Ok(());
    }

    fn coords_to_board_index(&mut self, x: u32, y: u32) -> Option<usize> {
        let square_x = 7 - (x as u32 - self.board_x) / self.square_side;
        if square_x > 7 {
            return None;
        }

        let square_y = if self.player_is_white {
            7 - (y as u32 - self.board_y) / self.square_side
        } else {
            (y as u32 - self.board_y) / self.square_side
        };
        if square_y > 7 {
            return None;
        }

        let square_index = square_x as usize + square_y as usize * 8;
        return Some(square_index);
    }
}

impl event::EventHandler for MainState {
    fn update(&mut self, _ctx: &mut Context) -> GameResult {
        if self.clicked_piece.is_some()
            && self.clicked_square.is_some()
            && self.promoting
            && self.promoting_piece.is_some()
        {
            return self.handle_promotion();
        }

        if self.clicked_piece.is_some() && self.clicked_square.is_some() && !self.promoting {
            return self.handle_move();
        }

        if self.player_is_white != self.board.white_turn {
            return self.handle_opponent_move();
        }

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

        // Only allow interaction when its the players turn
        if self.player_is_white != self.board.white_turn {
            return Ok(());
        }

        if self.promoting {
            let screen = _ctx.gfx.drawable_size();
            let popup_x = (screen.0 - POPUP_W) / 2.0;
            let popup_y = (screen.1 - POPUP_H) / 2.0;

            let piece_index_clicked = ((x as u32 - popup_x as u32) / (POPUP_W as u32 / 2)
                + 2 * ((y as u32 - popup_y as u32) / (POPUP_H as u32 / 2)))
                as usize;

            self.promoting_piece = Some(piece_index_clicked);
            return Ok(());
        }

        let square_index_option = self.coords_to_board_index(x as u32, y as u32);
        if square_index_option.is_none() {
            return Ok(());
        }
        let square_index = square_index_option.expect("???");

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

            self.clicked_piece = Some(square_index);
            return Ok(());
        }

        let piece_type = chess_library::Board::piece_type_on_position(
            &self.board,
            self.clicked_piece.expect("???"),
        );
        if piece_type == 0 || piece_type == 6 {
            // New square on last or first rank
            if square_index <= 7 || square_index >= 56 {
                self.promoting = true;
                self.clicked_square = Some(square_index);
                return Ok(());
            }
        }
        self.clicked_square = Some(square_index);

        return Ok(());
    }
}
