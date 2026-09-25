use std::net::SocketAddr;
use std::net::TcpListener;
use std::net::TcpStream;

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

pub fn send_move(start_position: usize, end_position: u64, promotion: Option<usize>) {}

fn position_to_algebraic(position: u64) -> String {
    let file_int = position & 8;
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
    };

    return format!("{file}{rank_int}");
}
