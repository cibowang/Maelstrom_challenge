use rsecho::*;

use anyhow::Context;
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
}

#[derive(Serialize, Deserialize, Debug, Clone)]
struct UNode {
    node: String,
    id: usize,
}

impl Node<Payload, ()> for UNode {
    fn from_init(init: rsecho::Init, _state: ()) -> anyhow::Result<Self> {
        Ok(UNode {
            node: init.node_id,
            id: 1,
        })
    }
    fn send(&mut self, input: Message<Payload>, output: &mut StdoutLock) -> anyhow::Result<()> {
        let mut res = input.into_reply(Some(&mut self.id));
        match res.body.payload {
            Payload::Generate => {
                let guid = format!("{}-{}", self.node, self.id);
                res.body.payload = Payload::GenerateOk { guid };
                serde_json::to_writer(&mut *output, &res).context("Serialize response to echo")?;
                output
                    .write_all(b"\n")
                    .context("Writing tailing new line")?;
            }
            Payload::GenerateOk { .. } => {}
        }
        Ok(())
    }
}

fn main() -> anyhow::Result<()> {
    main_loop::<_, UNode, _>(())
}
