use anyhow::bail;
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

//
#[derive(Serialize, Deserialize)]
struct Body {
    // new msg_id
    #[serde(rename = "msg_id")]
    id: Option<usize>,
    // original message_id
    in_reply_to: Option<usize>,
    #[serde(flatten)]
    payload: Payload,
}

#[derive(Serialize, Deserialize)]
enum Payload {
    Init {
        id: usize,
        node_id: String,
        node_ids: Vec<String>,
    },
    InitOk,
    // echo to the node
    // id will be abstracted into the Body & Node
    Echo {
        echo: String,
    },
    EchoOk {
        echo: String,
    },
}

// for reply_msg
struct EchoState {
    // new msg_id
    id: usize,
}

// serer_node to send reply msg to client node as per the incoming msg
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
                        in_reply_to: input.body.id,
                        payload: Payload::InitOk,
                    },
                };
                // deserialize msg into JSON to output
                serde_json::to_writer(&mut output, &reply_msg)?;
                output.write_all(b"\n")?;
                // try increment msg_id
                self.id += 1;
            }
            // should NOT send back init ok to maelstrom
            Payload::InitOk => {
                bail!("Not expect to send back init_ok")
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
                serde_json::to_writer(&mut output, &reply_msg)?;
                output.write_all(b"\n")?;
                self.id += 1;
            }
            // do nothing when rcvd echo_ok msg
            Payload::EchoOk { .. } => {}
        }
        Ok(())
    }
}

fn main() -> anyhow::Result<()> {
    // StdinLock
    let stdin_handle = std::io::stdin().lock();
    // StdoutLock
    let inputs = serde_json::Deserializer::from_reader(stdin_handle).into_iter::<Message>();
    let mut echo_state = EchoState { id: 0 };
    for input in inputs {
        let input = input.expect("the input no fail");
        let stdout_handle = std::io::stdout().lock();
        // send msg to stdout
        echo_state.send(input, stdout_handle)?
    }
    Ok(())
}
