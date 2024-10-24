use anyhow::Result;
#[derive(Debug, Clone)]
pub struct ImplDesktop {
}

impl ImplDesktop {
    pub fn get_desktop_size() -> Result<(i32,i32)> {
        let width = unsafe { winapi::um::winuser::GetSystemMetrics(winapi::um::winuser::SM_CXVIRTUALSCREEN) };
        let height = unsafe { winapi::um::winuser::GetSystemMetrics(winapi::um::winuser::SM_CYVIRTUALSCREEN) };
        Ok((width, height))
    }
}