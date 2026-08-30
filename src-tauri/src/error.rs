use serde::Serialize;
use std::fmt;

#[derive(Debug)]
pub enum Error {
    Msg(String),
    Io(std::io::Error),
    Http(reqwest::Error),
    Json(serde_json::Error),
}

impl Error {
    pub fn msg(m: impl Into<String>) -> Self {
        Self::Msg(m.into())
    }
}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Msg(m) => write!(f, "{m}"),
            Self::Io(e) => {
                if e.raw_os_error() == Some(5) {
                    write!(
                        f,
                        "Odmowa dostępu do pliku. Zamknij Minecraft (javaw.exe w Menedżerze zadań), \
                         wyłącz blokadę folderu przez antywirus lub uruchom Octra jako administrator. ({e})"
                    )
                } else {
                    write!(f, "Błąd pliku: {e}")
                }
            }
            Self::Http(e) => write!(f, "Błąd sieci: {e}"),
            Self::Json(e) => write!(f, "Błąd JSON: {e}"),
        }
    }
}

impl std::error::Error for Error {}

impl Serialize for Error {
    fn serialize<S: serde::Serializer>(&self, serializer: S) -> std::result::Result<S::Ok, S::Error> {
        serializer.serialize_str(&self.to_string())
    }
}

impl From<std::io::Error> for Error {
    fn from(value: std::io::Error) -> Self {
        Self::Io(value)
    }
}

impl From<reqwest::Error> for Error {
    fn from(value: reqwest::Error) -> Self {
        Self::Http(value)
    }
}

impl From<serde_json::Error> for Error {
    fn from(value: serde_json::Error) -> Self {
        Self::Json(value)
    }
}

impl From<zip::result::ZipError> for Error {
    fn from(value: zip::result::ZipError) -> Self {
        let text = value.to_string();
        let lower = text.to_ascii_lowercase();
        if lower.contains("eocd") || lower.contains("invalid zip archive") {
            Self::Msg(
                "Archiwum ZIP jest niekompletne lub uszkodzone (ucięty plik — brak końca archiwum). Jeśli to autorska paczka, podmień `packs/Cobblemon vasst 1.0.0.mrpack` na pełne archiwum."
                    .into(),
            )
        } else {
            Self::Msg(format!("Błąd archiwum: {text}"))
        }
    }
}

pub type Result<T> = std::result::Result<T, Error>;
