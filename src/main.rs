use std::io::{Read, Write};
use std::net::TcpListener;
use std::thread;
use std::str;

fn main() {
    // You can use print statements as follows for debugging, they'll be visible when running tests.
    println!("Logs from your program will appear here!");

    let listener = TcpListener::bind("127.0.0.1:6379").unwrap();

    for stream in listener.incoming() {
        match stream {
            Ok(mut stream) => {
                println!("accepted new connection");

                thread::spawn(move || {
                    let mut read_buffer = [0;512];

                    loop {
                        let read_count = stream.read(&mut read_buffer).unwrap();
                        if read_count == 0 {
                            break;
                        }
                        let command = str::from_utf8(&read_buffer).unwrap().to_string();

                        let break_down = parse_command(command);
                        let response = eval_command(&break_down);

                        stream.write(&*response.into_bytes()).unwrap();
                    }
                });
            }
            Err(e) => {
                println!("error: {}", e);
            }
        }
    }
}

/*
 * *1\r\n$4\r\nPING\r\n
 */
fn parse_command(buff: String) -> Vec<String> {
    // TODO: handle the length encoding lines.
    let mut pieces = buff.split("\r\n");
    let array_len = pieces.next().unwrap()[1..].parse::<usize>().unwrap();

    let mut cmd_buffer: Vec<String> = Vec::with_capacity(array_len);
    pieces.next().unwrap();

    let cmd = pieces.next().unwrap().to_ascii_uppercase();
    cmd_buffer.push(cmd);

    for piece in pieces {
        if piece.starts_with('$') {
            continue;
        }
        cmd_buffer.push(piece.to_string());
    }

    cmd_buffer
}


fn eval_command(segments: &Vec<String>) -> String {
    match &segments[0] {
        cmd if cmd == "ECHO" => eval_echo(segments),
        cmd if cmd == "PING" => eval_ping(segments),
        cmd => panic!("Not a valid command: {}", cmd)
    }
}

fn eval_echo(segments: &Vec<String>) -> String {
    let res = format!("${}\r\n{}\r\n", &segments[1].chars().count(), &segments[1]);
    println!("{}", res);
    res
}

fn eval_ping(_segments: &Vec<String>) -> String {
    String::from("+PONG\r\n")
}
