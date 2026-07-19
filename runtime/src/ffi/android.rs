use jni::objects::{JClass, JString};
use jni::sys::{jint, jstring};
use jni::JNIEnv;
use std::sync::atomic::{AtomicBool, Ordering};

static RUNTIME_STARTED: AtomicBool = AtomicBool::new(false);

#[unsafe(no_mangle)]
pub extern "system" fn Java_dev_pdos_PDOSNative_setDeviceName(
    mut env: JNIEnv,
    _class: JClass,
    name: JString,
) {
    let name: String = env
        .get_string(&name)
        .expect("Failed to get device name string")
        .into();
    let _ = crate::DEVICE_NAME.set(name);
}

#[unsafe(no_mangle)]
pub extern "system" fn Java_dev_pdos_PDOSNative_startRuntime(
    mut env: JNIEnv,
    _class: JClass,
    files_dir: JString,
) {
    crate::init_logging();

    // Prevent starting multiple runtimes.
    if RUNTIME_STARTED.swap(true, Ordering::SeqCst) {
        log::info!("PDOS Runtime already running");
        return;
    }

    let path: String = env
        .get_string(&files_dir)
        .expect("Failed to get files_dir string")
        .into();

    crate::APP_DATA_DIR.set(path).expect("Data dir already set");

    log::info!("Starting PDOS Runtime...");

    std::thread::spawn(|| {
        let runtime = tokio::runtime::Builder::new_multi_thread()
            .enable_all()
            .build()
            .expect("Failed to create Tokio runtime");

        runtime.block_on(async {
            crate::start_runtime().await;
        });
    });
}

#[unsafe(no_mangle)]
pub extern "system" fn Java_dev_pdos_PDOSNative_stopRuntime(
    _env: JNIEnv,
    _class: JClass,
) {
    log::info!("PDOS Runtime Stop");

    // Temporary only.
    // Later we'll gracefully stop the Tokio runtime using a shutdown signal.
    RUNTIME_STARTED.store(false, Ordering::SeqCst);
}

#[unsafe(no_mangle)]
pub extern "system" fn Java_dev_pdos_PDOSNative_runtimeStatus(
    _env: JNIEnv,
    _class: JClass,
) -> jint {
    if RUNTIME_STARTED.load(Ordering::SeqCst) {
        1
    } else {
        0
    }
}

#[unsafe(no_mangle)]
pub extern "system" fn Java_dev_pdos_PDOSNative_protocolVersion(
    _env: JNIEnv,
    _class: JClass,
) -> jint {
    crate::constants::PROTOCOL_VERSION as jint
}

#[unsafe(no_mangle)]
pub extern "system" fn Java_dev_pdos_PDOSNative_connectedNodes(
    _env: JNIEnv,
    _class: JClass,
) -> jint {
    if let Ok(list) = crate::NODE_LIST.lock() {
        list.len() as jint
    } else {
        0
    }
}

#[unsafe(no_mangle)]
pub extern "system" fn Java_dev_pdos_PDOSNative_runtimeVersion(
    mut env: JNIEnv,
    _class: JClass,
) -> jstring {
    env.new_string(env!("CARGO_PKG_VERSION"))
        .expect("Failed to create Java string")
        .into_raw()
}