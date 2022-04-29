#[macro_use]
extern crate lazy_static;

#[macro_use]
extern crate diesel;
extern crate dotenv;

mod server;
mod state;

mod onesignal;

mod db;
mod db_schema;

use std::sync::Mutex;

use dotenv::dotenv;
use std::env;

use actix_web::dev::Service;
use diesel::prelude::*;

use self::onesignal::notify_onesignal;
use self::server::MyWebSocket;
use self::state::{State, CURRENT_STATE, RECIEVER_ADDRS};

use actix_cors::Cors;
use actix_web::{
    get, middleware, post, web, App, Error, HttpRequest, HttpResponse, HttpServer, Responder,
};
use actix_web_actors::ws;

use actix_web::http::Method;

use futures_util::future::{self, Either, FutureExt};

// use openssl::ssl::{SslAcceptor, SslFiletype, SslMethod};

async fn echo_ws(req: HttpRequest, stream: web::Payload) -> Result<HttpResponse, Error> {
    ws::start(MyWebSocket::new(), &req, stream)
}

#[get("/current")]
async fn get_current_status() -> impl Responder {
    let current = CURRENT_STATE.lock().unwrap();

    let state = current.to_str().to_string();

    HttpResponse::Ok().body(state)
}

#[post("/current")]
async fn set_state(
    db_connection: web::Data<Mutex<SqliteConnection>>,
    req_body: String,
) -> impl Responder {
    let lock = RECIEVER_ADDRS.lock().unwrap();

    if let Ok(state) = State::from_str(req_body) {
        // log the recieved message in the db
        {
            let db_lock = db_connection.lock().unwrap();

            db::insert_state_log(&db_lock, state);
        }

        {
            let mut current = CURRENT_STATE.lock().unwrap();

            // do not notify anyone if the state did not change
            if *current == state {
                return HttpResponse::Ok();
            }

            *current = state;
        }

        notify_onesignal(state).await;

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
    dotenv().ok();

    env_logger::init_from_env(env_logger::Env::new().default_filter_or("info"));

    log::info!("starting HTTP server at http://localhost:8080");

    // let mut builder = SslAcceptor::mozilla_intermediate(SslMethod::tls()).unwrap();
    // builder
    //     .set_private_key_file("cert.key", SslFiletype::PEM)
    //     .unwrap();
    // builder.set_certificate_chain_file("cert.crt").unwrap();

    let db_connection = web::Data::new(Mutex::new(db::establish_connection()));

    HttpServer::new(move || {
        let cors = Cors::permissive();

        App::new()
            .service(web::resource("/live").route(web::get().to(echo_ws)))
            .service(get_current_status)
            .wrap_fn(|req, srv| {
                let mut access_allowed = true;

                if req.method() == Method::POST {
                    access_allowed = false;

                    let auth_header = req.headers().get(actix_web::http::header::AUTHORIZATION);

                    if let Some(token) = auth_header {
                        let mut token = token.to_str().unwrap().split(" ");

                        token.next();

                        let token = token.next();

                        if let Some(token) = token {
                            if token == "xw6k2YF4t5yCMky2DYxa7NV" {
                                access_allowed = true;
                            }
                        }
                    }
                }

                if access_allowed {
                    Either::Left(srv.call(req).map(|res| res))
                } else {
                    return Either::Right(future::ready(Ok(
                        req.into_response(HttpResponse::Forbidden())
                    )));
                }
            })
            .app_data(db_connection.clone())
            .service(set_state)
            .service(
                actix_files::Files::new("/", "./website")
                    .show_files_listing()
                    .index_file("index.html"),
            )
            .wrap(cors)
            .wrap(middleware::Logger::default())
    })
    // .bind_openssl(format!("0.0.0.0:{}", env::var("SERVER_PORT").expect("You have to set a server port!")), builder)?
    .bind(format!(
        "0.0.0.0:{}",
        env::var("SERVER_PORT").expect("You have to set a server port!")
    ))?
    .run()
    .await
}
