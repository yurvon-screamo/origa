//! Phrase index: owned JSON-built form, its deterministic CDN blob twin and
//! the zero-copy archived view.

use std::collections::{BTreeMap, BTreeSet, HashMap, HashSet};

use serde::Deserialize;
use ulid::Ulid;

use crate::domain::OrigaError;

/// One phrase as stored in the index: its token list, the data chunk it
/// lives in and the grammar rules it exercises.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct IndexEntry {
    id: Ulid,
    tokens: Vec<String>,
    chunk_id: u32,
    grammar_rules: Vec<Ulid>,
}

impl IndexEntry {
    pub fn id(&self) -> &Ulid {
        &self.id
    }

    pub fn tokens(&self) -> &[String] {
        &self.tokens
    }

    pub fn chunk_id(&self) -> u32 {
        self.chunk_id
    }

    pub fn grammar_rules(&self) -> &[Ulid] {
        &self.grammar_rules
    }
}

pub struct PhraseIndex {
    entries: HashMap<Ulid, IndexEntry>,
    token_to_phrases: HashMap<String, Vec<Ulid>>,
    all_ids: HashSet<Ulid>,
    pub(super) version: u32,
    pub(super) hash: String,
}

#[derive(Deserialize)]
struct IndexFile {
    #[serde(rename = "v")]
    version: u32,
    #[serde(rename = "h")]
    hash: String,
    #[serde(rename = "phrases")]
    phrases: Vec<IndexEntryRaw>,
}

#[derive(Deserialize)]
struct IndexEntryRaw {
    #[serde(rename = "i")]
    id: Ulid,
    #[serde(rename = "t")]
    tokens: Vec<String>,
    #[serde(rename = "c")]
    chunk_id: u32,
    #[serde(rename = "g", default)]
    grammar_rules: Option<Vec<String>>,
}

impl PhraseIndex {
    /// Build the index from JSON text without installing it into the
    /// global slot. Used by the loader core and its tests.
    pub fn from_json(json: &str) -> Result<Self, OrigaError> {
        let file: IndexFile =
            serde_json::from_str(json).map_err(|e| OrigaError::PhraseParseError {
                reason: format!("Failed to parse phrase index: {}", e),
            })?;

        let mut entries = HashMap::with_capacity(file.phrases.len());
        let mut token_to_phrases: HashMap<String, Vec<Ulid>> = HashMap::new();

        for raw in file.phrases {
            let id = raw.id;
            for token in &raw.tokens {
                token_to_phrases.entry(token.clone()).or_default().push(id);
            }
            let grammar_rules: Vec<Ulid> = raw
                .grammar_rules
                .unwrap_or_default()
                .into_iter()
                .filter_map(|s| Ulid::from_string(&s).ok())
                .collect();
            entries.insert(
                id,
                IndexEntry {
                    id,
                    tokens: raw.tokens,
                    chunk_id: raw.chunk_id,
                    grammar_rules,
                },
            );
        }

        let all_ids: HashSet<Ulid> = entries.keys().copied().collect();

        Ok(Self {
            entries,
            token_to_phrases,
            all_ids,
            version: file.version,
            hash: file.hash,
        })
    }

    pub(super) fn get_entry(&self, id: &Ulid) -> Option<&IndexEntry> {
        self.entries.get(id)
    }

    pub fn get_phrases_by_token(&self, token: &str) -> Vec<&IndexEntry> {
        self.token_to_phrases
            .get(token)
            .map(|ids| ids.iter().filter_map(|id| self.entries.get(id)).collect())
            .unwrap_or_default()
    }

    pub(super) fn iter_entries(&self) -> impl Iterator<Item = &IndexEntry> {
        self.entries.values()
    }

    pub(super) fn all_ids(&self) -> &HashSet<Ulid> {
        &self.all_ids
    }

    pub(super) fn len(&self) -> usize {
        self.entries.len()
    }
}

/// Build an index from JSON text without installing it into the global
/// slot. Mirrors `build_vocabulary_database_from_chunks`.
pub fn build_phrase_index_from_json(json: &str) -> Result<PhraseIndex, OrigaError> {
    PhraseIndex::from_json(json)
}

/// Deterministic CDN-blob form of the built index. Ulids are stored as
/// plain `u128` (the `ulid` crate's rkyv feature is not enabled and its
/// archived form lacks `Ord`, which the BTree collections require);
/// `u128` ordering equals `Ulid` ordering, so the serialized order — and
/// with it the manifest hash — stays stable across builds.
#[derive(rkyv::Archive, rkyv::Serialize, rkyv::Deserialize)]
pub struct PhraseIndexBlob {
    pub(super) entries: BTreeMap<u128, IndexEntryBlob>,
    pub(super) token_to_phrases: BTreeMap<String, Vec<u128>>,
    pub(super) all_ids: BTreeSet<u128>,
    pub(super) version: u32,
    pub(super) hash: String,
}

#[derive(rkyv::Archive, rkyv::Serialize, rkyv::Deserialize)]
pub struct IndexEntryBlob {
    pub(super) id: u128,
    pub(super) tokens: Vec<String>,
    pub(super) chunk_id: u32,
    pub(super) grammar_rules: Vec<u128>,
}

impl PhraseIndex {
    /// Deterministic CDN-blob form of this index.
    pub fn to_blob(&self) -> PhraseIndexBlob {
        PhraseIndexBlob {
            entries: self
                .entries
                .iter()
                .map(|(id, entry)| {
                    (
                        u128::from(*id),
                        IndexEntryBlob {
                            id: u128::from(*id),
                            tokens: entry.tokens.clone(),
                            chunk_id: entry.chunk_id,
                            grammar_rules: entry
                                .grammar_rules
                                .iter()
                                .map(|rule| u128::from(*rule))
                                .collect(),
                        },
                    )
                })
                .collect(),
            token_to_phrases: self
                .token_to_phrases
                .iter()
                .map(|(token, ids)| {
                    (
                        token.clone(),
                        ids.iter().map(|id| u128::from(*id)).collect(),
                    )
                })
                .collect(),
            all_ids: self.all_ids.iter().map(|id| u128::from(*id)).collect(),
            version: self.version,
            hash: self.hash.clone(),
        }
    }
}

/// Serialize the deterministic blob form (payload of a CDN blob).
pub fn serialize_phrase_index_blob_to_rkyv(blob: &PhraseIndexBlob) -> Result<Vec<u8>, OrigaError> {
    rkyv::to_bytes::<rkyv::rancor::Error>(blob)
        .map(|bytes| bytes.to_vec())
        .map_err(|e| OrigaError::PhraseParseError {
            reason: format!("failed to serialize phrase index blob: {e}"),
        })
}

impl ArchivedPhraseIndexBlob {
    fn entry_by_raw_id(&self, raw_id: u128) -> Option<IndexEntry> {
        let archived = self
            .entries
            .get(&rkyv::rend::u128_le::from_native(raw_id))?;
        Some(clone_entry(archived))
    }

    pub fn get_entry(&self, id: &Ulid) -> Option<IndexEntry> {
        self.entry_by_raw_id(u128::from(*id))
    }

    pub fn get_phrases_by_token(&self, token: &str) -> Vec<IndexEntry> {
        self.token_to_phrases
            .get(token)
            .map(|ids| {
                ids.iter()
                    .filter_map(|raw| self.entry_by_raw_id(u128::from(*raw)))
                    .collect()
            })
            .unwrap_or_default()
    }

    pub fn iter_entries(&self) -> impl Iterator<Item = IndexEntry> + '_ {
        self.entries.values().map(clone_entry)
    }

    pub fn all_ids(&self) -> HashSet<Ulid> {
        self.all_ids
            .iter()
            .map(|raw| Ulid::from(u128::from(*raw)))
            .collect()
    }

    pub fn version(&self) -> (u32, String) {
        (u32::from(self.version), self.hash.as_str().to_string())
    }

    pub fn len(&self) -> usize {
        self.entries.len()
    }

    pub fn is_empty(&self) -> bool {
        self.entries.is_empty()
    }
}

fn clone_entry(entry: &rkyv::Archived<IndexEntryBlob>) -> IndexEntry {
    IndexEntry {
        id: Ulid::from(u128::from(entry.id)),
        tokens: entry
            .tokens
            .iter()
            .map(|token| token.as_str().to_string())
            .collect(),
        chunk_id: u32::from(entry.chunk_id),
        grammar_rules: entry
            .grammar_rules
            .iter()
            .map(|rule| Ulid::from(u128::from(*rule)))
            .collect(),
    }
}

/// Zero-copy archived view of a blob payload. The payload is copied once
/// into an immortal `AlignedVec` (checked `access` requires properly
/// aligned bytes); a validation failure leaves the leaked bytes
/// unreclaimed until reload while callers fall back to the JSON path.
pub fn access_phrase_blob(payload: &[u8]) -> Result<&'static ArchivedPhraseIndexBlob, OrigaError> {
    crate::dictionary::cdn_blob::access_leaked::<ArchivedPhraseIndexBlob>(payload).map_err(|e| {
        OrigaError::PhraseParseError {
            reason: format!("failed to access phrase index blob: {e}"),
        }
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    fn index_json() -> &'static str {
        r#"{"v":1,"h":"test","total":2,"phrases":[{"i":"01KPJ5S3N1DRFFD236Z4EZ03HJ","t":["hello","world"],"c":0},{"i":"01KPJ5S3N1DRFFD236Z4EZ03HK","t":["goodbye","world"],"c":0}]}"#
    }

    fn index_json_with_grammar() -> &'static str {
        r#"{"v":1,"h":"test","total":1,"phrases":[{"i":"01KPJ5S3N1DRFFD236Z4EZ03HJ","t":["hello","world"],"c":0,"g":["01KPJ5S3N1DRFFD236Z4EZ03HK","01KPJ5S3N1DRFFD236Z4EZ03HM"]}]}"#
    }

    fn first_id() -> Ulid {
        Ulid::from_string("01KPJ5S3N1DRFFD236Z4EZ03HJ").expect("valid ULID")
    }

    #[test]
    fn from_json_valid_loads_entries() {
        let index = PhraseIndex::from_json(index_json()).expect("valid JSON should parse");
        let entries: Vec<_> = index.iter_entries().collect();
        assert_eq!(entries.len(), 2);
        assert_eq!(index.token_to_phrases.len(), 3);
    }

    #[test]
    fn from_json_invalid_json_returns_error() {
        let result = PhraseIndex::from_json("not json");
        assert!(result.is_err());
    }

    #[test]
    fn get_entry_found() {
        let index = PhraseIndex::from_json(index_json()).expect("valid JSON");
        let entry = index.get_entry(&first_id());
        assert!(entry.is_some());
        assert_eq!(entry.expect("entry").tokens(), &["hello", "world"]);
    }

    #[test]
    fn get_entry_not_found() {
        let index = PhraseIndex::from_json(index_json()).expect("valid JSON");
        let missing_id = Ulid::new();
        assert!(index.get_entry(&missing_id).is_none());
    }

    #[test]
    fn get_phrases_by_token_found() {
        let index = PhraseIndex::from_json(index_json()).expect("valid JSON");
        let results = index.get_phrases_by_token("world");
        assert_eq!(results.len(), 2);
    }

    #[test]
    fn get_phrases_by_token_not_found() {
        let index = PhraseIndex::from_json(index_json()).expect("valid JSON");
        let results = index.get_phrases_by_token(" nonexistent");
        assert!(results.is_empty());
    }

    #[test]
    fn all_ids_returns_all() {
        let index = PhraseIndex::from_json(index_json()).expect("valid JSON");
        let ids = index.all_ids();
        assert_eq!(ids.len(), 2);
    }

    #[test]
    fn version_returns_values() {
        let index = PhraseIndex::from_json(index_json()).expect("valid JSON");
        assert_eq!(index.version, 1);
        assert_eq!(index.hash, "test");
    }

    #[test]
    fn grammar_rules_parsed_from_g_field() {
        let index = PhraseIndex::from_json(index_json_with_grammar()).expect("valid JSON");
        let entry = index.get_entry(&first_id()).expect("entry should exist");
        assert_eq!(entry.grammar_rules().len(), 2);
        assert_eq!(
            entry.grammar_rules()[0],
            Ulid::from_string("01KPJ5S3N1DRFFD236Z4EZ03HK").unwrap()
        );
    }

    #[test]
    fn grammar_rules_empty_when_g_field_absent() {
        let index = PhraseIndex::from_json(index_json()).expect("valid JSON");
        let entry = index.get_entry(&first_id()).expect("entry should exist");
        assert!(entry.grammar_rules().is_empty());
    }

    #[test]
    fn blob_serialization_is_deterministic_across_builds() {
        let first = PhraseIndex::from_json(index_json()).unwrap().to_blob();
        let second = PhraseIndex::from_json(index_json()).unwrap().to_blob();
        assert_eq!(
            serialize_phrase_index_blob_to_rkyv(&first).unwrap(),
            serialize_phrase_index_blob_to_rkyv(&second).unwrap()
        );
    }

    /// The archived view must answer exactly like the owned index built
    /// from the same JSON — the zero-copy fast path may not change lookup
    /// semantics.
    fn parity_fixture() -> (PhraseIndex, &'static ArchivedPhraseIndexBlob) {
        let index = PhraseIndex::from_json(index_json_with_grammar()).unwrap();
        let payload = serialize_phrase_index_blob_to_rkyv(&index.to_blob()).unwrap();
        let view = access_phrase_blob(&payload).unwrap();
        (index, view)
    }

    #[test]
    fn archived_entry_lookup_matches_owned_index() {
        let (index, view) = parity_fixture();
        assert_eq!(
            view.get_entry(&first_id()),
            index.get_entry(&first_id()).cloned()
        );
        assert_eq!(view.get_entry(&Ulid::new()), None);
    }

    #[test]
    fn archived_token_lookup_matches_owned_index() {
        let (index, view) = parity_fixture();
        let owned: Vec<IndexEntry> = index
            .get_phrases_by_token("world")
            .into_iter()
            .cloned()
            .collect();
        assert_eq!(view.get_phrases_by_token("world"), owned);
        assert!(view.get_phrases_by_token("missing").is_empty());
    }

    #[test]
    fn archived_all_ids_match_owned_index() {
        let (index, view) = parity_fixture();
        assert_eq!(view.all_ids(), index.all_ids().clone());
    }

    #[test]
    fn archived_iteration_and_metadata_match_owned_index() {
        let (index, view) = parity_fixture();

        // HashMap iteration order is random — sort both sides by id.
        let mut owned: Vec<IndexEntry> = index.iter_entries().cloned().collect();
        owned.sort_by(|a, b| a.id().cmp(b.id()));
        let mut archived: Vec<IndexEntry> = view.iter_entries().collect();
        archived.sort_by(|a, b| a.id().cmp(b.id()));

        assert_eq!(archived, owned);
        assert_eq!(view.len(), index.len());
        assert_eq!(view.version(), (index.version, index.hash.clone()));
    }

    #[test]
    fn access_rejects_truncated_payload() {
        let index = PhraseIndex::from_json(index_json()).unwrap();
        let payload = serialize_phrase_index_blob_to_rkyv(&index.to_blob()).unwrap();
        assert!(access_phrase_blob(&payload[..payload.len() / 2]).is_err());
    }
}
