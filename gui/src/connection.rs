use iced::futures::{channel::mpsc, SinkExt, Stream};

use pants_store::{manager_message::ManagerMessage, output::Output, vault::manager::VaultManager};
#[derive(Debug, Clone)]
pub enum Event {
    Connected(Connection),
    Disconnected,
    ReceiveOutput(Output),
    ReceiveError(String),
}

impl From<Output> for Event {
    fn from(value: Output) -> Self {
        Self::ReceiveOutput(value)
    }
}

#[derive(Debug)]
enum State {
    Starting,
    Connected(mpsc::Receiver<ManagerMessage>),
}

#[derive(Debug, Clone)]
pub struct Connection(mpsc::Sender<ManagerMessage>);

impl Connection {
    pub fn send(&mut self, message: ManagerMessage) {
        self.0
            .try_send(message)
            .expect("Send message to echo server");
    }
}
pub fn connect() -> impl Stream<Item = Event> {
    let mut interface = VaultManager::default();
    iced::stream::channel(100, |mut output| async move {
        let mut state = State::Starting;

        loop {
            match &mut state {
                State::Starting => {
                    let (sender, receiver) = mpsc::channel(100);

                    let _ = output.send(Event::Connected(Connection(sender))).await;
                    state = State::Connected(receiver);
                }
                State::Connected(receiver) => {
                    use iced::futures::StreamExt;

                    let input = receiver.select_next_some().await;

                    let response = interface.receive(input);

                    match response {
                        Ok(vault_output) => {
                            let event = vault_output.into();
                            let _ = output.send(event).await;
                        }
                        // TODO: actually pass along errors so they can be reacted to and
                        // reported
                        Err(e) => {
                            let _ = output.send(Event::ReceiveError(e.to_string())).await;
                        }
                    }
                }
            }
        }
    })
}
