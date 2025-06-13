use anyhow::Context;
use serde::{de::DeserializeOwned, Deserialize, Serialize};
use std::{
    io::{BufRead, StdoutLock, Write},
    panic, thread,
};

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Message<Payload> {
    // Node id
    pub src: String,
    #[serde(rename = "dest")]
    pub dst: String,
    pub body: Body<Payload>,
}

impl<Payload> Message<Payload> {
    pub fn into_reply(self, id: Option<&mut usize>) -> Self {
        Self {
            src: self.dst,
            dst: self.src,
            body: Body {
                id: id.map(|id| {
                    let mid = *id;
                    *id += 1;
                    mid
                }),
                in_reply_to: self.body.id,
                payload: self.body.payload,
            },
        }
    }
    pub fn send_gossip(&self, output: &mut impl Write) -> anyhow::Result<()>
    where
        Payload: Serialize,
    {
        serde_json::to_writer(&mut *output, self).context("Serialize response to echo")?;
        output
            .write_all(b"\n")
            .context("Writing tailing new line")?;
        Ok(())
    }
}

#[derive(Debug, Clone)]
pub enum Event<Payload, InjectedPayload = ()> {
    Message(Message<Payload>),
    Injected(InjectedPayload),
    EOF,
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

pub trait Node<Payload, S, InjectedPayload = ()> {
    fn from_init(
        init: Init,
        _state: S,
        inject: std::sync::mpsc::Sender<Event<Payload, InjectedPayload>>,
    ) -> anyhow::Result<Self>
    where
        Self: Sized;
    fn send(
        &mut self,
        input: Event<Payload, InjectedPayload>,
        output: &mut StdoutLock,
    ) -> anyhow::Result<()>;
}

pub fn main_loop<S, N, P, IP>(init_state: S) -> anyhow::Result<()>
where
    N: Node<P, S, IP>,
    P: DeserializeOwned + Send + 'static,
    IP: Send + 'static,
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

    let (tx, rx) = std::sync::mpsc::channel();
    let mut node: N = Node::from_init(init, init_state, tx.clone()).context("node init failed")?;
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

    drop(stdin);

    let tx_clone = tx.clone();
    let th = thread::spawn(move || {
        let stdin = std::io::stdin().lock();
        for line in stdin.lines() {
            let line = line.context("Maelstrom input from stdin cannot be read")?;
            let input: Message<P> = serde_json::from_str(&line)
                .context("Maelstrom input from stdin cannot be deserialized")?;
            if tx_clone.send(Event::Message(input)).is_err() {
                return Ok::<_, anyhow::Error>(());
            }
        }
        let _ = tx.send(Event::EOF);
        Ok(())
    });

    for input in rx {
        node.send(input, &mut stdout)
            .context("Having probelm sending messages")?;
    }

    th.join()
        .expect("stdin thread panicked")
        .context("stdin err")?;

    Ok(())
}
