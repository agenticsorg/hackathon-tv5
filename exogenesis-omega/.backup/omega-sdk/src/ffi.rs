//! C Foreign Function Interface (FFI) for Omega SDK
//!
//! Provides C-compatible bindings for TV platforms that use C/C++

use std::ffi::{CStr, CString};
use std::os::raw::{c_char, c_int};
use std::sync::Mutex;
use once_cell::sync::Lazy;

use crate::builder::{OmegaBrain, OmegaBuilder};
use crate::error::{OmegaError, OmegaErrorCode};

/// Global Omega Brain instance
///
/// This is a singleton that holds the brain state across FFI calls.
/// Thread-safe with Mutex protection.
static BRAIN: Lazy<Mutex<Option<OmegaBrain>>> = Lazy::new(|| Mutex::new(None));

/// Thread-local error message storage for detailed error reporting
thread_local! {
    static LAST_ERROR: std::cell::RefCell<Option<String>> = std::cell::RefCell::new(None);
}

/// Store error message for retrieval via omega_get_last_error
fn set_last_error(error: &OmegaError) {
    LAST_ERROR.with(|e| {
        *e.borrow_mut() = Some(error.to_string());
    });
}

/// Initialize Omega Brain on TV
///
/// # Safety
/// - `model_path` must be a valid null-terminated C string
/// - `storage_path` must be a valid null-terminated C string
/// - Pointers must remain valid for the duration of the call
///
/// # Arguments
/// - `model_path`: Path to ONNX model file (e.g., "/data/omega/model.onnx")
/// - `storage_path`: Path for persistent storage (e.g., "/data/omega/brain.db")
///
/// # Returns
/// - 0 on success
/// - Negative error code on failure (see OmegaErrorCode)
///
/// # Example (C)
/// ```c
/// int result = omega_init("/data/omega/model.onnx", "/data/omega/brain.db");
/// if (result != 0) {
///     const char* error = omega_get_last_error();
///     fprintf(stderr, "Init failed: %s\n", error);
/// }
/// ```
#[no_mangle]
pub extern "C" fn omega_init(
    model_path: *const c_char,
    storage_path: *const c_char,
) -> c_int {
    // Validate pointers
    if model_path.is_null() || storage_path.is_null() {
        let error = OmegaError::InvalidInput("Null pointer passed to omega_init".to_string());
        set_last_error(&error);
        return OmegaErrorCode::InvalidInput as c_int;
    }

    // Convert C strings to Rust
    let model_path_str = match unsafe { CStr::from_ptr(model_path) }.to_str() {
        Ok(s) => s,
        Err(_) => {
            let error = OmegaError::InvalidInput("Invalid UTF-8 in model_path".to_string());
            set_last_error(&error);
            return OmegaErrorCode::InvalidInput as c_int;
        }
    };

    let storage_path_str = match unsafe { CStr::from_ptr(storage_path) }.to_str() {
        Ok(s) => s,
        Err(_) => {
            let error = OmegaError::InvalidInput("Invalid UTF-8 in storage_path".to_string());
            set_last_error(&error);
            return OmegaErrorCode::InvalidInput as c_int;
        }
    };

    // Build and initialize brain
    let result = tokio::runtime::Runtime::new()
        .expect("Failed to create tokio runtime")
        .block_on(async {
            OmegaBuilder::new()
                .model_path(model_path_str)
                .storage_path(storage_path_str)
                .build()
                .await
        });

    match result {
        Ok(brain) => {
            // Store in global singleton
            *BRAIN.lock().unwrap() = Some(brain);
            OmegaErrorCode::Success as c_int
        }
        Err(error) => {
            set_last_error(&error);
            OmegaErrorCode::from(&error) as c_int
        }
    }
}

/// Get personalized recommendations
///
/// # Safety
/// - `context_json` must be a valid null-terminated C string
/// - `out_json` must point to a valid buffer of at least `out_len` bytes
/// - Caller must ensure thread safety
///
/// # Arguments
/// - `context_json`: JSON context string (e.g., `{"time":"evening","mood":"relaxed"}`)
/// - `out_json`: Output buffer for JSON recommendations array
/// - `out_len`: Size of output buffer
///
/// # Returns
/// - 0 on success (recommendations written to out_json)
/// - Negative error code on failure
///
/// # Example (C)
/// ```c
/// char recommendations[4096];
/// int result = omega_recommend(
///     "{\"time\":\"evening\",\"mood\":\"relaxed\"}",
///     recommendations,
///     sizeof(recommendations)
/// );
/// if (result == 0) {
///     printf("Recommendations: %s\n", recommendations);
/// }
/// ```
#[no_mangle]
pub extern "C" fn omega_recommend(
    context_json: *const c_char,
    out_json: *mut c_char,
    out_len: c_int,
) -> c_int {
    // Validate pointers
    if context_json.is_null() || out_json.is_null() || out_len <= 0 {
        let error = OmegaError::InvalidInput("Invalid parameters to omega_recommend".to_string());
        set_last_error(&error);
        return OmegaErrorCode::InvalidInput as c_int;
    }

    // Convert context to Rust string
    let context_str = match unsafe { CStr::from_ptr(context_json) }.to_str() {
        Ok(s) => s,
        Err(_) => {
            let error = OmegaError::InvalidInput("Invalid UTF-8 in context_json".to_string());
            set_last_error(&error);
            return OmegaErrorCode::InvalidInput as c_int;
        }
    };

    // Get recommendations
    let mut brain = BRAIN.lock().unwrap();
    let brain = match brain.as_mut() {
        Some(b) => b,
        None => {
            let error = OmegaError::InitError("Brain not initialized. Call omega_init first".to_string());
            set_last_error(&error);
            return OmegaErrorCode::InitError as c_int;
        }
    };

    let recommendations = tokio::runtime::Runtime::new()
        .expect("Failed to create tokio runtime")
        .block_on(async {
            brain.recommend(context_str).await
        });

    let recommendations = match recommendations {
        Ok(r) => r,
        Err(error) => {
            set_last_error(&error);
            return OmegaErrorCode::from(&error) as c_int;
        }
    };

    // Serialize to JSON
    let json_str = match serde_json::to_string(&recommendations) {
        Ok(s) => s,
        Err(e) => {
            let error = OmegaError::Internal(format!("JSON serialization failed: {}", e));
            set_last_error(&error);
            return OmegaErrorCode::Internal as c_int;
        }
    };

    // Copy to output buffer
    let c_json = match CString::new(json_str) {
        Ok(s) => s,
        Err(_) => {
            let error = OmegaError::Internal("Failed to create C string".to_string());
            set_last_error(&error);
            return OmegaErrorCode::Internal as c_int;
        }
    };

    let bytes = c_json.as_bytes_with_nul();
    if bytes.len() > out_len as usize {
        let error = OmegaError::InvalidInput(
            format!("Output buffer too small: need {} bytes, have {}", bytes.len(), out_len)
        );
        set_last_error(&error);
        return OmegaErrorCode::InvalidInput as c_int;
    }

    unsafe {
        std::ptr::copy_nonoverlapping(
            c_json.as_ptr(),
            out_json,
            bytes.len(),
        );
    }

    OmegaErrorCode::Success as c_int
}

/// Record viewing event
///
/// # Safety
/// - `event_json` must be a valid null-terminated C string
///
/// # Arguments
/// - `event_json`: JSON event (e.g., `{"content_id":"abc123","watch_pct":0.95}`)
///
/// # Returns
/// - 0 on success
/// - Negative error code on failure
///
/// # Example (C)
/// ```c
/// int result = omega_observe("{\"content_id\":\"movie123\",\"watch_pct\":0.95}");
/// if (result != 0) {
///     fprintf(stderr, "Failed to record event\n");
/// }
/// ```
#[no_mangle]
pub extern "C" fn omega_observe(event_json: *const c_char) -> c_int {
    if event_json.is_null() {
        let error = OmegaError::InvalidInput("Null event_json".to_string());
        set_last_error(&error);
        return OmegaErrorCode::InvalidInput as c_int;
    }

    let event_str = match unsafe { CStr::from_ptr(event_json) }.to_str() {
        Ok(s) => s,
        Err(_) => {
            let error = OmegaError::InvalidInput("Invalid UTF-8 in event_json".to_string());
            set_last_error(&error);
            return OmegaErrorCode::InvalidInput as c_int;
        }
    };

    let mut brain = BRAIN.lock().unwrap();
    let brain = match brain.as_mut() {
        Some(b) => b,
        None => {
            let error = OmegaError::InitError("Brain not initialized".to_string());
            set_last_error(&error);
            return OmegaErrorCode::InitError as c_int;
        }
    };

    let result = tokio::runtime::Runtime::new()
        .expect("Failed to create tokio runtime")
        .block_on(async {
            brain.observe(event_str).await
        });

    match result {
        Ok(_) => OmegaErrorCode::Success as c_int,
        Err(error) => {
            set_last_error(&error);
            OmegaErrorCode::from(&error) as c_int
        }
    }
}

/// Sync patterns with Omega Constellation
///
/// Call every 5-15 minutes when network available.
/// This pushes local patterns (~1KB) and receives global patterns (~5KB).
///
/// # Returns
/// - 0 on success
/// - Negative error code on failure
///
/// # Example (C)
/// ```c
/// // Call periodically (e.g., every 5 minutes)
/// int result = omega_sync();
/// if (result != 0) {
///     fprintf(stderr, "Sync failed, will retry later\n");
/// }
/// ```
#[no_mangle]
pub extern "C" fn omega_sync() -> c_int {
    let mut brain = BRAIN.lock().unwrap();
    let brain = match brain.as_mut() {
        Some(b) => b,
        None => {
            let error = OmegaError::InitError("Brain not initialized".to_string());
            set_last_error(&error);
            return OmegaErrorCode::InitError as c_int;
        }
    };

    let result = tokio::runtime::Runtime::new()
        .expect("Failed to create tokio runtime")
        .block_on(async {
            brain.sync().await
        });

    match result {
        Ok(_) => OmegaErrorCode::Success as c_int,
        Err(error) => {
            set_last_error(&error);
            OmegaErrorCode::from(&error) as c_int
        }
    }
}

/// Shutdown and persist state
///
/// Call before TV shutdown to ensure all data is saved.
///
/// # Returns
/// - 0 on success
/// - Negative error code on failure
///
/// # Example (C)
/// ```c
/// // On TV shutdown
/// omega_shutdown();
/// ```
#[no_mangle]
pub extern "C" fn omega_shutdown() -> c_int {
    let mut brain = BRAIN.lock().unwrap();
    let brain = match brain.take() {
        Some(mut b) => b,
        None => {
            // Already shutdown or never initialized
            return OmegaErrorCode::Success as c_int;
        }
    };

    let result = tokio::runtime::Runtime::new()
        .expect("Failed to create tokio runtime")
        .block_on(async {
            let mut b = brain;
            b.shutdown().await
        });

    match result {
        Ok(_) => OmegaErrorCode::Success as c_int,
        Err(error) => {
            set_last_error(&error);
            OmegaErrorCode::from(&error) as c_int
        }
    }
}

/// Get last error message
///
/// Returns a pointer to a null-terminated error string.
/// The pointer is valid until the next call to any omega_* function on the same thread.
///
/// # Returns
/// - Pointer to error message, or NULL if no error
///
/// # Example (C)
/// ```c
/// int result = omega_init("/bad/path", "/bad/path");
/// if (result != 0) {
///     const char* error = omega_get_last_error();
///     if (error) {
///         fprintf(stderr, "Error: %s\n", error);
///     }
/// }
/// ```
#[no_mangle]
pub extern "C" fn omega_get_last_error() -> *const c_char {
    LAST_ERROR.with(|e| {
        match e.borrow().as_ref() {
            Some(err) => {
                match CString::new(err.as_str()) {
                    Ok(s) => s.into_raw() as *const c_char,
                    Err(_) => std::ptr::null(),
                }
            }
            None => std::ptr::null(),
        }
    })
}

/// Free error string returned by omega_get_last_error
///
/// # Safety
/// - Must only be called with pointers returned by omega_get_last_error
/// - Must not be called more than once with the same pointer
///
/// # Example (C)
/// ```c
/// const char* error = omega_get_last_error();
/// if (error) {
///     fprintf(stderr, "Error: %s\n", error);
///     omega_free_error(error);
/// }
/// ```
#[no_mangle]
pub extern "C" fn omega_free_error(ptr: *const c_char) {
    if !ptr.is_null() {
        unsafe {
            // Reconstruct CString to free it
            let _ = CString::from_raw(ptr as *mut c_char);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_error_code_values() {
        assert_eq!(OmegaErrorCode::Success as c_int, 0);
        assert_eq!(OmegaErrorCode::InitError as c_int, -1);
        assert_eq!(OmegaErrorCode::InferenceError as c_int, -2);
        assert_eq!(OmegaErrorCode::SyncError as c_int, -3);
    }

    #[test]
    fn test_set_get_last_error() {
        let error = OmegaError::InitError("test error".to_string());
        set_last_error(&error);

        LAST_ERROR.with(|e| {
            assert!(e.borrow().is_some());
            assert!(e.borrow().as_ref().unwrap().contains("test error"));
        });
    }
}
