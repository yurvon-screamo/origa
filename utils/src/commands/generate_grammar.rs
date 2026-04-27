use crate::api::{ReasoningConfig, generate_grammar_description};
use crate::utils::get_base_path;
use futures::stream::{FuturesUnordered, StreamExt};
use origa::domain::OrigaError;
use serde_json::Value;
use std::fs;
use std::path::PathBuf;
use std::sync::Arc;
use tokio::sync::Semaphore;

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
    // Strip UTF-8 BOM if present
    let content = content.strip_prefix('\u{FEFF}').unwrap_or(&content);
    serde_json::from_str(content).map_err(|e| OrigaError::TokenizerError {
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

struct GeneratedContent {
    ru_title: String,
    ru_short_description: String,
    ru_md_description: String,
    en_title: String,
    en_short_description: String,
    en_md_description: String,
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

#[allow(clippy::too_many_arguments)]
pub async fn run_generate_grammar(
    rule_id: Option<String>,
    all: bool,
    indices: Option<String>,
    level: Option<String>,
    api_base: String,
    api_key: String,
    workers: usize,
    dry_run: bool,
    grammar_path: Option<PathBuf>,
    model: Option<String>,
    reasoning: bool,
) -> Result<(), OrigaError> {
    let model = model.as_deref().unwrap_or("llm").to_string();
    let reasoning_config = if reasoning {
        Some(ReasoningConfig::high())
    } else {
        None
    };

    // Parse indices if provided
    let target_indices: Option<Vec<usize>> = indices.map(|s| {
        s.split(',')
            .filter_map(|part| part.trim().parse().ok())
            .collect()
    });

    if rule_id.is_none() && !all && target_indices.is_none() {
        return Err(OrigaError::TokenizerError {
            reason:
                "Specify --rule-id <ID>, --indices <idx1,idx2,...>, or use --all for batch mode"
                    .to_string(),
        });
    }

    let path = resolve_grammar_path(grammar_path.as_ref());
    tracing::info!("Loading grammar from: {}", path.display());

    let grammar_data = load_grammar_json(&path)?;

    let grammar_array = grammar_data
        .get("grammar")
        .and_then(|g| g.as_array())
        .ok_or_else(|| OrigaError::TokenizerError {
            reason: "grammar.json has invalid structure: missing 'grammar' array".to_string(),
        })?;

    // When indices are provided, collect all rules first (ignore level filter)
    let collect_all = all || target_indices.is_some();
    let mut rules = collect_rules(
        grammar_array,
        rule_id.as_deref(),
        collect_all,
        level.as_deref(),
    )?;

    // Filter by indices if provided
    if let Some(ref indices) = target_indices {
        rules.retain(|rule| indices.contains(&rule.index));
    }

    if rules.is_empty() {
        tracing::info!("No rules to process");
        return Ok(());
    }

    tracing::info!(
        "Found {} rule(s) to process with {} worker(s)",
        rules.len(),
        workers
    );

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
    let semaphore = Arc::new(Semaphore::new(workers.max(1)));
    let api_base = Arc::new(api_base);
    let api_key = Arc::new(api_key);
    let model = Arc::new(model);

    let mut tasks: FuturesUnordered<_> = rules
        .into_iter()
        .enumerate()
        .map(|(task_index, rule_info)| {
            let permit = semaphore.clone().acquire_owned();
            let api_base = api_base.clone();
            let api_key = api_key.clone();
            let model = model.clone();
            let reasoning_config = reasoning_config.clone();

            async move {
                let _permit = permit.await.expect("Semaphore error");
                let result = generate_grammar_description(
                    &api_base,
                    &api_key,
                    &model,
                    &rule_info.title,
                    &rule_info.level,
                    None,
                    reasoning_config,
                )
                .await;

                tracing::info!(
                    "[{}/{}] Generated: {} ({})",
                    task_index + 1,
                    total,
                    rule_info.title,
                    rule_info.level
                );

                (task_index, rule_info.index, result)
            }
        })
        .collect();

    let mut results: Vec<(usize, usize, Result<GeneratedContent, OrigaError>)> = Vec::new();
    let mut errors: Vec<String> = Vec::new();

    while let Some((task_index, rule_index, result)) = tasks.next().await {
        match result {
            Ok(bilingual) => {
                results.push((
                    task_index,
                    rule_index,
                    Ok(GeneratedContent {
                        ru_title: bilingual.ru.title,
                        ru_short_description: bilingual.ru.short_description,
                        ru_md_description: bilingual.ru.md_description,
                        en_title: bilingual.en.title,
                        en_short_description: bilingual.en.short_description,
                        en_md_description: bilingual.en.md_description,
                    }),
                ));
            },
            Err(e) => {
                let msg = format!("Rule index {}: {}", rule_index, e);
                tracing::error!("{}", msg);
                errors.push(msg);
            },
        }
    }

    if !errors.is_empty() {
        tracing::error!("{} rule(s) failed to generate", errors.len());
        for err in &errors {
            tracing::error!("  - {}", err);
        }
        return Err(OrigaError::TokenizerError {
            reason: format!("{} rule(s) failed to generate", errors.len()),
        });
    }

    // Apply all results to grammar data
    let mut grammar_data = load_grammar_json(&path)?;
    let grammar_array = grammar_data
        .get_mut("grammar")
        .and_then(|g| g.as_array_mut())
        .ok_or_else(|| OrigaError::TokenizerError {
            reason: "grammar.json has invalid structure: missing 'grammar' array".to_string(),
        })?;

    for (_, rule_index, content_result) in results {
        let content = content_result?;
        if let Some(rule) = grammar_array.get_mut(rule_index) {
            update_rule_content(
                rule,
                "Russian",
                &content.ru_title,
                &content.ru_short_description,
                &content.ru_md_description,
            );
            update_rule_content(
                rule,
                "English",
                &content.en_title,
                &content.en_short_description,
                &content.en_md_description,
            );
        }
    }

    save_grammar_json(&path, &grammar_data)?;

    tracing::info!("Done. Processed {} rule(s)", total);
    Ok(())
}
