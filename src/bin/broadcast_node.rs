use rsecho::*;

use anyhow::Context;
use serde::{Deserialize, Serialize};
use std::{
    collections::{HashMap, HashSet},
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
        messages: HashSet<usize>,
    },
    Topology {
        topology: HashMap<String, Vec<String>>,
    },
    TopologyOk,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
struct BroadcastNode {
    node: String,
    neighbourhood: Vec<String>,
    id: usize,
    messages: HashSet<usize>,
    known: HashMap<String, HashSet<usize>>,
    communicated: HashMap<String, HashSet<usize>>,
}

impl Node<Payload, ()> for BroadcastNode {
    fn from_init(init: rsecho::Init, _state: ()) -> anyhow::Result<Self> {
        Ok(Self {
            neighbourhood: Vec::new(),
            node: init.node_id,
            id: 1,
            messages: HashSet::new(),
            known: init
                .node_ids
                .into_iter()
                .map(|nid| (nid, HashSet::new()))
                .collect(),
            communicated: HashMap::new(),
        })
    }
    fn send(&mut self, input: Message<Payload>, output: &mut StdoutLock) -> anyhow::Result<()> {
        let mut res = input.into_reply(Some(&mut self.id));
        match res.body.payload {
            Payload::Broadcast { message } => {
                self.messages.insert(message);
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
            Payload::Topology { mut topology } => {
                self.neighbourhood = topology
                    .remove(&self.node)
                    .unwrap_or_else(|| panic!("no topology given to node {}", self.node));
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
