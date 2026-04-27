use crate::api::generate_grammar_description;
use crate::utils::get_base_path;
use origa::domain::OrigaError;
use serde_json::Value;
use std::fs;
use std::path::PathBuf;
use std::time::Duration;
use tokio::time::sleep;

fn resolve_grammar_path(custom_path: Option<&PathBuf>) -> PathBuf {
    match custom_path {
        Some(p) => p.clone(),
        None => get_base_path()
            .join("cdn")
            .join("grammar")
            .join("grammar.json"),
    }
}

fn load_grammar_json(path: &PathBuf) -> Result<Value, OrigaError> {
    let content = fs::read_to_string(path).map_err(|e| OrigaError::TokenizerError {
        reason: format!("Failed to read {}: {}", path.display(), e),
    })?;
    serde_json::from_str(&content).map_err(|e| OrigaError::TokenizerError {
        reason: format!("Failed to parse {}: {}", path.display(), e),
    })
}

fn save_grammar_json(path: &PathBuf, data: &Value) -> Result<(), OrigaError> {
    let json = serde_json::to_string_pretty(data).map_err(|e| OrigaError::TokenizerError {
        reason: format!("Failed to serialize grammar JSON: {}", e),
    })?;
    fs::write(path, json).map_err(|e| OrigaError::TokenizerError {
        reason: format!("Failed to write {}: {}", path.display(), e),
    })
}

struct RuleInfo {
    index: usize,
    rule_id: String,
    level: String,
    title: String,
}

fn collect_rules(
    grammar: &[Value],
    rule_id: Option<&str>,
    all: bool,
    level: Option<&str>,
) -> Result<Vec<RuleInfo>, OrigaError> {
    let mut rules = Vec::new();

    if let Some(target_id) = rule_id {
        let found = grammar
            .iter()
            .enumerate()
            .find(|(_, rule)| {
                rule.get("rule_id")
                    .and_then(|v| v.as_str())
                    .is_some_and(|id| id == target_id)
            })
            .ok_or_else(|| OrigaError::TokenizerError {
                reason: format!("Rule with id '{}' not found", target_id),
            })?;

        let (index, rule) = found;
        rules.push(RuleInfo {
            index,
            rule_id: target_id.to_string(),
            level: extract_level(rule),
            title: extract_title(rule),
        });
        return Ok(rules);
    }

    if all {
        for (index, rule) in grammar.iter().enumerate() {
            let rule_level = extract_level(rule);
            if let Some(filter_level) = level {
                if rule_level != filter_level {
                    continue;
                }
            }
            rules.push(RuleInfo {
                index,
                rule_id: extract_rule_id(rule),
                level: rule_level,
                title: extract_title(rule),
            });
        }
    }

    Ok(rules)
}

fn extract_rule_id(rule: &Value) -> String {
    rule.get("rule_id")
        .and_then(|v| v.as_str())
        .unwrap_or("unknown")
        .to_string()
}

fn extract_level(rule: &Value) -> String {
    rule.get("level")
        .and_then(|v| v.as_str())
        .unwrap_or("N5")
        .to_string()
}

fn extract_title(rule: &Value) -> String {
    rule.get("content")
        .and_then(|c| c.get("Russian"))
        .and_then(|r| r.get("title"))
        .and_then(|v| v.as_str())
        .or_else(|| {
            rule.get("content")
                .and_then(|c| c.get("English"))
                .and_then(|e| e.get("title"))
                .and_then(|v| v.as_str())
        })
        .unwrap_or("unknown")
        .to_string()
}

fn update_rule_content(rule: &mut Value, language: &str, title: &str, short_desc: &str, md: &str) {
    let content = rule.get_mut("content").and_then(|c| c.get_mut(language));

    if let Some(lang_obj) = content {
        lang_obj["title"] = Value::String(title.to_string());
        lang_obj["short_description"] = Value::String(short_desc.to_string());
        lang_obj["md_description"] = Value::String(md.to_string());
    }
}

async fn process_single_rule(
    grammar_data: &mut Value,
    rule_info: &RuleInfo,
    index: usize,
    total: usize,
    path: &PathBuf,
    api_base: &str,
    api_key: &str,
) -> Result<(), OrigaError> {
    tracing::info!(
        "[{}/{}] Generating RU description for: {} ({})",
        index + 1,
        total,
        rule_info.title,
        rule_info.level
    );

    let ru_content = generate_grammar_description(
        api_base,
        api_key,
        &rule_info.title,
        &rule_info.level,
        None,
        "russian",
    )
    .await?;

    sleep(Duration::from_secs(1)).await;

    tracing::info!(
        "[{}/{}] Generating EN description for: {} ({})",
        index + 1,
        total,
        rule_info.title,
        rule_info.level
    );

    let en_content = generate_grammar_description(
        api_base,
        api_key,
        &rule_info.title,
        &rule_info.level,
        None,
        "english",
    )
    .await?;

    let grammar_array = grammar_data
        .get_mut("grammar")
        .and_then(|g| g.as_array_mut())
        .ok_or_else(|| OrigaError::TokenizerError {
            reason: "grammar.json has invalid structure: missing 'grammar' array".to_string(),
        })?;

    if let Some(rule) = grammar_array.get_mut(rule_info.index) {
        update_rule_content(
            rule,
            "Russian",
            &ru_content.title,
            &ru_content.short_description,
            &ru_content.md_description,
        );
        update_rule_content(
            rule,
            "English",
            &en_content.title,
            &en_content.short_description,
            &en_content.md_description,
        );
    }

    save_grammar_json(path, grammar_data)?;

    tracing::info!(
        "[{}/{}] Updated rule: {} ({})",
        index + 1,
        total,
        rule_info.title,
        rule_info.level
    );

    Ok(())
}

#[allow(clippy::too_many_arguments)]
pub async fn run_generate_grammar(
    rule_id: Option<String>,
    all: bool,
    level: Option<String>,
    api_base: String,
    api_key: String,
    _workers: usize,
    dry_run: bool,
    grammar_path: Option<PathBuf>,
) -> Result<(), OrigaError> {
    if rule_id.is_none() && !all {
        return Err(OrigaError::TokenizerError {
            reason: "Specify --rule-id <ID> or use --all for batch mode".to_string(),
        });
    }

    let path = resolve_grammar_path(grammar_path.as_ref());
    tracing::info!("Loading grammar from: {}", path.display());

    let mut grammar_data = load_grammar_json(&path)?;

    let grammar_array = grammar_data
        .get("grammar")
        .and_then(|g| g.as_array())
        .ok_or_else(|| OrigaError::TokenizerError {
            reason: "grammar.json has invalid structure: missing 'grammar' array".to_string(),
        })?;

    let rules = collect_rules(grammar_array, rule_id.as_deref(), all, level.as_deref())?;

    if rules.is_empty() {
        tracing::info!("No rules to process");
        return Ok(());
    }

    tracing::info!("Found {} rule(s) to process", rules.len());

    if dry_run {
        for (i, rule) in rules.iter().enumerate() {
            tracing::info!(
                "  [{}/{}] {} ({}) - {}",
                i + 1,
                rules.len(),
                rule.title,
                rule.level,
                rule.rule_id
            );
        }
        return Ok(());
    }

    let total = rules.len();
    for (i, rule_info) in rules.iter().enumerate() {
        process_single_rule(
            &mut grammar_data,
            rule_info,
            i,
            total,
            &path,
            &api_base,
            &api_key,
        )
        .await?;

        if i < total - 1 {
            sleep(Duration::from_secs(1)).await;
        }
    }

    tracing::info!("Done. Processed {} rule(s)", total);
    Ok(())
}
