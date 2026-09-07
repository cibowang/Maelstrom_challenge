use anyhow::Context;
use serde::{Deserialize, Serialize, de::DeserializeOwned};
use std::io::{StdoutLock, Write};

#[derive(Serialize, Deserialize)]
pub struct Message<Payload> {
    pub src: String,
    #[serde(rename = "dest")]
    pub dst: String,
    pub body: Body<Payload>,
}

/* Node used to ONLY have <Payload> as generic type, now it has a State
 * Init logic abstracted into enum to get Init
 * also add init_payload extraction logic to Node, so that unique_node can have a global unique id
 * lastly write to the next line in stdout buffer
 * */
#[derive(Serialize, Deserialize)]
pub struct Body<Payload> {
    #[serde(rename = "msg_id")]
    pub id: Option<usize>,
    pub in_reply_to: Option<usize>,
    #[serde(flatten)]
    pub payload: Payload,
}

// used in echo to be extracted
// FIXME: abstract init logic herein
#[derive(Serialize, Deserialize)]
pub struct Init {
    node_id: String,
    node_ids: Vec<String>,
}

// define init payload to extract Init
// this will be retained after commit 12th
#[derive(Serialize, Deserialize)]
#[serde(tag = "type")]
#[serde(rename_all = "snake_case")]
pub enum InitPayload {
    Init(Init),
    InitOk,
}

// NEW: extract Some(Init) from init_payload (input Message) -> use let..else
// NEW: extract Init_Ok_msg
// FIXME: abstract Init logic herein
pub trait Payload: Sized {
    fn extract_init(input: Self) -> Option<Init>;
    fn extract_init_ok() -> Self;
}

// NEW: construct state from Init (S as the new bound)
pub trait Node<S, Payload>: Sized {
    fn from_init(state: S, init: Init) -> anyhow::Result<Self>;
    fn send(&mut self, input: Message<Payload>, output: &mut StdoutLock) -> anyhow::Result<()>;
}

// extract Init & write to buffer
// FIXME: defer to get a single input from a concrete type param (instead of from P)
pub fn main_loop<S, N, P>(state: S) -> anyhow::Result<()>
where
    P: Payload + DeserializeOwned + Serialize,
    N: Node<S, P>,
{
    let stdin_handle = std::io::stdin().lock();
    let mut stdout_handle = std::io::stdout().lock();
    let mut inputs = serde_json::Deserializer::from_reader(stdin_handle).into_iter::<Message<P>>();
    let init_msg = inputs
        .next()
        .expect("init msg should always present")
        .context("failed to deserialize init msg")?;
    let init = P::extract_init(init_msg.body.payload).expect("1st payload should be Init");
    let init_ok = P::extract_init_ok();
    let reply_msg = Message {
        src: init_msg.dst,
        dst: init_msg.src,
        body: Body {
            // reserved for InitOk
            id: Some(0),
            in_reply_to: init_msg.body.id,
            payload: init_ok,
        },
    };
    serde_json::to_writer(&mut stdout_handle, &reply_msg).context("serializing reply msg")?;
    let _ = &mut stdout_handle
        .write_all(b"\n")
        .context("writing to the next line of stdout")?;
    let mut node: N = Node::from_init(state, init)?;
    for input in inputs {
        let input = input.context("abc")?;
        let _ = node.send(input, &mut stdout_handle);
    }
    Ok(())
}
