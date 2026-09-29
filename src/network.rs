use chess_library::Board;
use core::panic;
use std::io;
use std::io::BufRead;
use std::io::BufReader;
use std::io::Write;
use std::net::SocketAddr;
use std::net::TcpListener;
use std::net::TcpStream;
use std::str;

pub fn create_stream(
    connect_to: &Option<String>,
    accept_only: &Option<String>,
    is_white: bool,
) -> (TcpStream, BufReader<TcpStream>) {
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

        let msg = if is_white { b"B\n" } else { b"W\n" };
        send_msg(&mut stream, msg);

        let reader = BufReader::new(stream.try_clone().expect("Fuck"));
        return (stream, reader);
    }

    let mut stream = TcpStream::connect(connect_to.clone().expect(("???")))
        .expect("Couldn't connect to the server...");

    stream
        .set_nonblocking(true)
        .expect("set_nonblocking call failed");

    let mut reader = BufReader::new(stream.try_clone().expect("Fuck"));
    return (stream, reader);
}

pub fn send_msg(stream: &mut TcpStream, data: &[u8]) {
    stream.write_all(data).expect("Failed to write message");
    stream.flush();
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

    let promotion_string = match promotion {
        Some(Board::W_QUEENS) | Some(Board::B_QUEENS) => "Q",
        Some(Board::W_ROOKS) | Some(Board::B_ROOKS) => "R",
        Some(Board::W_BISHOPS) | Some(Board::B_BISHOPS) => "B",
        Some(Board::W_KNIGHTS) | Some(Board::B_KNIGHTS) => "N",
        _ => "-",
    };

    let board_state = board_to_board_state(board);

    let msg = format!("{alg_start}{alg_stop}{promotion_string}{board_state}\n");
    send_msg(stream, msg.as_bytes());
}

pub fn read_msg(stream: &mut BufReader<TcpStream>, buffer: &mut Vec<u8>) -> io::Result<bool> {
    match stream.read_until(b'\n', buffer) {
        Ok(_) if buffer.ends_with(b"\n") => return Ok(true),
        Ok(_) => return Ok(false),
        Err(ref e) if e.kind() == io::ErrorKind::WouldBlock => return Ok(false),
        Err(e) => return Err(e),
    };
}

pub fn read_is_white(stream: &mut BufReader<TcpStream>) -> bool {
    let mut msg: Vec<u8> = Vec::new();
    while !read_msg(stream, &mut msg).expect("Something went wrong") {
        println!("{msg:?}");
    }
    println!("{msg:?}");
    match msg[0] {
        b'W' => true,
        b'B' => false,
        _ => false,
    }
}

pub fn read_move(
    stream: &mut BufReader<TcpStream>,
    writer: &mut TcpStream,
    buffer: &mut Vec<u8>,
    message: &mut (u64, u64, String, String),
) -> io::Result<(bool)> {
    if !read_msg(stream, buffer)? {
        return Ok(false);
    };

    println!("buffer = {buffer:?}");
    let msg_res = String::from_utf8(std::mem::take(buffer));
    if msg_res.is_err() {
        send_msg(writer, b"REJECT\n");
        return Err(io::Error::new(
            io::ErrorKind::InvalidData,
            "Invalid input data",
        ));
    }
    let mut msg = msg_res.unwrap();

    if !msg.is_ascii() {
        send_msg(writer, b"REJECT\n");
        return Err(io::Error::new(
            io::ErrorKind::InvalidData,
            "Data is not ascii 🤔",
        ));
    }

    println!("received move: {msg}");

    if msg.len() != 4 + 1 + 64 + 1 {
        send_msg(writer, b"REJECT\n");
        return Err(io::Error::new(
            io::ErrorKind::InvalidData,
            "Data is of incorrect length, should be 70 bytes",
        ));
    }

    let move_coords = &msg[0..4];
    let promotion_piece = msg[4..5].to_string();
    let board_state = msg[5..msg.len() - 1].to_string();

    let start_pos = algebraic_to_position(move_coords[0..2].to_string()).inspect_err(|e| {
        send_msg(writer, b"REJECT\n");
    })?;

    let end_pos = algebraic_to_position(move_coords[2..move_coords.len()].to_string())
        .inspect_err(|e| {
            send_msg(writer, b"REJECT\n");
        })?;

    println!("start: {start_pos}");
    println!("end: {end_pos}");
    *message = (start_pos, end_pos, promotion_piece, board_state);
    return Ok(true);
}

fn position_to_algebraic(position: u64) -> String {
    let file_int = position % 8;
    let rank_int = position / 8 + 1;

    let file = match file_int {
        0 => "h",
        1 => "g",
        2 => "f",
        3 => "e",
        4 => "d",
        5 => "c",
        6 => "b",
        7 => "a",
        _ => unreachable!(),
    };

    return format!("{file}{rank_int}");
}

fn algebraic_to_position(alg: String) -> std::io::Result<u64> {
    let mut coords = alg.chars();
    println!("alg: {:?}", coords);
    let file = coords.next();
    let rank = coords.next();

    if file.is_none() || rank.is_none() {
        return Err(io::Error::new(
            io::ErrorKind::InvalidData,
            "Invalid file or rank",
        ));
    }

    let file_int: u64 = match file.expect("???") {
        'h' => 0,
        'g' => 1,
        'f' => 2,
        'e' => 3,
        'd' => 4,
        'c' => 5,
        'b' => 6,
        'a' => 7,
        _ => return Err(io::Error::new(io::ErrorKind::InvalidData, "Invalid file")),
    };

    let rank_int_ = rank.expect("???").to_digit(10);
    if rank_int_.is_none() {
        return Err(io::Error::new(
            io::ErrorKind::InvalidData,
            "Provided rank not a number",
        ));
    }

    let rank_int = (rank_int_.expect("???") - 1) as u64;
    if rank_int > 7 || rank_int < 0 {
        return Err(io::Error::new(
            io::ErrorKind::InvalidData,
            "Rank is outside of board",
        ));
    }

    return Ok(rank_int * 8 + file_int);
}

pub fn board_to_board_state(board: &chess_library::Board) -> String {
    let mut board_state: String = "".to_string();
    for i in 0..64 {
        // This becomes a bit unintuitive since the board state should go a8 -> h8 -> a7 -> ... -> h1
        let piece_type = Board::piece_type_on_position(board, (63 - i % 8 - (i / 8) * 8));

        if piece_type < 0 {
            board_state += " ";
            continue;
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
    return board_state;
}
