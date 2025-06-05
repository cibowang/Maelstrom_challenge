use anyhow::Context;
use serde::{de::DeserializeOwned, Deserialize, Serialize};
use std::io::StdoutLock;

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
    node_id: String,
    node_ids: Vec<String>,
}

pub trait Node<Payload> {
    fn send(&mut self, input: Message<Payload>, output: &mut StdoutLock) -> anyhow::Result<()>;
}

pub fn main_loop<S, Payload>(mut state: S) -> anyhow::Result<()>
where
    S: Node<Payload>,
    Payload: DeserializeOwned,
{
    let stdin = std::io::stdin().lock();
    let mut stdout = std::io::stdout().lock();

    let inputs = serde_json::Deserializer::from_reader(stdin).into_iter::<Message<Payload>>();
    for input in inputs {
        let input = input.context("Cannot deserialize input message")?;
        state
            .send(input, &mut stdout)
            .context("Having probelm sending messages")?;
    }
    Ok(())
}
