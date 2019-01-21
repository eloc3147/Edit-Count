use std::thread;

pub trait Worker
where
    Self: Sized + Send,
{
    type W: Worker;
    const NAME: &'static str;

    fn start(self) -> WorkerWrapper
    where
        Self: 'static,
    {
        let handle = thread::spawn(move || {
            self.work();
        });

        WorkerWrapper {
            name: Self::NAME,
            handle,
        }
    }

    fn work(self)
    where
        Self: Sized;
}

pub trait WorkerConfig
where
    Self: Send + Sized,
{
}

pub struct WorkerWrapper {
    name: &'static str,
    handle: thread::JoinHandle<()>,
}

impl WorkerWrapper {
    pub fn wait(self) {
        if let Err(e) = self.handle.join() {
            println!("{} panicked: {:#?}", self.name, e);
        }
    }
}
