use actix::{Addr, Actor, StreamHandler, ActorContext, Handler, AsyncContext, Running};
use actix_web_actors::ws;

use crate::server::{Message, ChatServer, ClientMessage, Connect, Disconnect};


pub struct Session {
    pub id: String,
    pub name: Option<String>,
    pub channel: String,
    pub server: Addr<ChatServer>,
}

impl Actor for Session {
    type Context = ws::WebsocketContext<Self>;

    fn started(&mut self, ctx: &mut Self::Context) {
        // You can access the HTTP request here if needed
        self.server.do_send(Connect {
            id: self.id.clone(),
            channel: self.channel.clone(),
            addr: ctx.address().recipient(),
        });
        ctx.text("Session: Joined");
    }

    fn stopping(&mut self, _: &mut Self::Context) -> Running {
        // notify chat server
        self.server.do_send(Disconnect { id: self.id.clone(), channel: self.channel.clone() });
        Running::Stop
    }
}

impl Handler<Message> for Session {
    type Result = ();

    fn handle(&mut self, msg: Message, ctx: &mut Self::Context) {
        ctx.text(msg.0);
    }
}

impl StreamHandler<Result<ws::Message, ws::ProtocolError>> for Session {
    fn handle(&mut self, msg: Result<ws::Message, ws::ProtocolError>, ctx: &mut Self::Context) {
        let msg = match msg {
            Err(_) => {
                ctx.stop();
                return;
            }
            Ok(msg) => msg,
        };

        match msg {
            ws::Message::Ping(msg) => ctx.pong(&msg),
            ws::Message::Pong(_) => (),
            ws::Message::Text(text) => {
                self.server.do_send(ClientMessage {
                        id: self.id.clone(),
                        msg: text.to_string(),
                        channel: self.channel.clone(),
                    })
            }
            ws::Message::Binary(bin) => ctx.binary(bin),
            ws::Message::Close(reason) => {
                ctx.close(reason);
                ctx.stop();
            }
            ws::Message::Continuation(_) => {
                ctx.stop();
            }
            ws::Message::Nop => (),
        }
    }
}
