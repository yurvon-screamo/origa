//! Archived-branch dispatch coverage in a dedicated process.
//!
//! The dictionary mod.rs slots are one-shot per process: inside the lib
//! test binary the phrase slot is owned by the shared fixture and the
//! vocabulary slot by the real-chunks fixture, so the `Archived` arms of
//! the lookup dispatch are unreachable there. This integration target has
//! its own process — each test installs an archived view into its own
//! still-empty slot and pins the dispatch behaviour end to end.

use origa::dictionary::phrase::{
    PhraseIndex, access_phrase_blob, get_all_index_ids, get_chunk_id, get_index_entry,
    get_phrases_by_token, index_version, install_archived_phrase_index, iter_index_entries,
    phrase_index_len, serialize_phrase_index_blob_to_rkyv,
};
use origa::dictionary::pitch_audio::{
    PitchAudioIndex, access_pitch_index_blob, get_audio_entry_count, get_audio_for_reading,
    get_audio_for_word, install_archived_pitch_index, pitch_audio_version,
    serialize_pitch_index_blob_to_rkyv,
};
use origa::dictionary::vocabulary::{
    VocabularyChunkData, access_vocabulary_blob, build_vocabulary_database_from_chunks,
    get_description, get_translation, get_translations, install_archived_vocabulary,
    serialize_vocabulary_blob_to_rkyv, serialize_vocabulary_to_rkyv,
};
use origa::domain::NativeLanguage;
use ulid::Ulid;

fn chunk_data() -> VocabularyChunkData {
    let empty = "{}";
    VocabularyChunkData {
        chunk_01: r#"{"猫": {"russian_translation": "кот", "english_translation": "cat"}}"#
            .to_string(),
        chunk_02: empty.to_string(),
        chunk_03: empty.to_string(),
        chunk_04: empty.to_string(),
        chunk_05: empty.to_string(),
        chunk_06: empty.to_string(),
        chunk_07: empty.to_string(),
        chunk_08: empty.to_string(),
        chunk_09: empty.to_string(),
        chunk_10: empty.to_string(),
        chunk_11: empty.to_string(),
    }
}

fn phrase_index() -> PhraseIndex {
    let json = r#"{"v":1,"h":"archived-test","total":2,"phrases":[
        {"i":"01KPJ5S3N1DRFFD236Z4EZ03HJ","t":["test","hello"],"c":0},
        {"i":"01KPJ5S3N1DRFFD236Z4EZ03HK","t":["test","bye"],"c":1}
    ]}"#;
    PhraseIndex::from_json(json).expect("valid fixture")
}

fn first_fixture_id() -> Ulid {
    Ulid::from_string("01KPJ5S3N1DRFFD236Z4EZ03HJ").expect("valid ULID")
}

#[test]
fn archived_phrase_dispatch_serves_every_lookup() {
    // Arrange — install the archived v2 view into the empty slot.
    let index = phrase_index();
    let payload = serialize_phrase_index_blob_to_rkyv(&index.to_blob()).expect("serialize");
    let view = access_phrase_blob(&payload).expect("access");
    install_archived_phrase_index(view).expect("empty slot accepts the view");

    // Assert — every free function routes through the Archived branch.
    assert_eq!(phrase_index_len(), 2);
    assert_eq!(get_all_index_ids().len(), 2);
    let (version, hash) = index_version();
    assert_eq!(version, 1);
    assert_eq!(hash, "archived-test");

    let by_token = get_phrases_by_token("test");
    assert_eq!(by_token.len(), 2, "both fixture phrases carry 'test'");

    let entry = get_index_entry(&first_fixture_id()).expect("entry by id");
    assert_eq!(entry.chunk_id(), 0);
    assert_eq!(get_chunk_id(&first_fixture_id()), Some(0));
    assert_eq!(iter_index_entries().expect("loaded").count(), 2);
}

#[test]
fn archived_vocabulary_dispatch_serves_lookups_and_refuses_reserialization() {
    // Arrange
    let db = build_vocabulary_database_from_chunks(chunk_data()).expect("valid chunks");
    let payload = serialize_vocabulary_blob_to_rkyv(&db).expect("serialize");
    let view = access_vocabulary_blob(&payload).expect("access");
    install_archived_vocabulary(view).expect("empty slot accepts the view");
    let language = &NativeLanguage::Russian;

    // Assert — lookups route through the Archived branch. The singular
    // lookup renders the bullet-list format ("- кот"), the plural one
    // returns plain translations.
    assert_eq!(get_translation("猫", language), Some("- кот".to_string()));
    assert_eq!(
        get_translations("猫", language),
        Some(vec!["кот".to_string()])
    );
    assert_eq!(get_translation("犬", language), None);
    assert_eq!(get_description("猫", language), None);

    // The archived store is already the CDN-blob form: re-serialization is
    // refused instead of silently round-tripping.
    let Err(origa::domain::OrigaError::VocabularyParseError { reason }) =
        serialize_vocabulary_to_rkyv()
    else {
        panic!("expected the archived-form refusal");
    };
    assert!(
        reason.contains("already the CDN blob form"),
        "reason: {reason}"
    );
}

#[test]
fn archived_pitch_dispatch_serves_the_full_fallback_chain() {
    // Arrange — the v3 composite-key form.
    let index = PitchAudioIndex::from_json(
        r#"{"v":3,"total":5,"entries":{
            "役|やく":{"f":"yaku.opus","p":1},
            "役|えき":{"f":"eki.opus","p":0},
            "やく":{"f":"yaku_kana.opus","p":1},
            "えき":{"f":"eki_kana.opus","p":0},
            "役":{"f":"fallback.opus","p":0}
        }}"#,
    )
    .expect("valid index");
    let payload = serialize_pitch_index_blob_to_rkyv(&index.to_blob()).expect("serialize");
    let view = access_pitch_index_blob(&payload).expect("access");
    install_archived_pitch_index(view).expect("empty slot accepts the view");

    // Assert
    assert_eq!(pitch_audio_version(), 3);
    assert_eq!(get_audio_entry_count(), 5);
    assert_eq!(
        get_audio_for_word("役|やく").expect("composite").file(),
        "yaku.opus"
    );
    assert_eq!(
        get_audio_for_reading("NoSuchWord", "えき")
            .expect("reading fallback")
            .file(),
        "eki_kana.opus"
    );
    assert_eq!(
        get_audio_for_reading("役", "NoSuchReading")
            .expect("word fallback")
            .file(),
        "fallback.opus"
    );
    assert!(get_audio_for_word("NoSuchEntry").is_none());
}
