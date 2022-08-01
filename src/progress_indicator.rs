use anyhow::{anyhow, Result};
use std::io::{stdout, Write};
use std::sync::mpsc::{self, Sender};
use std::thread::{self, JoinHandle};

// TODO: wrapper api of some sort - function? macro?
pub struct ProgressIndicator {
    tx: Option<Sender<()>>,
    handle: Option<JoinHandle<()>>,
    start_message: String,
    display_char: String,
    bye_message: String,
    display_interval_ms: u16,
}

impl ProgressIndicator {
    pub fn new(
        display_char: &str,
        start_message: &str,
        bye_message: &str,
        interval: u16,
    ) -> ProgressIndicator {
        ProgressIndicator {
            tx: None,
            handle: None,
            start_message: start_message.into(),
            bye_message: bye_message.into(),
            display_char: display_char.into(),
            display_interval_ms: interval,
        }
    }

    pub fn start(&mut self) -> Result<()> {
        print!("{}", self.start_message);
        let (tx, rx) = mpsc::channel::<()>();
        self.tx = Some(tx);
        let interval: u64 = self.display_interval_ms.into();
        let display_char = self.display_char.clone();
        let handle = thread::Builder::new()
            .name(String::from("SimpleProgressIndicator-Thread"))
            .spawn(move || loop {
                if rx.try_recv().is_ok() {
                    stdout().flush().expect("Something went badly wrong");
                    break;
                };
                print!("{}", display_char);
                stdout().flush().expect("Something went badly wrong");
                thread::sleep(std::time::Duration::from_millis(interval));
            })?;
        self.handle = Some(handle);
        Ok(())
    }

    pub fn stop(self, was_success: bool) -> Result<()> {
        let error_message = String::from("Something went wrong stopping progress indicator thread");
        self.tx
            .as_ref()
            // unwrap seems OK here - not sure how it fails?
            .unwrap()
            .send(())
            .map_err(|_e| anyhow!(error_message.clone()))?;

        if was_success {println!("\n{}", self.bye_message)};

        self.handle.unwrap().join().map_err(|_e| anyhow!(error_message))
    }
}
