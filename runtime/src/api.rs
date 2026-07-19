use crate::manager::runtime;

pub fn start_runtime() {

    let mut runtime = runtime().lock().unwrap();

    runtime.start();
}

pub fn stop_runtime() {

    let mut runtime = runtime().lock().unwrap();

    runtime.stop();
}

pub fn runtime_status() -> i32 {

    let runtime = runtime().lock().unwrap();

    runtime.state() as i32
}