use anyhow::{Context, bail};
use rsecho::*;
use serde::{Deserialize, Serialize};
use std::io::{StdoutLock, Write};

#[derive(Serialize, Deserialize)]
#[serde(tag = "type")]
#[serde(rename_all = "snake_case")]
enum Payload {
    Init {
        node_id: String,
        node_ids: Vec<String>,
    },
    InitOk,
    Echo {
        echo: String,
    },
    EchoOk {
        echo: String,
    },
}

// define echo node with msg_id
#[derive(Serialize, Deserialize)]
struct EchoNode {
    id: usize,
}

impl rsecho::State<Payload> for EchoNode {
    fn send(&mut self, input: Message<Payload>, mut output: StdoutLock) -> anyhow::Result<()> {
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
            Payload::InitOk => {
                bail!("should not rcvd init_ok msg");
            }
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
            // do nothing when rcvd echo_ok
            Payload::EchoOk { .. } => {}
        }
        Ok(())
    }
}

fn main() -> anyhow::Result<()> {
    main_loop(EchoNode { id: 0 })
}
