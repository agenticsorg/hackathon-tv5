//! C FFI bindings for the Omega TV SDK

use crate::error::*;
use crate::SdkState;
use once_cell::sync::Lazy;
use omega_tv_brain::{ViewContext, ViewingEvent, Recommendation};
use parking_lot::Mutex;
use std::ffi::{CStr, CString};
use std::os::raw::{c_char, c_int};
use std::ptr;

/// Global SDK state
static SDK: Lazy<Mutex<Option<SdkState>>> = Lazy::new(|| Mutex::new(None));

/// Last error message (thread-local for thread safety)
thread_local! {
    static LAST_ERROR: std::cell::RefCell<Option<CString>> = std::cell::RefCell::new(None);
}

/// Helper to convert C string to Rust string
unsafe fn c_str_to_string(ptr: *const c_char) -> Result<String> {
    if ptr.is_null() {
        return Err(Error::InvalidArg("Null pointer".into()));
    }
    CStr::from_ptr(ptr)
        .to_str()
        .map(|s| s.to_string())
        .map_err(|e| Error::InvalidArg(format!("Invalid UTF-8: {}", e)))
}

/// Helper to set last error
fn set_last_error(error: Error) -> c_int {
    let code = error.to_c_code();
    let message = error.message();

    if let Ok(c_str) = CString::new(message) {
        LAST_ERROR.with(|e| {
            *e.borrow_mut() = Some(c_str);
        });
    }

    code
}

/// Initialize the Omega SDK
///
/// # Arguments
/// * `storage_path` - Path to local storage directory (e.g., "/data/omega")
/// * `constellation_url` - URL of the constellation server (e.g., "https://constellation.example.com")
///
/// # Returns
/// * `OMEGA_SUCCESS` (0) on success
/// * `OMEGA_ERR_INIT` (-1) on initialization failure
/// * `OMEGA_ERR_INVALID_ARG` (-5) on invalid arguments
/// * `OMEGA_ERR_ALREADY_INITIALIZED` (-9) if already initialized
///
/// # Safety
/// Both pointers must be valid null-terminated C strings.
#[no_mangle]
pub unsafe extern "C" fn omega_init(
    storage_path: *const c_char,
    constellation_url: *const c_char,
) -> c_int {
    // Check if already initialized
    {
        let sdk = SDK.lock();
        if sdk.is_some() {
            return set_last_error(Error::AlreadyInitialized);
        }
    }

    // Convert C strings
    let storage = match c_str_to_string(storage_path) {
        Ok(s) => s,
        Err(e) => return set_last_error(e),
    };

    let url = match c_str_to_string(constellation_url) {
        Ok(s) => s,
        Err(e) => return set_last_error(e),
    };

    // Initialize SDK
    match SdkState::new(storage, url) {
        Ok(state) => {
            *SDK.lock() = Some(state);
            OMEGA_SUCCESS
        }
        Err(e) => set_last_error(e),
    }
}

/// Get recommendations based on viewing context
///
/// # Arguments
/// * `context_json` - JSON string with viewing context (e.g., '{"genre":"action","time":"evening"}')
/// * `out_json` - Output buffer for JSON recommendations
/// * `out_len` - Size of output buffer
///
/// # Returns
/// * `OMEGA_SUCCESS` (0) on success
/// * `OMEGA_ERR_RECOMMEND` (-2) on recommendation failure
/// * `OMEGA_ERR_INVALID_ARG` (-5) on invalid arguments
/// * `OMEGA_ERR_JSON_PARSE` (-6) on JSON parsing error
/// * `OMEGA_ERR_BUFFER_TOO_SMALL` (-7) if output buffer is too small
/// * `OMEGA_ERR_NOT_INITIALIZED` (-8) if SDK not initialized
///
/// # Safety
/// * `context_json` must be a valid null-terminated C string
/// * `out_json` must point to a buffer of at least `out_len` bytes
#[no_mangle]
pub unsafe extern "C" fn omega_recommend(
    context_json: *const c_char,
    out_json: *mut c_char,
    out_len: c_int,
) -> c_int {
    if out_json.is_null() || out_len <= 0 {
        return set_last_error(Error::InvalidArg("Invalid output buffer".into()));
    }

    // Get context JSON
    let context_str = match c_str_to_string(context_json) {
        Ok(s) => s,
        Err(e) => return set_last_error(e),
    };

    // Parse context
    let context: ViewContext = match serde_json::from_str(&context_str) {
        Ok(c) => c,
        Err(e) => return set_last_error(Error::JsonParse(e.to_string())),
    };

    // Get recommendations
    let recommendations = {
        let sdk = SDK.lock();
        match sdk.as_ref() {
            Some(state) => match state.recommend(&context) {
                Ok(recs) => recs,
                Err(e) => return set_last_error(e),
            },
            None => return set_last_error(Error::NotInitialized),
        }
    };

    // Serialize to JSON
    let json_output = match serde_json::to_string(&recommendations) {
        Ok(j) => j,
        Err(e) => return set_last_error(Error::Internal(e.to_string())),
    };

    // Check buffer size
    if json_output.len() + 1 > out_len as usize {
        return set_last_error(Error::BufferTooSmall(json_output.len() + 1));
    }

    // Copy to output buffer
    let bytes = json_output.as_bytes();
    ptr::copy_nonoverlapping(bytes.as_ptr(), out_json as *mut u8, bytes.len());
    *out_json.add(bytes.len()) = 0; // Null terminate

    OMEGA_SUCCESS
}

/// Record a viewing event
///
/// # Arguments
/// * `event_json` - JSON string with viewing event data
///
/// # Returns
/// * `OMEGA_SUCCESS` (0) on success
/// * `OMEGA_ERR_OBSERVE` (-3) on observation failure
/// * `OMEGA_ERR_INVALID_ARG` (-5) on invalid arguments
/// * `OMEGA_ERR_JSON_PARSE` (-6) on JSON parsing error
/// * `OMEGA_ERR_NOT_INITIALIZED` (-8) if SDK not initialized
///
/// # Safety
/// `event_json` must be a valid null-terminated C string
#[no_mangle]
pub unsafe extern "C" fn omega_observe(event_json: *const c_char) -> c_int {
    // Get event JSON
    let event_str = match c_str_to_string(event_json) {
        Ok(s) => s,
        Err(e) => return set_last_error(e),
    };

    // Parse event
    let event: ViewingEvent = match serde_json::from_str(&event_str) {
        Ok(e) => e,
        Err(e) => return set_last_error(Error::JsonParse(e.to_string())),
    };

    // Record event
    let mut sdk = SDK.lock();
    match sdk.as_mut() {
        Some(state) => match state.observe(event) {
            Ok(_) => OMEGA_SUCCESS,
            Err(e) => set_last_error(e),
        },
        None => set_last_error(Error::NotInitialized),
    }
}

/// Sync with constellation server
///
/// # Returns
/// * `OMEGA_SUCCESS` (0) on success
/// * `OMEGA_ERR_SYNC` (-4) on sync failure
/// * `OMEGA_ERR_NOT_INITIALIZED` (-8) if SDK not initialized
///
/// # Safety
/// This function is safe to call from any thread.
#[no_mangle]
pub unsafe extern "C" fn omega_sync() -> c_int {
    let mut sdk = SDK.lock();
    match sdk.as_mut() {
        Some(state) => match state.sync() {
            Ok(_) => OMEGA_SUCCESS,
            Err(e) => set_last_error(e),
        },
        None => set_last_error(Error::NotInitialized),
    }
}

/// Shutdown the SDK and free resources
///
/// # Returns
/// * `OMEGA_SUCCESS` (0) on success
/// * `OMEGA_ERR_NOT_INITIALIZED` (-8) if SDK not initialized
///
/// # Safety
/// After calling this function, the SDK must be reinitialized before use.
/// This function should be called before program exit.
#[no_mangle]
pub unsafe extern "C" fn omega_shutdown() -> c_int {
    let mut sdk = SDK.lock();
    if sdk.is_none() {
        return set_last_error(Error::NotInitialized);
    }

    *sdk = None;
    OMEGA_SUCCESS
}

/// Get the last error message
///
/// # Returns
/// Pointer to null-terminated error string, or NULL if no error.
/// The returned pointer is valid until the next error occurs or the thread exits.
///
/// # Safety
/// The returned pointer must not be freed by the caller.
/// The string is valid until the next call to any omega_* function from the same thread.
#[no_mangle]
pub unsafe extern "C" fn omega_get_last_error() -> *const c_char {
    LAST_ERROR.with(|e| {
        e.borrow()
            .as_ref()
            .map(|s| s.as_ptr())
            .unwrap_or(ptr::null())
    })
}

/// Get SDK version string
///
/// # Returns
/// Pointer to null-terminated version string (e.g., "0.1.0")
///
/// # Safety
/// The returned pointer points to a static string and is always valid.
#[no_mangle]
pub unsafe extern "C" fn omega_version() -> *const c_char {
    static VERSION: &[u8] = b"0.1.0\0";
    VERSION.as_ptr() as *const c_char
}

/// Check if SDK is initialized
///
/// # Returns
/// * 1 if initialized
/// * 0 if not initialized
///
/// # Safety
/// This function is safe to call from any thread.
#[no_mangle]
pub unsafe extern "C" fn omega_is_initialized() -> c_int {
    let sdk = SDK.lock();
    if sdk.is_some() { 1 } else { 0 }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_ffi_version() {
        unsafe {
            let version = omega_version();
            assert!(!version.is_null());
            let version_str = CStr::from_ptr(version).to_str().unwrap();
            assert_eq!(version_str, "0.1.0");
        }
    }

    #[test]
    fn test_ffi_initialization_state() {
        unsafe {
            assert_eq!(omega_is_initialized(), 0);
        }
    }
}
