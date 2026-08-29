//! Kanał wydania: Dev (debug) vs Stable (release). Stable włącza auto-updater.

use std::sync::OnceLock;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Channel {
    Dev,
    Stable,
}

impl Channel {
    pub fn as_str(self) -> &'static str {
        match self {
            Channel::Dev => "dev",
            Channel::Stable => "stable",
        }
    }

    pub fn display_name(self) -> &'static str {
        "Octra Launcher"
    }

    pub fn badge(self) -> Option<&'static str> {
        match self {
            Channel::Dev => Some("Dev"),
            Channel::Stable => None,
        }
    }

    pub fn is_stable(self) -> bool {
        matches!(self, Channel::Stable)
    }

    pub fn data_dir_name(self) -> &'static str {
        ".octralauncher"
    }

    pub fn window_title(self) -> String {
        match self.badge() {
            Some(b) => format!("{} [{b}]", self.display_name()),
            None => self.display_name().to_string(),
        }
    }
}

pub fn current() -> Channel {
    static C: OnceLock<Channel> = OnceLock::new();
    *C.get_or_init(|| {
        if let Ok(raw) = std::env::var("OCTRA_CHANNEL") {
            match raw.trim().to_ascii_lowercase().as_str() {
                "stable" | "release" | "prod" => return Channel::Stable,
                "dev" | "debug" => return Channel::Dev,
                _ => {}
            }
        }
        #[cfg(debug_assertions)]
        {
            Channel::Dev
        }
        #[cfg(not(debug_assertions))]
        {
            Channel::Stable
        }
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn experimental_uses_octralauncher_dir() {
        assert_eq!(current().data_dir_name(), ".octralauncher");
    }

    #[test]
    fn dev_has_badge() {
        assert_eq!(Channel::Dev.badge(), Some("Dev"));
        assert_eq!(Channel::Stable.badge(), None);
    }
}
