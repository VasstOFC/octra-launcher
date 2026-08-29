const GUID: &str = "{F3017226-FE2A-4295-8BDF-00C3A9A7E4C5}";
const BOOTSTRAPPER: &str = "https://go.microsoft.com/fwlink/p/?LinkId=2124703";

pub fn is_installed() -> bool {
    use winreg::enums::{HKEY_CURRENT_USER, HKEY_LOCAL_MACHINE, KEY_READ};
    use winreg::RegKey;

    let paths = [
        (
            HKEY_LOCAL_MACHINE,
            r"SOFTWARE\WOW6432Node\Microsoft\EdgeUpdate\Clients\",
        ),
        (
            HKEY_LOCAL_MACHINE,
            r"SOFTWARE\Microsoft\EdgeUpdate\Clients\",
        ),
        (
            HKEY_CURRENT_USER,
            r"SOFTWARE\Microsoft\EdgeUpdate\Clients\",
        ),
    ];
    for (hive, prefix) in paths {
        let root = RegKey::predef(hive);
        if let Ok(key) = root.open_subkey_with_flags(format!("{prefix}{GUID}"), KEY_READ) {
            if let Ok(pv) = key.get_value::<String, _>("pv") {
                if !pv.is_empty() {
                    return true;
                }
            }
        }
    }
    false
}

pub fn ensure(mut on_status: impl FnMut(&str)) -> Result<bool, String> {
    if is_installed() {
        return Ok(true);
    }
    on_status("Przygotowywanie środowiska…");
    let tmp = std::env::temp_dir().join("Octra-WebView2Setup.exe");
    let _ = std::fs::remove_file(&tmp);
    download(BOOTSTRAPPER, &tmp)?;
    let status = std::process::Command::new(&tmp)
        .args(["/silent", "/install"])
        .status()
        .map_err(|e| format!("Nie udało się przygotować środowiska: {e}"))?;
    let _ = std::fs::remove_file(&tmp);
    if status.success() || is_installed() {
        Ok(true)
    } else {
        Ok(false)
    }
}

fn download(url: &str, dest: &std::path::Path) -> Result<(), String> {
    let resp = ureq::get(url)
        .timeout(std::time::Duration::from_secs(120))
        .call()
        .map_err(|e| format!("Nie udało się pobrać składnika środowiska: {e}"))?;
    let mut file = std::fs::File::create(dest).map_err(|e| e.to_string())?;
    std::io::copy(&mut resp.into_reader(), &mut file).map_err(|e| e.to_string())?;
    Ok(())
}
