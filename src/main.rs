#![cfg_attr(
    all(not(debug_assertions), target_os = "windows"),
    windows_subsystem = "windows"
)]

use clap::Parser;
use std::path::PathBuf;
use winapi::shared::minwindef::{BOOL, DWORD, LPVOID};
use winapi::shared::ntdef::{LPCWSTR, LPWSTR};
use winapi::um::libloaderapi::{GetProcAddress, LoadLibraryW};
use winapi::um::minwinbase::LPSECURITY_ATTRIBUTES;
use winapi::um::processthreadsapi::{
    LPPROCESS_INFORMATION, LPSTARTUPINFOW, PROCESS_INFORMATION, STARTUPINFOW,
};
use winapi::um::winbase::DETACHED_PROCESS;

type VGRL1337 = extern "stdcall" fn(
    lpApplicationName: LPCWSTR,
    lpCommandLine: LPWSTR,
    lpProcessAttributes: LPSECURITY_ATTRIBUTES,
    lpThreadAttributes: LPSECURITY_ATTRIBUTES,
    bInheritHandles: BOOL,
    dwCreationFlags: DWORD,
    lpEnvironment: LPVOID,
    lpCurrentDirectory: LPCWSTR,
    lpStartupInfo: LPSTARTUPINFOW,
    lpProcessInformation: LPPROCESS_INFORMATION,
) -> BOOL;

#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
struct Args {
    #[arg(short, long)]
    vgrl: PathBuf,

    #[arg(short, long)]
    executable: PathBuf,

    #[arg(short, long)]
    parameter: Option<String>,

    #[arg(short, long)]
    directory: Option<PathBuf>,
}

fn to_wide_chars(s: &str) -> Vec<u16> {
    use std::ffi::OsStr;
    use std::os::windows::ffi::OsStrExt;
    OsStr::new(s)
        .encode_wide()
        .chain(Some(0).into_iter())
        .collect::<Vec<_>>()
}

fn main() -> std::io::Result<()> {
    let args: Args = Args::parse();

    let vgrl_path: Vec<u16> = to_wide_chars(&args.vgrl.to_string_lossy());
    let vgrl = unsafe { LoadLibraryW(vgrl_path.as_ptr()) };
    if vgrl.is_null() {
        return Err(std::io::Error::last_os_error());
    }

    println!("vgrl.dll loaded: {:p}", vgrl);

    let fn_ptr = unsafe { GetProcAddress(vgrl, 1337 as _) };
    if fn_ptr.is_null() {
        return Err(std::io::Error::last_os_error());
    }

    println!("vgrl.dll ordinal 1337 loaded: {:p}", fn_ptr);

    let vgrl1337 = unsafe { std::mem::transmute::<_, VGRL1337>(fn_ptr) };
    let executable_path = to_wide_chars(&args.executable.to_string_lossy());

    let mut startup_info: STARTUPINFOW = unsafe { std::mem::zeroed() };
    let mut process_info: PROCESS_INFORMATION = unsafe { std::mem::zeroed() };

    let result: BOOL = vgrl1337(
        executable_path.as_ptr(),
        match args.parameter {
            Some(v) => to_wide_chars(&v).as_ptr() as _,
            None => std::ptr::null_mut(),
        },
        std::ptr::null_mut(),
        std::ptr::null_mut(),
        0,
        DETACHED_PROCESS,
        std::ptr::null_mut(),
        match args.directory {
            Some(v) => to_wide_chars(&v.to_string_lossy()).as_ptr() as _,
            None => std::ptr::null_mut(),
        },
        &mut startup_info,
        &mut process_info,
    );

    println!("vgrl1337 returned: {}", result != 0);

    if result == 0 {
        return Err(std::io::Error::last_os_error());
    }

    return Ok(());
}
