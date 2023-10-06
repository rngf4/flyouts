use std::mem;

use windows::{
    core::{Error as WinError, Result},
    Win32::System::ProcessStatus::{EnumProcesses, GetProcessImageFileNameW},
};

// some function that uses EnumProcesses
fn get_all_processes() -> Result<(Vec<u32>, u32)> {
    let mut buf = vec![0u32; 1024];
    let mut returned_bytes = 0;
    unsafe {
        EnumProcesses(
            buf.as_mut_ptr(),
            (mem::size_of::<u32>() * buf.len()) as u32,
            &mut returned_bytes,
        )?;

        //https://learn.microsoft.com/en-us/windows/win32/api/shellapi/nf-shellapi-extracticona

        let file = GetProcessImageFileNameW(hprocess, lpimagefilename);
        
    }
    Ok((buf, returned_bytes / (mem::size_of::<u32>() as u32)))
}

fn main() {
    dbg!(get_all_processes().unwrap());
}