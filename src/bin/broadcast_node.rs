use rsecho::*;

use anyhow::Context;
use serde::{Deserialize, Serialize};
use std::{
    collections::{HashMap, HashSet},
    io::StdoutLock,
    thread,
    time::Duration,
};

#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(tag = "type")]
#[serde(rename_all = "snake_case")]
enum Payload {
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
    Gossip {
        seen: HashSet<usize>,
    },
}

pub enum InjectedPayload {
    Gossip,
}

struct BroadcastNode {
    node: String,
    neighbourhood: Vec<String>,
    id: usize,
    messages: HashSet<usize>,
    known: HashMap<String, HashSet<usize>>,
}

impl Node<Payload, (), InjectedPayload> for BroadcastNode {
    fn from_init(
        init: rsecho::Init,
        _state: (),
        tx: std::sync::mpsc::Sender<Event<Payload, InjectedPayload>>,
    ) -> anyhow::Result<Self> {
        thread::spawn(move || loop {
            std::thread::sleep(Duration::from_millis(300));
            if tx.send(Event::Injected(InjectedPayload::Gossip)).is_err() {
                break;
            }
        });
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
        })
    }
    fn send(
        &mut self,
        input: Event<Payload, InjectedPayload>,
        output: &mut StdoutLock,
    ) -> anyhow::Result<()> {
        match input {
            Event::Injected(payload) => match payload {
                InjectedPayload::Gossip => {
                    for n in &self.neighbourhood {
                        let n_known = &self.known[n];
                        Message {
                            src: self.node.clone(),
                            dst: n.clone(),
                            body: Body {
                                id: None,
                                in_reply_to: None,
                                payload: Payload::Gossip {
                                    seen: self
                                        .messages
                                        .iter()
                                        .copied()
                                        .filter(|m| !n_known.contains(m))
                                        .collect(),
                                },
                            },
                        }
                        .send_gossip(&mut *output)
                        .with_context(|| format!("Gossip to {}", n))?;
                        self.id += 1;
                    }
                }
            },
            Event::Message(input) => {
                let mut res = input.into_reply(Some(&mut self.id));
                match res.body.payload {
                    Payload::Gossip { seen } => {
                        self.messages.extend(seen);
                    }
                    Payload::Broadcast { message } => {
                        self.messages.insert(message);
                        res.body.payload = Payload::BroadcastOk;
                        res.send_gossip(&mut *output)
                            .context("reply to broadcast")?;
                    }
                    Payload::Read => {
                        res.body.payload = Payload::ReadOk {
                            messages: self.messages.clone(),
                        };
                        res.send_gossip(&mut *output).context("reply to read")?;
                    }
                    Payload::Topology { mut topology } => {
                        self.neighbourhood = topology
                            .remove(&self.node)
                            .unwrap_or_else(|| panic!("no topology given to node {}", self.node));
                        res.body.payload = Payload::TopologyOk;
                        res.send_gossip(&mut *output).context("reply to topology")?;
                    }
                    Payload::BroadcastOk | Payload::ReadOk { .. } | Payload::TopologyOk => {}
                }
            }
            Event::EOF => {}
        }
        Ok(())
    }
}

fn main() -> anyhow::Result<()> {
    main_loop::<_, BroadcastNode, _, _>(())
}
