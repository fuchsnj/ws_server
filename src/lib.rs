extern crate websocket;
extern crate openssl;

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
use openssl::ssl::{SslContext, SslMethod};
use openssl::x509::X509FileType;
use std::path::Path;


struct MyHandler;
impl Handler for MyHandler{
	fn new(ws: &Websocket) -> MyHandler{
		MyHandler
	}
	fn handle(&mut self, event: Event, ws: &mut Websocket){
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
	MyHandler::start("0.0.0.0:8080", None);
}

pub enum Event{
	Close,
	Text(String),
	Binary(Vec<u8>),
}
pub enum Protocol{

}

#[derive(Clone)]
pub struct Websocket{
	sender: Arc<Mutex<websocket::server::sender::Sender<WebSocketStream>>>
}
impl Websocket{
	fn new(sender: websocket::server::sender::Sender<WebSocketStream>) -> Websocket{
		Websocket{
			sender: Arc::new(Mutex::new(sender))
		}
	}
	pub fn send_text(&mut self, msg: &str){
		let mut data = self.sender.lock().unwrap();
		match data.send_message(
			Message::Text(msg.to_string())
		){
			Ok(_) => println!("ws send success: {}", msg),
			Err(_) => println!("WS SEND FAILED: {}", msg)
		}
	}
	pub fn send_binary(&mut self, data: Vec<u8>){
		let mut ws = self.sender.lock().unwrap();
		ws.send_message(
			Message::Binary(data)
		).unwrap();
	}
	pub fn close(&mut self){
		let mut ws = self.sender.lock().unwrap();
		ws.send_message(
			Message::Close(None)
		).unwrap();
	}
}

pub struct SSLCert{
	pub certificate_file: String,
	pub key_file: String
}

pub trait Handler: Sized{
	fn new(&Websocket) -> Self;
	fn handle(&mut self, Event, &mut Websocket);
	fn run_server(server: websocket::Server){
		for conn in server{
			println!("new incoming ws connection...");
			thread::spawn(move ||{
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
				let mut handler = Self::new(&ws);

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
	}
	fn start<A>(addr: A, ssl: Option<SSLCert>) where A: ToSocketAddrs{
		match ssl{
			Some(cert) => {
				let mut context = SslContext::new(SslMethod::Tlsv1).unwrap();
				let _ = context.set_certificate_file(&(Path::new(&cert.certificate_file)), X509FileType::PEM);
				let _ = context.set_private_key_file(&(Path::new(&cert.key_file)), X509FileType::PEM);
				let server = websocket::Server::bind_secure(addr, &context).unwrap();
				Self::run_server(server);
			},
			None => {
				let server = websocket::Server::bind(addr).unwrap();
				Self::run_server(server);
			}
		};
	}
}



