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
    Init(rsecho::Init),
    InitOk,
    Generate,
    // need to specify as guid
    GenerateOk {
        #[serde(rename = "id")]
        guid: String,
    },
}

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

// define echo node with msg_id
#[derive(Serialize, Deserialize)]
struct UniqueNode {
    id: usize,
}

impl Node<(), Payload> for UniqueNode {
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

    fn from_init(_state: (), _init: Init) -> anyhow::Result<Self> {
        todo!()
    }
}

fn main() -> anyhow::Result<()> {
    main_loop(UniqueNode { id: 0 })
}
