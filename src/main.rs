use actix::{Actor, Addr, StreamHandler};
use actix_web::{get, web, App, Error, HttpRequest, HttpResponse, HttpServer, Responder};
use actix_web_actors::ws;
use env_logger::Env;
use nanoid::nanoid;
use server::ChatServer;

mod server;
mod session;

struct Channel {
    id: String,
    user_names: Vec<String>,
}

impl Actor for Channel {
    type Context = ws::WebsocketContext<Self>;
}

impl StreamHandler<Result<ws::Message, ws::ProtocolError>> for Channel {
    fn handle(&mut self, msg: Result<ws::Message, ws::ProtocolError>, ctx: &mut Self::Context) {
        let id = nanoid!();

        match msg {
            Ok(ws::Message::Ping(msg)) => ctx.pong(&msg),
            Ok(ws::Message::Text(text)) => ctx.text(format!("{} {} {}", text, id, self.id)),
            Ok(ws::Message::Binary(bin)) => ctx.binary(bin),
            _ => (),
        }
    }
}

// #[get("/ws/{channel}")]
// async fn index_ws(
//     req: HttpRequest,
//     path: web::Path<String>,
//     stream: web::Payload,
// ) -> Result<HttpResponse, Error> {
//     let channel = path.into_inner();
//     let res = ws::start(
//         Channel {
//             id: channel,
//             user_names: Vec::new(),
//         },
//         &req,
//         stream,
//     );
//     println!("{:?}", res);
//     res
// }

#[get("/ws/{channel}")]
async fn websocket(
    req: HttpRequest,
    stream: web::Payload,
    path: web::Path<String>,
    srv: web::Data<Addr<server::ChatServer>>,
) -> Result<HttpResponse, Error> {
    let channel = path.into_inner();
    ws::start(
        session::Session {
            id: nanoid!(),
            name: None,
            channel,
            server: srv.get_ref().clone(),
        },
        &req,
        stream,
    )
}

#[get("/")]
async fn hello() -> impl Responder {
    HttpResponse::Ok().body("Hello, world!")
}

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    env_logger::init_from_env(Env::new().default_filter_or("info"));

    let server = ChatServer::new().start();

    HttpServer::new(move || {
        App::new()
            .app_data(web::Data::new(server.clone()))
            .service(websocket)
            .service(hello)
    })
    .bind(("127.0.0.1", 8080))?
    .run()
    .await
}
