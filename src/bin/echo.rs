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
    Echo { echo: String },
    EchoOk { echo: String },
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

#[derive(Serialize, Deserialize)]
struct EchoNode {
    id: usize,
}

impl rsecho::Node<(), Payload> for EchoNode {
    fn from_init(_state: (), _init: rsecho::Init) -> anyhow::Result<Self> {
        Ok(EchoNode { id: 1 })
    }
    fn send(&mut self, input: Message<Payload>, mut output: &mut StdoutLock) -> anyhow::Result<()> {
        match input.body.payload {
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
}

// 0 reserved for init_msg
fn main() -> anyhow::Result<()> {
    main_loop::<_, EchoNode, _>(())
}
