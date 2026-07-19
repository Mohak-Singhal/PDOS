use crate::models::{DeviceType, OperatingSystem};

#[derive(Debug, Clone)]
pub struct Platform {
    pub operating_system: OperatingSystem,
    pub device_type: DeviceType,
}

pub fn detect() -> Platform {
    Platform {
        operating_system: detect_operating_system(),
        device_type: detect_device_type(),
    }
}

fn detect_operating_system() -> OperatingSystem {
    #[cfg(target_os = "macos")]
    {
        OperatingSystem::MacOS
    }

    #[cfg(target_os = "android")]
    {
        OperatingSystem::Android
    }

    #[cfg(not(any(target_os = "macos", target_os = "android")))]
    {
        panic!("PDOS Runtime currently supports only macOS and Android.");
    }
}

fn detect_device_type() -> DeviceType {
    #[cfg(target_os = "macos")]
    {
        DeviceType::Laptop
    }

    #[cfg(target_os = "android")]
    {
        DeviceType::Phone
    }

    #[cfg(not(any(target_os = "macos", target_os = "android")))]
    {
        panic!("Unsupported device type.");
    }
}
