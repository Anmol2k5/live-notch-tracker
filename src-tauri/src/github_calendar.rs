#[derive(Debug, PartialEq)]
pub struct ContributionDay {
    pub date: String,
    pub count: u32,
    pub level: u8,
}

pub fn valid_username(name: &str) -> bool {
    !name.is_empty()
        && name.len() <= 39
        && !name.starts_with('-')
        && !name.ends_with('-')
        && name.bytes().all(|b| b.is_ascii_alphanumeric() || b == b'-')
}

fn attribute<'a>(tag: &'a str, name: &str) -> Option<&'a str> {
    tag.split_once(&format!(" {name}=\""))?.1.split('"').next()
}

/// Parse GitHub's public calendar, matching tooltip counts to their cell IDs.
/// Unknown markup is an error, never a fabricated zero-activity calendar.
pub fn parse_calendar(html: &str) -> Result<Vec<ContributionDay>, String> {
    let mut counts = std::collections::HashMap::new();
    for tooltip in html.split("<tool-tip ").skip(1) {
        let Some((tag, content)) = tooltip.split_once('>') else {
            continue;
        };
        let Some(id) = attribute(&format!(" {tag}"), "for").map(str::to_owned) else {
            continue;
        };
        let text = content.split("</tool-tip>").next().unwrap_or("").trim();
        let count = if text.starts_with("No contributions") {
            Some(0)
        } else {
            text.split_whitespace()
                .next()
                .and_then(|n| n.replace(',', "").parse::<u32>().ok())
        };
        if let Some(count) = count {
            counts.insert(id, count);
        }
    }
    let mut days = Vec::new();
    for cell in html.split("<td ").skip(1) {
        let tag = format!(" {}", cell.split('>').next().unwrap_or(""));
        let Some(date) = attribute(&tag, "data-date") else {
            continue;
        };
        let error = || "GitHub's calendar format could not be read. Try again later.".to_string();
        if date.len() != 10
            || !date.bytes().enumerate().all(|(i, b)| {
                if i == 4 || i == 7 {
                    b == b'-'
                } else {
                    b.is_ascii_digit()
                }
            })
        {
            return Err(error());
        }
        let level = attribute(&tag, "data-level")
            .and_then(|s| s.parse::<u8>().ok())
            .filter(|n| *n <= 4)
            .ok_or_else(error)?;
        let count = attribute(&tag, "id")
            .and_then(|id| counts.get(id))
            .copied()
            .ok_or_else(error)?;
        days.push(ContributionDay {
            date: date.into(),
            count,
            level,
        });
    }
    days.sort_by(|a, b| a.date.cmp(&b.date));
    days.dedup_by(|a, b| a.date == b.date);
    if days.is_empty() {
        return Err("GitHub returned no contribution calendar for this username.".into());
    }
    Ok(days)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn rejects_paths_and_invalid_github_usernames() {
        assert!(valid_username("Anmol2k5"));
        for name in ["", "-a", "a-", "../secrets", "a?tab=x", "a b", "a_b"] {
            assert!(!valid_username(name), "{name}");
        }
        assert!(!valid_username(&"a".repeat(40)));
    }

    #[test]
    fn parses_real_calendar_cells_and_matches_tooltips_by_id() {
        let html = r#"<td data-date="2026-11-02" id="day-2" data-level="3"></td>
          <td data-level="0" data-date="2026-11-01" id="day-1"></td>
          <tool-tip for="day-1">No contributions on November 1st.</tool-tip>
          <tool-tip for="day-2">12 contributions on November 2nd.</tool-tip>"#;
        let days = parse_calendar(html).unwrap();
        assert_eq!(days.len(), 2);
        assert_eq!(days[0].date, "2026-11-01");
        assert_eq!(days[0].count, 0);
        assert_eq!(days[1].count, 12);
        assert_eq!(days[1].level, 3);
    }

    #[test]
    fn refuses_unknown_html_instead_of_fabricating_an_empty_calendar() {
        assert!(parse_calendar("<html>Sign in</html>").is_err());
        assert!(
            parse_calendar(r#"<td data-date="2026-11-02" data-level="1" id="day"></td>"#).is_err()
        );
    }
}
