use anyhow::Result;
use crossbeam_channel::{self, TrySendError};

#[derive(Clone)]
pub struct ThreadChannel<T> {
    pub sender: crossbeam_channel::Sender<T>,
    pub receiver: crossbeam_channel::Receiver<T>,
}

impl<T> ThreadChannel<T> {
    pub fn new(
        sender: crossbeam_channel::Sender<T>,
        receiver: crossbeam_channel::Receiver<T>,
    ) -> Self {
        Self { sender, receiver }
    }

    pub fn new_pair() -> (ThreadChannel<T>, ThreadChannel<T>) {
        let (a_tx, b_rx) = crossbeam_channel::unbounded();
        let (b_tx, a_rx) = crossbeam_channel::unbounded();

        let a = ThreadChannel::new(a_tx, a_rx);
        let b = ThreadChannel::new(b_tx, b_rx);

        (a, b)
    }

    pub fn send(&self, message: T) -> Result<(), TrySendError<T>> {
        self.sender.try_send(message)
    }

    pub fn receive(&self) -> Vec<T> {
        self.receiver.try_iter().collect()
    }
}
