use iced::{
    futures::{channel::mpsc, SinkExt},
    subscription::{self, Subscription},
};

use crate::{manager_message::ManagerMessage, output::Output, vault::manager::VaultManager};
#[derive(Debug, Clone)]
pub enum Event {
    Connected(Connection),
    Disconnected,
    ReceiveOutput(Output),
    // ReceiveSchema(Schema),
    // ReceiveRead(Reads<Store>),
    // ReceiveList(Vec<String>),
    // ReceiveBackup(BackupFile),
    // ReceiveBackupFiles(Vec<BackupFile>),
    // ReceiveNothing,
}

impl From<Output> for Event {
    fn from(value: Output) -> Self {
        Self::ReceiveOutput(value)
        // match value {
        //     Output::Schema(s) => Self::ReceiveSchema(s),
        //     Output::Read(r) => Self::ReceiveRead(r),
        //     Output::List(l) => Self::ReceiveList(l),
        //     Output::Backup(b) => Self::ReceiveBackup(b),
        //     Output::BackupFiles(f) => Self::ReceiveBackupFiles(f),
        //     Output::Nothing => Self::ReceiveNothing,
        // }
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
pub fn connect() -> Subscription<Event> {
    struct Connect;
    let mut interface = VaultManager::new();
    subscription::channel(
        std::any::TypeId::of::<Connect>(),
        100,
        |mut output| async move {
            let mut state = State::Starting;

            loop {
                match &mut state {
                    State::Starting => {
                        let (sender, receiver) = mpsc::channel(100);

                        let _ = output.send(Event::Connected(Connection(sender))).await;
                        state = State::Connected(receiver);
                    }
                    State::Connected(receiver) => {
                        use iced_futures::futures::StreamExt;

                        let input = receiver.select_next_some().await;

                        let response = interface.receive(input);

                        if let Ok(vault_output) = response {
                            let event = vault_output.into();
                            let _ = output.send(event).await;
                        }
                    }
                }
            }
        },
    )
}
