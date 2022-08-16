use std::io::{stdout, Write};
use std::sync::mpsc;
use std::thread;

use anyhow::{anyhow, Result};

pub struct ProgressIndicator<'a> {
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
            do_before,
            progress_char,
            display_interval_ms: interval,
        }
    }

    pub fn run<F>(mut self, job: F) -> Result<()>
    where
        F: FnOnce() -> Result<()>,
    {
        (self.do_before)();

        let (tx, rx) = mpsc::channel::<()>();

        thread::scope(|s| {
            s.spawn(move || loop {
                if rx.try_recv().is_ok() {
                    stdout().flush().expect("Couldn't flush stdout");
                    break;
                };
                print!("{}", self.progress_char);
                stdout().flush().expect("Couldn't flush stdout");
                thread::sleep(std::time::Duration::from_millis(self.display_interval_ms));
            });

            let result = job();

            tx.send(()).map_err(|_e| {
                anyhow!(String::from(
                    "Something went wrong stopping progress indicator thread"
                ))
            })?;

            result
        })
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
