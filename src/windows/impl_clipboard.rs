use anyhow::Error;

use clipboard::{ClipboardContext, ClipboardProvider};
#[derive(Debug, Clone)]
pub struct ImplClipboard {
}

impl ImplClipboard {
    pub fn set_text(text: &str) -> Result<(), Error> {
        // 复制 color 到剪贴板
        let mut ctx: ClipboardContext = ClipboardProvider::new().unwrap();
        ctx.set_contents(text.to_string()).unwrap();  // 转换为 String 并复制
        Ok(())
    }
}