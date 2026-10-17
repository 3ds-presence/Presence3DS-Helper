pub fn reboot() {
    unsafe {
        let _ = ctru_sys::APT_HardwareResetAsync();
    }
}
