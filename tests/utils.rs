use chrono::prelude::*;

/// Get current datetime in cvapi persistence format
pub fn now() -> DateTime<FixedOffset> {
    // checkvist api dates only resolve to s
    Local::now().trunc_subsecs(0).try_into().unwrap()
}
