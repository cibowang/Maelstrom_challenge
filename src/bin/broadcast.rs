use anyhow::Context;
use rsecho::*;
use serde::{Deserialize, Serialize};
use std::{
    collections::HashMap,
    io::{StdoutLock, Write},
};

// msg is val
#[derive(Serialize, Deserialize)]
#[serde(tag = "type")]
#[serde(rename_all = "snake_case")]
enum Payload {
    Broadcast {
        message: usize,
    },
    BroadcastOk,
    Read,
    ReadOk {
        messages: Vec<usize>,
    },
    Topology {
        topology: HashMap<String, Vec<usize>>,
    },
    TopologyOk,
}

// FIXME: will need node to record neighbour
#[derive(Serialize, Deserialize)]
struct BroadcastNode {
    id: usize,
    // 1 msg value
    msgs: Vec<usize>,
}

// construct the node
impl rsecho::Node<(), Payload> for BroadcastNode {
    fn from_init(_state: (), _init: rsecho::Init) -> anyhow::Result<Self> {
        Ok(Self {
            id: 1,
            msgs: Vec::new(),
        })
    }
    // use input to construct reply msg from lib.rs & match on msg payload
    // then construct the return msg from reply msg with corresponding payload
    fn send(&mut self, input: Message<Payload>, output: &mut StdoutLock) -> anyhow::Result<()> {
        let mut reply_msg = input.into_reply(Some(&mut self.id));
        match reply_msg.body.payload {
            Payload::Broadcast { message } => {
                // push known msg to boradcast msgs..
                self.msgs.push(message);
                reply_msg.body.payload = Payload::BroadcastOk;
                serde_json::to_writer(&mut *output, &reply_msg)
                    .context("deserializing reply msg")?;
                output.write_all(b"\n").context("writing to stdout")?;
                self.id += 1;
            }
            Payload::Read => {
                reply_msg.body.payload = Payload::ReadOk {
                    messages: self.msgs.clone(),
                };
                serde_json::to_writer(&mut *output, &reply_msg)
                    .context("deserializing reply msg")?;
                output.write_all(b"\n").context("writing to stdout")?;
                self.id += 1;
            }
            Payload::Topology { .. } => {
                reply_msg.body.payload = Payload::TopologyOk;
                serde_json::to_writer(&mut *output, &reply_msg)
                    .context("deserializing reply msg")?;
                output.write_all(b"\n").context("writing to stdout")?;
                self.id += 1;
            }
            Payload::BroadcastOk => {}
            Payload::ReadOk { .. } => {}
            Payload::TopologyOk => {}
        }
        Ok(())
    }
}

// 0 reserved for init_msg
fn main() -> anyhow::Result<()> {
    main_loop::<_, BroadcastNode, _>(())
}
