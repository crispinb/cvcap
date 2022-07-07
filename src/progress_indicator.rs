use std::error::Error;
use std::fmt;
use std::io::{stdout, Write};
use std::sync::mpsc::{self, Sender};
use std::thread::{self, JoinHandle};

type Result<T> = std::result::Result<T, Box<dyn Error + Send + Sync + 'static>>;
// type Result<T> = std::result::Result<T, Box<dyn Error + 'static>>;

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

    pub fn run<F>(mut self, mut f: F) -> Result<()>
    where
        F: FnMut() -> Result<()>,
    {
        self.start()?;
        let r = f();
        self.stop()?;
        r
    }

    fn start(&mut self) -> Result<()> {
        print!("{}", self.start_message);
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

    fn stop(self) -> Result<()> {
        const ERROR_MESSAGE: &str =
            "Something went wrong stopping progress indicator thread";
        self.tx
            .as_ref()
            // seems OK here - not sure how it fails?
            .unwrap()
            .send(())
            .map_err(|_e| ERROR_MESSAGE)?;

        //.join doesn't return an error trait implementation so we create an ad
        //hoc one that in this context requires a hinted coercion bcs compiler
        //types the closure from its params & return. Latter isn't a dyn.
        // see https://stackoverflow.com/a/69500996/445929
        self.handle.unwrap().join().map_err(|_e| {
            Box::new(AdHocError {
                error: ERROR_MESSAGE.into(),
            }) as _
        })
    }
}

#[derive(Debug)]
struct AdHocError {
    error: String,
}
impl fmt::Display for AdHocError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.error)
    }
}

impl Error for AdHocError {}
