use arboard::Clipboard;

#[cfg(target_os = "linux")]
use arboard::SetExtLinux;
#[cfg(target_os = "linux")]
use std::thread;

pub fn copy_text(text:&str) {
	// https://github.com/1Password/arboard/blob/master/README.md

	//linux clipboard manager (X11 and Wayland) does not hold the clipboard contents, this stays with the initiating app, so hold a thread open with the Clipboard object.
	#[cfg(target_os = "linux")]
	{
		let text2 = text.to_string();
		thread::spawn(move || {threaded_copy_text(text2.as_str());});
	}

	//windows and macos clipboard  manager hold the clipboard contents, so once copied to the clipboard, it stays there.  No need to keep the apps Clipboard alive.
	#[cfg(target_os = "windows")]
	{
		if let Ok(mut ctx) = Clipboard::new() {
			_ = ctx.set_text(text);
		}
	}
}

#[cfg(target_os = "linux")]
fn threaded_copy_text(text:&str) {
	//this thread keeps the clipboard source active until the clipboard is used again.
	//It will auto-exit once ctx.set.wait ends.
	if let Ok(mut ctx) = Clipboard::new() {
		_ = ctx.set().wait().text(text);
	}
}

#[cfg(test)]
mod tests {
	use super::*;
	use uuid::Uuid;

	#[test]
    fn test_copy_text() {
		let expected = Uuid::new_v4().to_string();
		copy_text(&expected);
		let mut result = String::new();
		if let Ok(mut ctx) = Clipboard::new() {
			result = ctx.get_text().unwrap();
		}
        assert_eq!(expected, result);
    }
}
