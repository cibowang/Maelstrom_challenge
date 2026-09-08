use anyhow::Context;
use rsecho::*;
use serde::{Deserialize, Serialize};
use std::io::{StdoutLock, Write};
use ulid::Ulid;

#[derive(Serialize, Deserialize)]
#[serde(tag = "type")]
#[serde(rename_all = "snake_case")]
enum Payload {
    Generate,
    // need to specify as guid
    GenerateOk {
        #[serde(rename = "id")]
        guid: String,
    },
}

//impl rsecho::Payload for Payload {
//    fn extract_init(input: Self) -> Option<Init> {
//        let Payload::Init(init) = input else {
//            return None;
//        };
//        Some(init)
//    }
//
//    fn extract_init_ok() -> Self {
//        Payload::InitOk
//    }
//}

// define unique node with msg_id
// node should NEVER BE NONE (since always know we can get init_ok_msg in the 1st place)
#[derive(Serialize, Deserialize)]
struct UniqueNode {
    node: String,
    id: usize,
}

impl Node<(), Payload> for UniqueNode {
    fn from_init(_state: (), init: rsecho::Init) -> anyhow::Result<Self> {
        Ok(UniqueNode {
            node: init.node_id,
            id: 1,
        })
    }
    fn send(&mut self, input: Message<Payload>, mut output: &mut StdoutLock) -> anyhow::Result<()> {
        match input.body.payload {
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

// 0 reserved for init_msg
fn main() -> anyhow::Result<()> {
    main_loop::<_, UniqueNode, _>(())
}
