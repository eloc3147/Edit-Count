use derive_new::new;
use failure::{Error, Fail};
use std::thread;

pub trait Worker
where
    Self: Sized + Send,
{
    type W: Worker;
    const NAME: &'static str;

    fn start(self) -> Result<WorkerHandle, Error>
    where
        Self: 'static,
    {
        let handle = thread::Builder::new()
            .name(Self::NAME.to_string())
            .spawn(move || self.work())?;

        Ok(WorkerHandle {
            handle,
            name: Self::NAME,
        })
    }

    fn work(self) -> WorkerResult
    where
        Self: Sized;
}

#[derive(Debug)]
pub struct WorkerHandle {
    handle: thread::JoinHandle<WorkerResult>,
    name: &'static str,
}

impl WorkerHandle {
    pub fn join(self) -> WorkerResult {
        let result: WorkerResult = self.handle.join().or(Err(WorkerError::ThreadPanicked {
            name: self.name.to_string(),
        }))?;
        if result.is_err() {
            println!("Error in {}, terminating.", self.name);
        } else {
            println!("{} shutting down.", self.name);
        }

        result
    }
}

#[derive(new, Debug, Fail)]
pub enum WorkerError {
    #[fail(display = "Attempted to access {}, but it was poisoned", name)]
    ResourcePoisoned { name: String },

    #[fail(display = "{} panicked.", name)]
    ThreadPanicked { name: String },
}

pub type WorkerResult = Result<(), Error>;
