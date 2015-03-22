extern crate websocket;
extern crate task_pool;

use task_pool::Pool;
use std::net::ToSocketAddrs;
use std::thread;
//use websocket;
use websocket::ws::receiver::Receiver;
use websocket::message::Message;
use websocket::stream::WebSocketStream;
// use core::clone::Clone;
// use core::marker::Sync;
use std::collections::HashMap;
use std::sync::{Mutex,Arc};
use websocket::ws::sender::Sender;


struct MyHandler;
impl Handler for MyHandler{
	fn new(ws: &Websocket) -> MyHandler{
		MyHandler
	}
	fn handle(&self, event: Event, ws: &mut Websocket){
		match event{
			Event::Close => {
				println!("WS CLOSED");
			},
			Event::Text(msg) => {
				println!("MSG recieved: {}", msg);
				ws.send_text(&msg);
				ws.close();
			},
			Event::Binary(data) => {
				println!("BINARY recieved.");
				ws.send_binary(data);
			}
		}
	}
}

#[test]
fn it_works() {
	MyHandler::start("0.0.0.0:8080");
}

pub enum Event{
	Close,
	Text(String),
	Binary(Vec<u8>),
}

pub struct Websocket{
	sender: websocket::server::sender::Sender<WebSocketStream>
}
impl Websocket{
	fn new(sender: websocket::server::sender::Sender<WebSocketStream>) -> Websocket{
		Websocket{
			sender: sender
		}
	}
	pub fn send_text(&mut self, msg: &str){
		self.sender.send_message(
			Message::Text(msg.to_string())
		);
	}
	pub fn send_binary(&mut self, data: Vec<u8>){
		self.sender.send_message(
			Message::Binary(data)
		);
	}
	pub fn close(&mut self){
		self.sender.send_message(
			Message::Close(None)
		);
	}
}

pub trait Handler: Sized{
	fn new(&Websocket) -> Self;
	fn handle(&self, Event, &mut Websocket);
	fn start<A>(addr: A) where A: ToSocketAddrs{
		let mut pool = Pool::new();
		let server = websocket::Server::bind(addr).unwrap();
		for conn in server{
			println!("new incoming ws connection...");
			pool.add_task(move ||{
				let request = match conn{
					Err(err) => {
						println!("connection error: {}",err);
						return;
					},
					Ok(conn) => match conn.read_request(){
						Err(err) => {
							println!("request read error: {}",err);
							return;
						},
						Ok(request) => request
					}
				};
				println!("got valid request");
				
				let (mut sender, mut receiver) = match request.accept().send(){
					Err(err) => {
						println!("accept error: {}",err);
						return;
					},
					Ok(client) => client.split()
				};

				let mut ws = Websocket::new(sender);
				let handler = Self::new(&ws);

				for message in receiver.incoming_messages::<websocket::Message>(){
					match message{
						Ok(Message::Text(msg)) => {
							handler.handle(
								Event::Text(msg), &mut ws
							);
						},
						Ok(Message::Binary(data)) => {
							handler.handle(
								Event::Binary(data), &mut ws
							);
						},
						Ok(Message::Close(optional_close_data)) => {
							handler.handle(
								Event::Close, &mut ws
							);
							return;
						},
						Ok(Message::Ping(data)) => {},
						Ok(Message::Pong(data)) => {},
						Err(err) => {
							println!("message err: {}", err);
							return;
						}
					}
				}
			});
		}
		pool.join();
	}
}



