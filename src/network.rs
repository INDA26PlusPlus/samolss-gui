use chess_library::Board;
use core::panic;
use std::io::BufRead;
use std::io::BufReader;
use std::io::Write;
use std::net::SocketAddr;
use std::net::TcpListener;
use std::net::TcpStream;
use std::str;
pub fn create_stream(
    connect_to: Option<String>,
    accept_only: &Option<String>,
    is_white: Option<bool>,
) -> TcpStream {
    if connect_to.is_none() {
        let listener = TcpListener::bind("127.0.0.1:6767").expect("Could not open listener");
        let (mut stream, addr) = loop {
            let (stream, addr) = listener.accept().expect("Failed to accept connection");
            if accept_only.clone().is_none()
                || accept_only.clone().expect("???").parse() == Ok(addr)
            {
                break (stream, addr);
            }

            drop(stream);
        };
        stream
            .set_nonblocking(true)
            .expect("set_nonblocking call failed");
        drop(listener);
        return stream;
    }

    let mut stream =
        TcpStream::connect(connect_to.expect(("???"))).expect("Couldn't connect to the server...");
    stream
        .set_nonblocking(true)
        .expect("set_nonblocking call failed");

    return stream;
}

pub fn send_msg(stream: &mut TcpStream, data: &[u8]) {
    stream.write_all(data).expect("Failed to write message");
}

pub fn send_move(
    stream: &mut TcpStream,
    start_position: usize,
    end_position: u64,
    promotion: Option<usize>,
    board: &Board,
) {
    let alg_start = position_to_algebraic(start_position as u64);
    let alg_stop = position_to_algebraic(end_position);
    let promotion_string = match promotion.unwrap() {
        Board::W_QUEENS | Board::B_QUEENS => "q",
        Board::W_ROOKS | Board::B_ROOKS => "r",
        Board::W_BISHOPS | Board::B_BISHOPS => "b",
        Board::W_KNIGHTS | Board::B_KNIGHTS => "n",
        _ => "-",
    };

    let mut board_state: String = "".to_string();
    for i in 0..64 {
        // This becomes a bit unintuitive since the board state should go a8 -> h8 -> a7 -> ... -> h1
        let piece_type = Board::piece_type_on_position(board, (56 + i % 8 - i / 8));

        if piece_type < 0 {
            board_state += " ";
        }

        match piece_type as usize {
            Board::W_KINGS => board_state += "K",
            Board::W_QUEENS => board_state += "Q",
            Board::W_ROOKS => board_state += "R",
            Board::W_BISHOPS => board_state += "B",
            Board::W_KNIGHTS => board_state += "N",
            Board::W_PAWNS => board_state += "P",
            Board::B_KINGS => board_state += "k",
            Board::B_QUEENS => board_state += "q",
            Board::B_ROOKS => board_state += "r",
            Board::B_BISHOPS => board_state += "b",
            Board::B_KNIGHTS => board_state += "n",
            Board::B_PAWNS => board_state += "p",
            _ => board_state += " ",
        };
    }

    let msg = format!("{alg_start}{alg_stop}{promotion_string}{board_state}\n");
    send_msg(stream, msg.as_bytes());
}

pub fn read_msg(stream: &mut BufReader<TcpStream>) -> Vec<u8> {
    let mut msg: Vec<u8> = Vec::new();
    let len = stream
        .read_until(b'\n', &mut msg)
        .expect("Failed to read msg");
    return msg;
}

pub fn read_is_white(stream: &mut BufReader<TcpStream>) -> bool {
    let msg = read_msg(stream);

    match msg[0] {
        b'w' => true,
        b'b' => false,
        _ => false,
    }
}

pub fn read_move(stream: &mut BufReader<TcpStream>) -> (u64, u64, String, String) {
    let msg = String::from_utf8(read_msg(stream)).expect("Read bad string");

    let move_coords = &msg[0..4];
    let promotion_piece = msg[4..5].to_string();
    let board_state = msg[5..msg.len() - 1].to_string();

    let start_pos = algebraic_to_position(move_coords[0..1].to_string());
    let end_pos = algebraic_to_position(move_coords[1..move_coords.len()].to_string());

    return (start_pos, end_pos, promotion_piece, board_state);
}

fn position_to_algebraic(position: u64) -> String {
    let file_int = position % 8;
    let rank_int = position / 8 + 1;

    let file = match file_int {
        0 => "a",
        1 => "b",
        2 => "c",
        3 => "d",
        4 => "e",
        5 => "f",
        6 => "g",
        7 => "h",
        _ => unreachable!(),
    };

    return format!("{file}{rank_int}");
}

fn algebraic_to_position(alg: String) -> u64 {
    let mut coords = alg.chars();
    let file = coords.next();
    let rank = coords.next();

    if file.is_none() || rank.is_none() {
        panic!("Bad algebraic position gotten");
    }

    let file_int: u64 = match file.expect("???") {
        'a' => 0,
        'b' => 1,
        'c' => 2,
        'd' => 3,
        'e' => 4,
        'f' => 5,
        'g' => 6,
        'h' => 7,
        _ => panic!("Bad file for algebraic position gotten"),
    };
    let rank_int = ((rank.expect("???").to_digit(10).expect("Rank is not digit") - 1) * 8) as u64;
    return rank_int + file_int;
}
