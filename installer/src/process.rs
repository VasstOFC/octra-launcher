use std::time::{Duration, Instant};

#[cfg(windows)]
pub fn octra_pids() -> Vec<u32> {
    use windows::Win32::Foundation::CloseHandle;
    use windows::Win32::System::Diagnostics::ToolHelp::{
        CreateToolhelp32Snapshot, Process32FirstW, Process32NextW, PROCESSENTRY32W,
        TH32CS_SNAPPROCESS,
    };

    let mut pids = Vec::new();
    let self_pid = std::process::id();
    unsafe {
        let snap = match CreateToolhelp32Snapshot(TH32CS_SNAPPROCESS, 0) {
            Ok(h) => h,
            Err(_) => return pids,
        };
        let mut entry = PROCESSENTRY32W {
            dwSize: std::mem::size_of::<PROCESSENTRY32W>() as u32,
            ..Default::default()
        };
        if Process32FirstW(snap, &mut entry).is_ok() {
            loop {
                let name = {
                    let len = entry
                        .szExeFile
                        .iter()
                        .position(|&c| c == 0)
                        .unwrap_or(entry.szExeFile.len());
                    String::from_utf16_lossy(&entry.szExeFile[..len])
                };
                if name.eq_ignore_ascii_case("octra.exe") && entry.th32ProcessID != self_pid {
                    pids.push(entry.th32ProcessID);
                }
                if Process32NextW(snap, &mut entry).is_err() {
                    break;
                }
            }
        }
        let _ = CloseHandle(snap);
    }
    pids
}

#[cfg(not(windows))]
pub fn octra_pids() -> Vec<u32> {
    Vec::new()
}

pub fn is_running() -> bool {
    !octra_pids().is_empty()
}

pub fn wait_until_gone(timeout: Duration) -> bool {
    let start = Instant::now();
    while start.elapsed() < timeout {
        if !is_running() {
            return true;
        }
        std::thread::sleep(Duration::from_millis(200));
    }
    !is_running()
}

#[cfg(windows)]
pub fn terminate_all() {
    use windows::Win32::Foundation::CloseHandle;
    use windows::Win32::System::Threading::{OpenProcess, TerminateProcess, PROCESS_TERMINATE};

    for pid in octra_pids() {
        unsafe {
            if let Ok(h) = OpenProcess(PROCESS_TERMINATE, false, pid) {
                let _ = TerminateProcess(h, 0);
                let _ = CloseHandle(h);
            }
        }
    }
}

#[cfg(not(windows))]
pub fn terminate_all() {}

pub fn wait_or_stop(_update_or_silent: bool) -> Result<(), String> {
    if !is_running() {
        return Ok(());
    }
    if !wait_until_gone(Duration::from_secs(8)) {
        terminate_all();
        if !wait_until_gone(Duration::from_secs(5)) {
            return Err(
                "Octra jest nadal uruchomiona (sprawdź Menedżer zadań → octra.exe). Zamknij ją i spróbuj ponownie.".into(),
            );
        }
    }
    Ok(())
}
