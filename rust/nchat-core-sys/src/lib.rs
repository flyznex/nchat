use std::ffi::{c_char, c_void, CStr, CString};
use std::fmt;
use std::sync::mpsc::Sender;

pub mod raw {
    use std::ffi::{c_char, c_int, c_void};

    pub type EventCallback = extern "C" fn(json: *const c_char, user: *mut c_void);

    extern "C" {
        pub fn nc_abi_version() -> c_int;
        pub fn nc_init(config_dir: *const c_char) -> c_int;
        pub fn nc_set_callback(cb: Option<EventCallback>, user: *mut c_void);
        pub fn nc_start() -> c_int;
        pub fn nc_send(json: *const c_char) -> c_int;
        pub fn nc_profiles() -> *mut c_char;
        pub fn nc_free_string(s: *mut c_char);
        pub fn nc_stop();
    }
}

#[derive(Debug, PartialEq, Eq)]
pub enum CoreError {
    /// Non-zero NC_ERR_* code from the bridge (1 state, 2 request, 3 no profile, 4 confdir, 5 locked).
    Code(i32),
    /// The string passed in contained an interior NUL byte.
    Nul,
}

impl fmt::Display for CoreError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            CoreError::Code(c) => write!(f, "core error code {c}"),
            CoreError::Nul => write!(f, "string contains NUL"),
        }
    }
}

impl std::error::Error for CoreError {}

fn check(code: i32) -> Result<(), CoreError> {
    if code == 0 {
        Ok(())
    } else {
        Err(CoreError::Code(code))
    }
}

// Called from C++ protocol threads; a panic must not unwind across the extern "C" boundary.
extern "C" fn on_event(json: *const c_char, user: *mut c_void) {
    if json.is_null() || user.is_null() {
        return;
    }
    let _ = std::panic::catch_unwind(|| {
        let tx = unsafe { &*(user as *const Sender<String>) };
        let text = unsafe { CStr::from_ptr(json) }.to_string_lossy().into_owned();
        let _ = tx.send(text);
    });
}

/// The bridge state is not internally synchronized: use one Core from one control thread.
pub struct Core {
    sender: *mut Sender<String>,
    stopped: bool,
}

unsafe impl Send for Core {}

impl Core {
    pub fn init(config_dir: &str, events: Sender<String>) -> Result<Core, CoreError> {
        let dir = CString::new(config_dir).map_err(|_| CoreError::Nul)?;
        check(unsafe { raw::nc_init(dir.as_ptr()) })?;
        let sender = Box::into_raw(Box::new(events));
        unsafe { raw::nc_set_callback(Some(on_event), sender as *mut c_void) };
        Ok(Core { sender, stopped: false })
    }

    pub fn start(&self) -> Result<String, CoreError> {
        check(unsafe { raw::nc_start() })?;
        let p = unsafe { raw::nc_profiles() };
        if p.is_null() {
            return Ok("[]".to_string());
        }
        let json = unsafe { CStr::from_ptr(p) }.to_string_lossy().into_owned();
        unsafe { raw::nc_free_string(p) };
        Ok(json)
    }

    pub fn send(&self, json: &str) -> Result<(), CoreError> {
        let c = CString::new(json).map_err(|_| CoreError::Nul)?;
        check(unsafe { raw::nc_send(c.as_ptr()) })
    }

    pub fn stop(&mut self) {
        if self.stopped {
            return;
        }
        // nc_stop clears the callback, so no event can reach the sender after it returns.
        unsafe { raw::nc_stop() };
        drop(unsafe { Box::from_raw(self.sender) });
        self.stopped = true;
    }
}

impl Drop for Core {
    fn drop(&mut self) {
        self.stop();
    }
}

#[cfg(test)]
mod tests {
    #[test]
    fn abi_version_matches_header() {
        assert_eq!(unsafe { super::raw::nc_abi_version() }, 1);
    }

    #[test]
    fn init_rejects_interior_nul() {
        let (tx, _rx) = std::sync::mpsc::channel();
        assert!(matches!(
            super::Core::init("a\0b", tx),
            Err(super::CoreError::Nul)
        ));
    }
}
