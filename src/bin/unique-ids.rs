use anyhow::{Context, bail};
use rsecho::*;
use serde::{Deserialize, Serialize};
use std::io::{StdoutLock, Write};
use ulid::Ulid;

// need to use dedicated bin file name
#[derive(Serialize, Deserialize)]
#[serde(tag = "type")]
#[serde(rename_all = "snake_case")]
enum Payload {
    Init {
        node_id: String,
        node_ids: Vec<String>,
    },
    InitOk,
    Generate,
    // need to specify as guid
    GenerateOk {
        #[serde(rename = "id")]
        guid: String,
    },
}

// define echo node with msg_id
#[derive(Serialize, Deserialize)]
struct UniqueNode {
    id: usize,
}

impl rsecho::State<Payload> for UniqueNode {
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
            Payload::Generate => {
                let ulid = Ulid::generate().to_string();
                let reply_msg = Message {
                    src: input.dst,
                    dst: input.src,
                    body: Body {
                        id: Some(self.id),
                        in_reply_to: input.body.id,
                        payload: Payload::GenerateOk { guid: ulid },
                    },
                };
                serde_json::to_writer(&mut output, &reply_msg)
                    .context("deserializing reply msg")?;
                output.write_all(b"\n").context("writing to stdout")?;
                self.id += 1;
            }
            // do nothing when rcvd echo_ok
            Payload::GenerateOk { .. } => {}
        }
        Ok(())
    }
}

fn main() -> anyhow::Result<()> {
    main_loop(UniqueNode { id: 0 })
}
