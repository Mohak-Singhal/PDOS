use std::thread::JoinHandle;

pub struct RuntimeHandle {
    pub thread: Option<JoinHandle<()>>,
}