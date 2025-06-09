use rsecho::*;

use anyhow::Context;
use serde::{Deserialize, Serialize};
use std::{
    collections::HashMap,
    io::{StdoutLock, Write},
};

#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(tag = "type")]
#[serde(rename_all = "snake_case")]
pub enum Payload {
    Broadcast {
        message: usize,
    },
    BroadcastOk,
    Read,
    ReadOk {
        messages: Vec<usize>,
    },
    Topology {
        topology: HashMap<String, Vec<String>>,
    },
    TopologyOk,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
struct BroadcastNode {
    node: String,
    id: usize,
    messages: Vec<usize>,
}

impl Node<Payload, ()> for BroadcastNode {
    fn from_init(init: rsecho::Init, _state: ()) -> anyhow::Result<Self> {
        Ok(Self {
            node: init.node_id,
            id: 1,
            messages: Vec::new(),
        })
    }
    fn send(&mut self, input: Message<Payload>, output: &mut StdoutLock) -> anyhow::Result<()> {
        let mut res = input.into_reply(Some(&mut self.id));
        match res.body.payload {
            Payload::Broadcast { message } => {
                self.messages.push(message);
                res.body.payload = Payload::BroadcastOk;
                serde_json::to_writer(&mut *output, &res)
                    .context("Serialize response to broadcast")?;
                output
                    .write_all(b"\n")
                    .context("Writing tailing new line")?;
            }
            Payload::Read => {
                res.body.payload = Payload::ReadOk {
                    messages: self.messages.clone(),
                };
                serde_json::to_writer(&mut *output, &res).context("Serialize response to read")?;
                output
                    .write_all(b"\n")
                    .context("Writing tailing new line")?;
            }
            Payload::Topology { topology: _ } => {
                res.body.payload = Payload::TopologyOk;
                serde_json::to_writer(&mut *output, &res)
                    .context("Serialize response to topoloy")?;
                output
                    .write_all(b"\n")
                    .context("Writing tailing new line")?;
            }
            Payload::BroadcastOk | Payload::ReadOk { .. } | Payload::TopologyOk => {}
        }
        Ok(())
    }
}

fn main() -> anyhow::Result<()> {
    main_loop::<_, BroadcastNode, _>(())
}
