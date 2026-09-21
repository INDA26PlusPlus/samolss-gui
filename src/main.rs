use chess_library::Board;
use ggez::{
    Context, GameResult, event,
    glam::*,
    graphics::{self, Color},
};

struct MainState {
    pos_x: f32,
    circle: graphics::Mesh,
}

impl MainState {
    fn new(ctx: &mut Context) -> GameResult<MainState> {
        let circle = graphics::Mesh::new_circle(
            ctx,
            graphics::DrawMode::fill(),
            vec2(0., 0.),
            100.0,
            2.0,
            Color::WHITE,
        )?;

        Ok(MainState { pos_x: 0.0, circle })
    }
}

impl event::EventHandler for MainState {
    fn update(&mut self, _ctx: &mut Context) -> GameResult {
        self.pos_x = self.pos_x % 800.0 + 1.0;
        Ok(())
    }

    fn draw(&mut self, ctx: &mut Context) -> GameResult {
        let mut canvas =
            graphics::Canvas::from_frame(ctx, graphics::Color::from([0.1, 0.2, 0.3, 1.0]));

        canvas.draw(&self.circle, Vec2::new(self.pos_x, 380.0));

        canvas.finish(ctx)?;

        Ok(())
    }
}

pub fn main() -> GameResult {
    let cb = ggez::ContextBuilder::new("super_simple", "ggez");
    let (mut ctx, event_loop) = cb.build()?;
    let state = MainState::new(&mut ctx)?;
    event::run(ctx, event_loop, state)
}
fn main() {
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

    Board::print_board(board);
}
