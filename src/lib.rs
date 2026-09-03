use anyhow::Context;
use serde::{Deserialize, Serialize, de::DeserializeOwned};
use std::io::StdoutLock;

/* Payload as generic type param (DeserializedOwned)
 * its enum still need to be defined in diff ctx
 * */

#[derive(Serialize, Deserialize)]
pub struct Message<Payload> {
    pub src: String,
    #[serde(rename = "dest")]
    pub dst: String,
    pub body: Body<Payload>,
}

#[derive(Serialize, Deserialize)]
pub struct Body<Payload> {
    #[serde(rename = "msg_id")]
    pub id: Option<usize>,
    pub in_reply_to: Option<usize>,
    #[serde(flatten)]
    pub payload: Payload,
}

// abstract away init node
#[derive(Serialize, Deserialize)]
pub struct InitNode {
    pub node_id: String,
    pub node_ids: Vec<String>,
}

// state machine
pub trait State<Payload> {
    fn send(&mut self, input: Message<Payload>, output: StdoutLock) -> anyhow::Result<()>;
}

// state machine on Node
pub fn main_loop<S, Payload>(mut state: S) -> anyhow::Result<()>
where
    S: State<Payload>,
    Payload: DeserializeOwned,
{
    // get StdinLock
    let stdin_handle = std::io::stdin().lock();
    let inputs =
        serde_json::Deserializer::from_reader(stdin_handle).into_iter::<Message<Payload>>();
    for input in inputs {
        let input = input.expect("deserializing reply msg");
        // get StdoutLock
        // or use let mut output = serde_json::Serializer::new(stdout_handle)
        let stdout_handle = std::io::stdout().lock();
        // send msg to stdout
        state
            .send(input, stdout_handle)
            .context("sending reply msg failed")?
    }
    Ok(())
}
