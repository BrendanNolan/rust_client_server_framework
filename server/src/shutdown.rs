use tokio::sync::watch;

#[derive(Clone)]
pub struct ShutdownListener {
    shutting_down: bool,
    shutdown_notification_receiver: watch::Receiver<()>,
}

impl ShutdownListener {
    pub fn new(shutdown_signal_receiver: watch::Receiver<()>) -> Self {
        Self {
            shutting_down: false,
            shutdown_notification_receiver: shutdown_signal_receiver,
        }
    }

    pub fn is_shutting_down(&self) -> bool {
        self.shutting_down
    }

    pub async fn listen_for_shutdown_signal(&mut self) {
        if self.shutting_down {
            return;
        }
        let _ = self.shutdown_notification_receiver.changed().await;
        self.shutting_down = true;
    }
}
