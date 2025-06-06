use rsecho::*;

use anyhow::{bail, Context};
use serde::{Deserialize, Serialize};
use std::io::{StdoutLock, Write};

#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(tag = "type")]
#[serde(rename_all = "snake_case")]
pub enum Payload {
    Generate,
    GenerateOk {
        #[serde(rename = "id")]
        guid: String,
    },
    Init {
        node_id: String,
        node_ids: Vec<String>,
    },
    InitOk,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
struct UNode {
    // msg_id
    id: usize,
}

impl Node<Payload> for UNode {
    fn send(&mut self, input: Message<Payload>, output: &mut StdoutLock) -> anyhow::Result<()> {
        match input.body.payload {
            Payload::Generate => {
                let guid = ulid::Ulid::new().to_string();
                let res = Message {
                    src: input.dst,
                    dst: input.src,
                    body: Body {
                        id: Some(self.id),
                        in_reply_to: input.body.id,
                        payload: Payload::GenerateOk { guid },
                    },
                };
                serde_json::to_writer(&mut *output, &res).context("Serialize response to echo")?;
                output
                    .write_all(b"\n")
                    .context("Writing tailing new line")?;
                self.id += 1
            }
            Payload::Init { .. } => {
                let res = Message {
                    src: input.dst,
                    dst: input.src,
                    body: Body {
                        id: Some(self.id),
                        in_reply_to: input.body.id,
                        payload: Payload::InitOk,
                    },
                };
                serde_json::to_writer(&mut *output, &res).context("Serialize response to init")?;
                output
                    .write_all(b"\n")
                    .context("Writing tailing new line")?;
                self.id += 1
            }
            Payload::GenerateOk { .. } => {}
            Payload::InitOk => bail!("Received InitOk message"),
        }
        Ok(())
    }
}

fn main() -> anyhow::Result<()> {
    main_loop(UNode { id: 0 })
}
