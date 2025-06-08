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
    fn from_init(_init: rsecho::Init, _state: ()) -> anyhow::Result<Self> {
        Ok(ENode { id: 1 })
    }
    fn send(&mut self, input: Message<Payload>, output: &mut StdoutLock) -> anyhow::Result<()> {
        match input.body.payload {
            Payload::Echo { echo } => {
                let res = Message {
                    src: input.dst,
                    dst: input.src,
                    body: Body {
                        id: Some(self.id),
                        in_reply_to: input.body.id,
                        payload: Payload::EchoOk { echo },
                    },
                };
                serde_json::to_writer(&mut *output, &res).context("Serialize response to echo")?;
                output
                    .write_all(b"\n")
                    .context("Writing tailing new line")?;
                self.id += 1
            }
            Payload::EchoOk { .. } => {}
        }
        Ok(())
    }
}

fn main() -> anyhow::Result<()> {
    main_loop::<_, ENode, _>(())
}
