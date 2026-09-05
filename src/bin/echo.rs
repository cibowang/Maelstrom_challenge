use anyhow::Context;
use rsecho::*;
use serde::{Deserialize, Serialize};
use std::io::{StdoutLock, Write};

/* move init logic into lib for better abstraction
 * echo will be based on init
 * payload enum still need to include Init logic
 * */
#[derive(Serialize, Deserialize)]
#[serde(tag = "type")]
#[serde(rename_all = "snake_case")]
enum Payload {
    Init(rsecho::Init),
    InitOk,
    Echo { echo: String },
    EchoOk { echo: String },
}

// match on Init
impl rsecho::Payload for Payload {
    fn extract_init(input: Self) -> Option<Init> {
        let Payload::Init(init) = input else {
            return None;
        };
        Some(init)
    }

    fn extract_init_ok() -> Self {
        Payload::InitOk
    }
}

#[derive(Serialize, Deserialize)]
struct EchoNode {
    id: usize,
}

// don't care about the state
// still need
impl rsecho::Node<(), Payload> for EchoNode {
    fn send(&mut self, input: Message<Payload>, output: &mut StdoutLock) -> anyhow::Result<()> {
        match input.body.payload {
            Payload::Init { .. } => {
                let reply_msg = Message {
                    src: input.dst,
                    dst: input.src,
                    body: Body {
                        id: Some(self.id),
                        in_reply_to: input.body.id,
                        payload: Payload::InitOk,
                    },
                };
                serde_json::to_writer(&mut output, &reply_msg)
                    .context("deserializing reply msg")?;
                output.write_all(b"\n").context("writing to stdout")?;
                self.id += 1;
            }
            // do nothing when rcvd echo_ok
            Payload::InitOk => {}
            Payload::Echo { echo } => {
                let reply_msg = Message {
                    src: input.dst,
                    dst: input.src,
                    body: Body {
                        id: Some(self.id),
                        in_reply_to: input.body.id,
                        payload: Payload::EchoOk { echo },
                    },
                };
                serde_json::to_writer(&mut output, &reply_msg)
                    .context("deserializing reply msg")?;
                output.write_all(b"\n").context("writing to stdout")?;
                self.id += 1;
            }
            Payload::EchoOk { .. } => {}
        }
        Ok(())
    }

    fn from_init(_state: (), _init: Init) -> anyhow::Result<Self> {
        todo!()
    }
}

fn main() -> anyhow::Result<()> {
    main_loop(EchoNode { id: 0 })
}
