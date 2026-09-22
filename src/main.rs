use std::ops::Bound;

use chess_library::Board;
use ggez::{
    Context, GameResult, event,
    glam::*,
    graphics::{self, Color},
};

struct MainState {
    board: Board,
    piece_assets: [graphics::Image; 12],
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

        Ok(MainState {
            board,
            piece_assets,
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

        draw_board(ctx, &mut canvas, self.board, &self.piece_assets);
        canvas.finish(ctx)?;

        Ok(())
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
) {
    let mut board_list: [&str; 64] = [" . "; 64];
    // chess_library::Board::print_board(board);

    for i in 0..board_list.len() {
        let bounds = graphics::Rect {
            x: (100 + 100 * (i % 8)) as f32,
            y: (100 + 100 * (i / 8)) as f32,
            w: 100 as f32,
            h: 100 as f32,
        };
        let color = if i % 2 == (i / 8 % 2) {
            Color::WHITE
        } else {
            Color::GREEN
        };

        let square = graphics::Mesh::new_rectangle(ctx, graphics::DrawMode::fill(), bounds, color)
            .expect("FFFFFFFUUUUUCKKKK");

        canvas.draw(&square, Vec2::new(0 as f32, 0 as f32));

        for j in 0..board.boards.len() {
            if board.boards[j] & (i as u64) != 0 {
                canvas.draw(&piece_assets[j], Vec2::new(0 as f32, 0 as f32));
                break;
            }
        }
    }
}
