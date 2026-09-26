mod gui;
mod network;
use std::{error::Error, ops::Bound};

use chess_library::Board;
use ggez::{
    Context, GameError, GameResult, event,
    glam::*,
    graphics::{self, Color, DrawParam},
    winit::event::MouseButton,
};
use std::cmp;
use std::env;
use std::fmt;
use std::net::TcpListener;
use std::net::TcpStream;

use crate::network::read_is_white;

#[derive(Debug)]
struct ParsedArgs {
    connect_to: Option<String>,
    accept_only: Option<String>,
    is_white: Option<bool>,
}

const CC_PORT: u32 = 6767;

fn parse_args() -> ParsedArgs {
    let args: Vec<String> = env::args().collect();
    let mut parsed = ParsedArgs {
        connect_to: None,
        accept_only: None,
        is_white: None,
    };
    let mut i = 0;
    while i < args.len() {
        match args[i].as_str() {
            "--connect" => parsed.connect_to = Some(<String as Clone>::clone(&args[i + 1])),
            "--accept-only" => parsed.accept_only = Some(<String as Clone>::clone(&args[i + 1])),
            "--color" => match args[i + 1].as_str() {
                "w" => parsed.is_white = Some(true),
                "b" => parsed.is_white = Some(false),
                _ => parsed.is_white = None,
            },
            _ => {}
        }
        i += 1;
    }
    return parsed;
}

fn main() -> GameResult {
    let args = parse_args();
    let mut player_is_white = false;
    if args.connect_to.is_none() {
        player_is_white = if args.is_white.is_none() {
            rand::random_bool(0.5)
        } else {
            args.is_white.expect("???")
        }
    }
    let (mut writer, mut reader) =
        network::create_stream(&args.connect_to, &args.accept_only, player_is_white);

    if args.connect_to.is_some() {
        player_is_white = read_is_white(&mut reader);
    }
    println!("is white: {player_is_white}");

    let cb = ggez::ContextBuilder::new("chess", "elbjork-samolss");
    let (mut ctx, event_loop) = cb.build()?;
    ctx.fs.print_all();
    let state = gui::MainState::new(&mut ctx, player_is_white, writer, reader)?;
    event::run(ctx, event_loop, state)
}
