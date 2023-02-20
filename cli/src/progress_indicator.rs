use std::io::{stdout, Write};
use std::sync::mpsc::{self, Sender};
use std::thread::{self, JoinHandle};

use anyhow::{anyhow, Result};

pub struct ProgressIndicator<'a> {
    tx: Option<Sender<()>>,
    handle: Option<JoinHandle<()>>,
    do_before: Box<dyn FnMut() + 'a>,
    progress_char: char,
    display_interval_ms: u64,
}

impl ProgressIndicator<'_> {
    pub fn new<'a>(
        progress_char: char,
        do_before: Box<dyn FnMut() + 'a>,
        interval: u64,
    ) -> ProgressIndicator<'a> {
        ProgressIndicator {
            tx: None,
            handle: None,
            do_before,
            progress_char,
            display_interval_ms: interval,
        }
    }

    pub fn run<F, T>(mut self, job: F) -> Result<T>
    where
        F: FnOnce() -> Result<T>,
    {
        (self.do_before)();

        let (tx, rx) = mpsc::channel::<()>();

        self.tx = Some(tx);
        let progress_char = self.progress_char;
        let display_interval = self.display_interval_ms;
        // Not using a scoped thread, because it doesn't stop if the supplied job (on main thread)
        // panics
        // see https://users.rust-lang.org/t/why-does-a-main-thread-panic-not-exit-the-process-when-a-scoped-thread-is-running/89510/5
        let handle = thread::Builder::new()
            .name(String::from("SimpleProgressIndicator-Thread"))
            .spawn(move || loop {
                if rx.try_recv().is_ok() {
                    stdout().flush().expect("Couldn't flush stdout");
                    break;
                };
                print!("{}", progress_char);
                stdout().flush().expect("Couldn't flush stdout");
                thread::sleep(std::time::Duration::from_millis(display_interval));
            })?;
        self.handle = Some(handle);

        let result = job();
        println!();

        let error_message = String::from("Couldn't stop progress indicator thread");
        self.tx
            .as_ref()
            .ok_or_else(|| anyhow!("Self.tx is None. Was .start() never called?"))?
            .send(())
            .map_err(|_e| anyhow!(error_message.clone()))?;

        self.handle
            .ok_or_else(|| anyhow!("Self.handle is None"))?
            .join()
            .map_err(|_e| anyhow!(error_message))?;

        result
    }
}

#[cfg(test)]
mod test {
    use super::ProgressIndicator;

    #[test]
    fn supplied_closures_are_run() {
        let mut before_was_run = false;
        let mut job_was_run = false;
        let job = || {
            job_was_run = true;
            Ok(())
        };

        ProgressIndicator::new('.', Box::new(|| before_was_run = true), 10)
            .run(job)
            .unwrap();

        assert!(before_was_run);
        assert!(job_was_run);
    }
}
