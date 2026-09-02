use anyhow::{Context, bail};
use serde::{Deserialize, Serialize};
use std::io::{StdoutLock, Write};

#[derive(Serialize, Deserialize)]
struct Message {
    // client node (from malstrom client)
    src: String,
    // server node
    #[serde(rename = "dest")]
    dst: String,
    body: Body,
}

#[derive(Serialize, Deserialize)]
struct Body {
    // outgoing msg_id
    #[serde(rename = "msg_id")]
    id: Option<usize>,
    // incoming message_id
    in_reply_to: Option<usize>,
    #[serde(flatten)]
    payload: Payload,
}

// abstract Payload into type (snake_case)
#[derive(Serialize, Deserialize)]
#[serde(tag = "type")]
#[serde(rename_all = "snake_case")]
enum Payload {
    Init {
        node_id: String,
        // all nodes including our node
        node_ids: Vec<String>,
    },
    InitOk,
    // echo to the node
    // id will be abstracted into the Body & used in Node
    Echo {
        echo: String,
    },
    // return echo
    EchoOk {
        echo: String,
    },
}

// for reply_msg
struct EchoState {
    // outgoing msg_id
    id: usize,
}

// our node to send reply msg to client node
// input: deserialized format (stdin)
// output: serialized format (stdout)
impl EchoState {
    fn send(&mut self, input: Message, mut output: StdoutLock) -> anyhow::Result<()> {
        match input.body.payload {
            Payload::Init { .. } => {
                let reply_msg = Message {
                    src: input.dst,
                    dst: input.src,
                    body: Body {
                        id: Some(self.id),
                        // incoming msg
                        in_reply_to: input.body.id,
                        payload: Payload::InitOk,
                    },
                };
                // deserialize msg into JSON to output
                // or use reply_msg.serialize(output)
                serde_json::to_writer(&mut output, &reply_msg)
                    .context("deserializing reply msg")?;
                output.write_all(b"\n").context("writiing to stdout")?;
                // try increment msg_id
                self.id += 1;
            }
            // should NOT send back init ok to maelstrom
            Payload::InitOk => {
                bail!("Not expecting init_ok msg to send back")
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
    // get StdinLock
    let stdin_handle = std::io::stdin().lock();
    let inputs = serde_json::Deserializer::from_reader(stdin_handle).into_iter::<Message>();
    let mut echo_state = EchoState { id: 0 };
    for input in inputs {
        let input = input.expect("deserializing reply msg");
        // get StdoutLock
        // or use let mut output = serde_json::Serializer::new(stdout_handle)
        let stdout_handle = std::io::stdout().lock();
        // send msg to stdout
        echo_state
            .send(input, stdout_handle)
            .context("sending reply msg failed")?
    }
    Ok(())
}
