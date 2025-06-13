use rsecho::*;

use anyhow::Context;
use serde::{Deserialize, Serialize};
use std::io::{StdoutLock, Write};

#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(tag = "type")]
#[serde(rename_all = "snake_case")]
pub enum Payload {
    Echo { echo: String },
    EchoOk { echo: String },
}

#[derive(Serialize, Deserialize, Debug, Clone)]
struct ENode {
    // msg_id
    id: usize,
}

impl Node<Payload, ()> for ENode {
    fn from_init(
        _init: rsecho::Init,
        _state: (),
        _tx: std::sync::mpsc::Sender<Event<Payload>>,
    ) -> anyhow::Result<Self> {
        Ok(ENode { id: 1 })
    }
    fn send(&mut self, input: Event<Payload>, output: &mut StdoutLock) -> anyhow::Result<()> {
        let Event::Message(input) = input else {
            panic!("Expecting injection but got none")
        };
        let mut res = input.into_reply(Some(&mut self.id));
        match res.body.payload {
            Payload::Echo { echo } => {
                res.body.payload = Payload::EchoOk { echo };
                serde_json::to_writer(&mut *output, &res).context("Serialize response to echo")?;
                output
                    .write_all(b"\n")
                    .context("Writing tailing new line")?;
            }
            Payload::EchoOk { .. } => {}
        }
        Ok(())
    }
}

fn main() -> anyhow::Result<()> {
    main_loop::<_, ENode, _, _>(())
}
