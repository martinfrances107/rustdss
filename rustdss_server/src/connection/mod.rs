use crate::request::Request;
use rustdss_core::Message;
use rustdss_data::RespData;
use rustdss_transport::{deserialise::DeserialiseRespData, serialise::SerialiseRespData};
use std::io::{BufReader, Read, Write};
use std::net::{TcpListener, TcpStream};
use std::sync::mpsc::Sender;
use std::thread;

pub struct Connection {}

impl Connection {
    fn handle_incoming_stream(core_sender: Sender<Message>, stream: &mut TcpStream) {
        // This function will create and use instances of Request
        println!("[connection], handling tcp stream from client {:?}", stream);

        let bufreader: BufReader<TcpStream> = BufReader::new(stream.try_clone().unwrap());

        let mut byte_stream = &mut bufreader
            .bytes()
            .map(|item| item.expect("should be achar") as char);

        let mut database_id = None;
        loop {
            if let Some(input_data) = RespData::from_char_stream(&mut byte_stream) {
                // Parse each request and give the parsed request to the Request module
                // Turn the bytes into a stream of chars!
                //
                // We need some state here to manage which database this connection/session is
                // talking to
                let (new_database_id, response) =
                    Request::handle(database_id.clone(), core_sender.clone(), input_data);

                database_id = new_database_id;

                stream
                    .write(response.as_string().as_bytes())
                    .expect("Can't write to socket");
            } else {
                break;
            }
        }
    }

    pub fn start(outer_core_sender: Sender<Message>) -> std::io::Result<Self> {
        println!("[connection] Starting to listen to connections");

        let listener = TcpListener::bind("0.0.0.0:6380")?;

        for stream in listener.incoming() {
            let core_sender = outer_core_sender.clone();
            thread::spawn(move || {
                Connection::handle_incoming_stream(core_sender.clone(), &mut stream.unwrap());
                println!("[connection] connection terminated");
            });
        }

        Ok(Self {})
    }
}
