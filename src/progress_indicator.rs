use std::io::{stdout, Write};
use std::sync::mpsc::{self, Receiver, Sender};
use std::sync::Arc;
use std::thread::{self, JoinHandle};

// TODO: wrapper api of some sort - function? macro?
pub struct ProgressIndicator {
    tx: Option<Sender<()>>,
    handle: Option<JoinHandle<()>>,
    display_char: String,
    bye_message: String,
    display_interval_ms: u16,
}

impl ProgressIndicator {
    pub fn new(display_char: &str, bye_message: &str, interval: u16) -> ProgressIndicator {
        ProgressIndicator {
            tx: None,
            handle: None,
            bye_message: bye_message.into(),
            display_char: display_char.into(),
            display_interval_ms: interval,
        }
    }

    pub fn start(&mut self) -> Result<(), std::io::Error> {
        let (tx, rx) = mpsc::channel::<()>();
        self.tx = Some(tx);
        let interval: u64 = self.display_interval_ms.into();
        let bye_msg = self.bye_message.clone();
        let display_char = self.display_char.clone();
        let handle = thread::Builder::new()
            .name(String::from("SimpleProgressIndicator-Thread"))
            .spawn(move || loop {
                if rx.try_recv().is_ok() {
                    println!("\n{}", bye_msg);
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

    pub fn stop(self) -> Result<(), String> {
        let error_message = String::from("Something went wrong stopping progress indicator thread");
        self.tx
            .as_ref()
            // unwrap seems OK here - not sure how it fails?
            .unwrap()
            .send(())
            .map_err(|e| error_message.clone())?;

        self.handle
            .unwrap()
            .join()
            .map_err(|e| error_message)
    }
}
