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

// need to be pub
#[derive(Serialize, Deserialize)]
pub struct Init {
    pub node_id: String,
    pub node_ids: Vec<String>,
}

// define init payload to extract Init
// this will be retained after commit 12th
#[derive(Serialize, Deserialize)]
#[serde(tag = "type")]
#[serde(rename_all = "snake_case")]
enum InitPayload {
    Init(Init),
    InitOk,
}

// extract Some(Init) from init_payload (input Message) -> use let..else
// extract Init_Ok_msg
// FIXME: abstract Init logic herein
//pub trait Payload: Sized {
//    fn extract_init(input: Self) -> Option<Init>;
//    fn extract_init_ok() -> Self;
//}

// construct state from Init (S as the new bound)
pub trait Node<S, Payload>: Sized {
    fn from_init(state: S, init: Init) -> anyhow::Result<Self>;
    fn send(&mut self, input: Message<Payload>, output: &mut StdoutLock) -> anyhow::Result<()>;
}

// extract Init payload & write to buffer
// defer to acquire a single input payload from a concrete type param (instead of from P)
pub fn main_loop<S, N, P>(state: S) -> anyhow::Result<()>
where
    P: DeserializeOwned + Serialize,
    N: Node<S, P>,
{
    let mut stdin_handle = std::io::stdin().lock();
    let mut stdout_handle = std::io::stdout().lock();
    // get the init_msg from stdin (channel) with the concrete msg type (Init)
    let init_msg = serde_json::Deserializer::from_reader(&mut stdin_handle)
        .into_iter::<Message<InitPayload>>()
        .next()
        .expect("no msg rcvd")
        .context("cannot deserialize msg from channel")?;
    // check it's the Init herein
    let InitPayload::Init(init) = init_msg.body.payload else {
        panic!("1st msg should ALWAYS be Init")
    };
    // if yes, construct the reply msg (with init_ok)
    let reply_msg = Message {
        src: init_msg.dst,
        dst: init_msg.src,
        body: Body {
            id: Some(0),
            in_reply_to: init_msg.body.id,
            payload: InitPayload::InitOk,
        },
    };
    // serialize the reply_msg 1st & write to stdout
    serde_json::to_writer(&mut stdout_handle, &reply_msg).context("serializing reply msg")?;
    let _ = &mut stdout_handle
        .write_all(b"\n")
        .context("writing to the next line of stdout")?;
    // construct the node
    let mut node: N = Node::from_init(state, init)?;
    // node to read into stdin & write from the stdout buffer
    let inputs = serde_json::Deserializer::from_reader(stdin_handle).into_iter::<Message<P>>();
    for input in inputs {
        let input = input.context("abc")?;
        let _ = node.send(input, &mut stdout_handle);
    }
    Ok(())
}
