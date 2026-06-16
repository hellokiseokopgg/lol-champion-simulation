use regex::Regex;
use std::error::Error;

pub struct OpggScraper;

impl OpggScraper {
    pub fn fetch_items(url: &str) -> Result<Vec<String>, Box<dyn Error>> {
        // Example URL: https://op.gg/lol/summoners/kr/nodfan-KR1/matches/...
        // Extract "nodfan" from URL
        let re_summoner = Regex::new(r"/summoners/[^/]+/([^/\-]+)")?;
        let summoner_name = if let Some(caps) = re_summoner.captures(url) {
            caps.get(1).unwrap().as_str().to_string()
        } else {
            return Err("Failed to parse summoner name from URL".into());
        };

        // Fetch HTML with User-Agent
        let client = reqwest::blocking::Client::builder()
            .user_agent("Mozilla/5.0 (Macintosh; Intel Mac OS X 10_15_7) AppleWebKit/537.36")
            .build()?;
        let res = client.get(url).send()?;
        let status = res.status();
        let html = res.text()?;
        
        if !status.is_success() {
            return Err(format!("OP.GG returned status: {}", status).into());
        }

        let re_chunk = Regex::new(r#"self\.__next_f\.push\(\[1,"(.*?)\]\)"#)?;
        
        let mut item_ids = Vec::new();

        for cap in re_chunk.captures_iter(&html) {
            let chunk = cap.get(1).unwrap().as_str();
            let decoded = chunk.replace("\\\"", "\"").replace("\\\\", "\\");

            // Look for the summoner name and an items array
            if decoded.contains(&summoner_name) && decoded.contains("\"items\":[") {
                // Find all items arrays
                let re_items = Regex::new(r#""items":\[(.*?)\]"#)?;
                for items_cap in re_items.captures_iter(&decoded) {
                    let items_str = items_cap.get(1).unwrap().as_str();
                    // We only want the items for THIS summoner. 
                    // This is a naive approach: we take the first items array found in a chunk containing the summoner.
                    // A proper parser would parse the JSON, but since it's deeply nested and escaped, we use regex.
                    let re_id = Regex::new(r#""id":(\d+)"#)?;
                    for id_cap in re_id.captures_iter(items_str) {
                        item_ids.push(id_cap.get(1).unwrap().as_str().to_string());
                    }
                    if !item_ids.is_empty() {
                        return Ok(item_ids);
                    }
                }
            }
        }

        if item_ids.is_empty() {
            return Err("Items not found in OP.GG page".into());
        }

        Ok(item_ids)
    }
}
