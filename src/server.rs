use std::collections::{HashMap, HashSet};

use actix::{Recipient, Actor, Context, Handler};

#[derive(actix::Message)]
#[rtype(result = "()")]
pub struct Message(pub String);

#[derive(actix::Message)]
#[rtype(result = "()")]
pub struct ClientMessage {
    pub id: String,
    pub msg: String,
    pub channel: String,
}

#[derive(actix::Message)]
#[rtype(result = "()")]
pub struct Connect {
    pub id: String,
    pub channel: String,
    pub addr: Recipient<Message>,
}

#[derive(actix::Message)]
#[rtype(result = "()")]
pub struct Disconnect {
    pub id: String,
    pub channel: String,
}

#[derive(Debug)]
pub struct ChatServer {
    users: HashMap<String, HashSet<String>>, 
    channels: HashMap<String, HashMap<String, Recipient<Message>>>,
}

impl ChatServer {
    pub fn new() -> Self {
        ChatServer { channels: HashMap::new(), users: HashMap::new() } 
    }

    fn send_message(&self, channel_id: &str, message: &str) {
        if let Some(channel) = self.channels.get(channel_id) {
            for (_, addr) in channel.iter() {
                addr.do_send(Message(message.to_owned()))
            }
        }
    }
}

impl Actor for ChatServer {
    /// We are going to use simple Context, we just need ability to communicate
    /// with other actors.
    type Context = Context<Self>;
}

impl Handler<Connect> for ChatServer {
    type Result = ();

    fn handle(&mut self, msg: Connect, _ctx: &mut Self::Context) {
        if let Some(channel) = self.channels.get_mut(&msg.channel) {
            channel.insert(msg.id.clone(), msg.addr);
        } else {
            let mut channel = HashMap::new();
            channel.insert(msg.id.clone(), msg.addr);
            self.channels.insert(msg.channel.clone(), channel);
        }

        if let Some(channel) = self.channels.get(&msg.channel) {
            if let Some(addr) = channel.get(&msg.id) {
                addr.do_send(Message("Server: Joined".to_string()));
            }
        }
    }
}

impl Handler<Disconnect> for ChatServer {
    type Result = ();

    fn handle(&mut self, msg: Disconnect, _ctx: &mut Self::Context) {
        if let Some(channel) = self.channels.get_mut(&msg.channel) {
            channel.remove(&msg.id);

            if channel.is_empty() {
                self.channels.remove(&msg.channel);
            }
        }       
    }
}

impl Handler<ClientMessage> for ChatServer {
    type Result = ();

    fn handle(&mut self, msg: ClientMessage, _ctx: &mut Self::Context) {
        self.send_message(&msg.channel, &msg.msg);
    }
}
