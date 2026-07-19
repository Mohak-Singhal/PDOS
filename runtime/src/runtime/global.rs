use std::sync::{Mutex, OnceLock};

use super::handle::RuntimeHandle;

static HANDLE: OnceLock<Mutex<RuntimeHandle>> = OnceLock::new();

pub fn runtime_handle() -> &'static Mutex<RuntimeHandle> {

    HANDLE.get_or_init(|| {

        Mutex::new(RuntimeHandle {

            thread: None,

        })

    })

}