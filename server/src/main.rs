#[macro_use]
extern crate lazy_static;

mod server;
mod state;

use self::server::MyWebSocket;
use self::state::{State, CURRENT_STATE, RECIEVER_ADDRS};

use actix_web::{
    middleware, post, web, App, Error, HttpRequest, HttpResponse, HttpServer, Responder,
};
use actix_web_actors::ws;

use openssl::ssl::{SslAcceptor, SslFiletype, SslMethod};

async fn echo_ws(req: HttpRequest, stream: web::Payload) -> Result<HttpResponse, Error> {
    ws::start(MyWebSocket::new(), &req, stream)
}

#[post("/current")]
async fn set_state(req_body: String) -> impl Responder {
    let lock = RECIEVER_ADDRS.lock().unwrap();

    if let Ok(state) = State::from_str(req_body) {
        {
            let mut current = CURRENT_STATE.lock().unwrap();

            *current = state;
        }

        for (_uuid, addr) in &mut lock.iter() {
            match addr.try_send(state) {
                Ok(_) => {}
                Err(e) => log::error!("Could not send message. Got error: {}", e),
            }
        }

        HttpResponse::Ok()
    } else {
        HttpResponse::NotFound()
    }
}

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    env_logger::init_from_env(env_logger::Env::new().default_filter_or("info"));

    log::info!("starting HTTP server at http://localhost:8080");

    let mut builder = SslAcceptor::mozilla_intermediate(SslMethod::tls()).unwrap();
    builder
        .set_private_key_file("cert.key", SslFiletype::PEM)
        .unwrap();
    builder.set_certificate_chain_file("cert.crt").unwrap();

    HttpServer::new(|| {
        App::new()
            .service(web::resource("/ws").route(web::get().to(echo_ws)))
            .service(set_state)
            // enable logger
            .wrap(middleware::Logger::default())
    })
    .bind_openssl("0.0.0.0:8080", builder)?
    .run()
    .await
}
