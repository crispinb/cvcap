use log::error;

use copypasta::{ClipboardContext, ClipboardProvider};

pub fn get_clipboard_as_string() -> Option<String> {
    let mut ctx = ClipboardContext::new()
        .map_err(|e| error!("Error getting clipboard contents: {}", e))
        .ok()?;
    let cliptext = ctx
        .get_contents()
        .map_err(|e| error!("Error getting clipboard contents: {}", e))
        .ok()?;

    Some(cliptext)
}
