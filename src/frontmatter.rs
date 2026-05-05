use crate::error::{IcsError, Result};
use serde_yaml::Mapping;
use std::fs;

const FM_DELIM: &str = "---";

/// Split a markdown file into optional YAML frontmatter (without fences) and body text.
pub fn split_frontmatter(raw: &str) -> Result<(Option<String>, String)> {
    let lines: Vec<&str> = raw.lines().collect();
    if lines.is_empty() {
        return Ok((None, String::new()));
    }
    if lines[0].trim() != FM_DELIM {
        return Ok((None, raw.to_string()));
    }
    let mut end = None;
    for (i, line) in lines.iter().enumerate().skip(1) {
        if line.trim() == FM_DELIM {
            end = Some(i);
            break;
        }
    }
    let end = end.ok_or_else(|| IcsError::Msg("unclosed frontmatter fence".into()))?;
    let yaml = lines[1..end].join("\n");
    let body = if end + 1 < lines.len() {
        lines[(end + 1)..].join("\n")
    } else {
        String::new()
    };
    if body.is_empty() {
        Ok((Some(yaml), body))
    } else {
        Ok((Some(yaml), format!("{}\n", body)))
    }
}

pub fn merge_frontmatter(raw: &str, patch: Mapping) -> Result<String> {
    let (existing_yaml, body) = split_frontmatter(raw)?;
    let mut merged = Mapping::new();
    if let Some(y) = existing_yaml {
        let old: Mapping = serde_yaml::from_str(&y).map_err(|e| IcsError::Msg(e.to_string()))?;
        merged = old;
    }
    for (k, v) in patch {
        merged.insert(k, v);
    }
    let yaml_out = serde_yaml::to_string(&merged).map_err(|e| IcsError::Msg(e.to_string()))?;
    Ok(format!("{FM_DELIM}\n{yaml_out}{FM_DELIM}\n{body}"))
}

pub fn parse_frontmatter_map(raw: &str) -> Result<(Option<Mapping>, String)> {
    let (yaml, body) = split_frontmatter(raw)?;
    let map = if let Some(y) = yaml {
        Some(serde_yaml::from_str::<Mapping>(&y).map_err(|e| IcsError::Msg(e.to_string()))?)
    } else {
        None
    };
    Ok((map, body))
}

pub fn read_file_string(path: &std::path::Path) -> Result<String> {
    fs::read_to_string(path).map_err(IcsError::from)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn round_trip_frontmatter() {
        let raw = "---\ntitle: hi\n---\nbody\n";
        let (fm, body) = split_frontmatter(raw).unwrap();
        assert!(fm.is_some());
        assert!(body.contains("body"));
        let mut m = Mapping::new();
        m.insert(
            serde_yaml::Value::String("stratum_note_id".into()),
            serde_yaml::Value::Number(serde_yaml::Number::from(42)),
        );
        let out = merge_frontmatter(raw, m).unwrap();
        assert!(out.contains("stratum_note_id"));
    }
}
