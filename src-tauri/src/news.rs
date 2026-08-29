//! Pobieranie aktualności Mojang (RSS) po stronie Rust — bez blokady CSP w WebView.

use serde::Serialize;

use crate::error::{Error, Result};

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct MojangNewsItem {
    pub id: String,
    pub title: String,
    pub link: String,
    pub summary: String,
    pub published: String,
}

const RSS_URL: &str = "https://www.minecraft.net/en-us/feeds/community-content/rss";

pub async fn fetch_news(http: &reqwest::Client) -> Result<Vec<MojangNewsItem>> {
    let xml = http
        .get(RSS_URL)
        .send()
        .await?
        .error_for_status()?
        .text()
        .await?;
    parse_rss(&xml)
}

fn parse_rss(xml: &str) -> Result<Vec<MojangNewsItem>> {
    let re = regex::Regex::new(r"(?is)<item>(.*?)</item>").map_err(|e| Error::msg(e.to_string()))?;
    let mut out = Vec::new();
    for cap in re.captures_iter(xml).take(6) {
        let block = cap.get(1).map(|m| m.as_str()).unwrap_or("");
        let title = tag_text(block, "title").unwrap_or_else(|| "Aktualność".into());
        let link = tag_text(block, "link").unwrap_or_default();
        let published = tag_text(block, "pubDate").unwrap_or_default();
        let mut summary = tag_text(block, "description").unwrap_or_default();
        summary = strip_html(&summary);
        if summary.len() > 180 {
            summary.truncate(180);
            summary.push('…');
        }
        out.push(MojangNewsItem {
            id: if link.is_empty() { title.clone() } else { link.clone() },
            title,
            link,
            summary,
            published,
        });
    }
    if out.is_empty() {
        return Err(Error::msg("Brak wpisów w kanale RSS Mojang."));
    }
    Ok(out)
}

fn tag_text(block: &str, tag: &str) -> Option<String> {
    let re = regex::Regex::new(&format!(r"(?is)<{tag}[^>]*>(.*?)</{tag}>")).ok()?;
    let inner = re.captures(block)?.get(1)?.as_str();
    let text = inner
        .replace("<![CDATA[", "")
        .replace("]]>", "")
        .trim()
        .to_string();
    if text.is_empty() { None } else { Some(text) }
}

fn strip_html(s: &str) -> String {
    let re = regex::Regex::new(r"<[^>]+>").unwrap();
    re.replace_all(s, "").trim().to_string()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_sample_item() {
        let xml = r#"<rss><channel><item><title>Test</title><link>https://x</link><description><p>Hi</p></description><pubDate>Mon, 01 Jan 2024</pubDate></item></channel></rss>"#;
        let items = parse_rss(xml).unwrap();
        assert_eq!(items.len(), 1);
        assert_eq!(items[0].title, "Test");
    }
}
