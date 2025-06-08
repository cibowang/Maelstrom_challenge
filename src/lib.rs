use anyhow::Context;
use serde::{de::DeserializeOwned, Deserialize, Serialize};
use std::{
    io::{BufRead, StdoutLock, Write},
    panic,
};

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Message<Payload> {
    // Node id
    pub src: String,
    #[serde(rename = "dest")]
    pub dst: String,
    pub body: Body<Payload>,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Body<Payload> {
    // msg id
    #[serde(rename = "msg_id")]
    pub id: Option<usize>,
    pub in_reply_to: Option<usize>,
    #[serde(flatten)]
    pub payload: Payload,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Init {
    pub node_id: String,
    pub node_ids: Vec<String>,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(tag = "type")]
#[serde(rename_all = "snake_case")]
pub enum InitPayload {
    Init(Init),
    InitOk,
}

pub trait Node<Payload, S> {
    fn from_init(init: Init, state: S) -> anyhow::Result<Self>
    where
        Self: Sized;
    fn send(&mut self, input: Message<Payload>, output: &mut StdoutLock) -> anyhow::Result<()>;
}

pub fn main_loop<S, N, P>(init_state: S) -> anyhow::Result<()>
where
    N: Node<P, S>,
    P: DeserializeOwned,
{
    let stdin = std::io::stdin().lock();
    let mut stdin = stdin.lines();
    let mut stdout = std::io::stdout().lock();

    let init_msg: Message<InitPayload> = serde_json::from_str(
        &stdin
            .next()
            .expect("no init msg rcvd")
            .context("failed to read init msg from stdin")?,
    )
    .context("init msg could not be deserialized ")?;
    let InitPayload::Init(init) = init_msg.body.payload else {
        panic!("1st mdg should be init")
    };

    let mut node: N = Node::from_init(init, init_state).context("node init failed")?;
    let res = Message {
        src: init_msg.dst,
        dst: init_msg.src,
        body: Body {
            id: Some(0),
            in_reply_to: init_msg.body.id,
            payload: InitPayload::InitOk,
        },
    };
    serde_json::to_writer(&mut stdout, &res).context("Serialize response to echo")?;
    stdout
        .write_all(b"\n")
        .context("Writing tailing new line")?;

    for line in stdin {
        let line = line.context("Maelstrom input from stdin cannot be read")?;
        let input: Message<P> = serde_json::from_str(&line)
            .context("Maelstrom input from stdin cannot be deserialized")?;
        node.send(input, &mut stdout)
            .context("Having probelm sending messages")?;
    }
    Ok(())
}
