extern crate futures;
extern crate tokio_minihttp;
extern crate tokio_proto;
extern crate tokio_service;

use std::io;

use futures::future;
use tokio_minihttp::{Request, Response, Http};
use tokio_proto::TcpServer;
use tokio_service::Service;

struct Redirector;

impl Redirector {
	fn redirect(&self, host: &str, path: &str) -> future::Ok<Response, io::Error> {
		let mut resp = Response::new();
		resp.status_code(301, "Moved Permanently");
		resp.header("Location", &format!("https://{}{}", host, path));
		future::ok(resp)
	}

	fn error(&self) -> future::Ok<Response, io::Error> {
		let mut resp = Response::new();
		resp.status_code(400, "Bad Request");
		resp.header("Content-Type", "text/plain");
		resp.body("Bad Request");
		future::ok(resp)
	}
}

impl Service for Redirector {
	type Request = Request;
	type Response = Response;
	type Error = io::Error;
	type Future = future::Ok<Response, io::Error>;

	fn call(&self, request: Request) -> Self::Future {
		let path = request.path();

		// validate path
		if !path.starts_with("/") {
			return self.error();
		}

		request.headers()
			.find(|item| item.0.to_lowercase() == "host")
			.and_then(|item| std::str::from_utf8(item.1).ok())
			.map(|host| self.redirect(host, path))
			.unwrap_or_else(|| self.error())
	}
}

fn main() {
	let addr = "0.0.0.0:8080".parse().unwrap();
	TcpServer::new(Http, addr)
		.serve(|| Ok(Redirector));
}
