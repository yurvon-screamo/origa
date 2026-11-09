//! Archived v3 phrase-dispatch coverage in a dedicated process.
//!
//! The phrase slot is one-shot per process and the v2 dispatch test already
//! claims it (`dictionary_archived_dispatch.rs`), so the v3 branch needs
//! its own process to install into an empty slot.

use origa::dictionary::phrase::{
    PhraseIndex, access_phrase_blob_v3, get_all_index_ids, get_chunk_id, get_phrases_by_token,
    index_version, install_archived_phrase_index_v3, phrase_index_len,
    serialize_phrase_index_v3_blob_to_rkyv,
};
use ulid::Ulid;

#[test]
fn archived_phrase_v3_dispatch_serves_every_lookup() {
    // Arrange — the deduplicated v3 blob form.
    let json = r#"{"v":1,"h":"archived-v3","total":2,"phrases":[
        {"i":"01KPJ5S3N1DRFFD236Z4EZ03HJ","t":["test","hello"],"c":0},
        {"i":"01KPJ5S3N1DRFFD236Z4EZ03HK","t":["test","bye"],"c":1}
    ]}"#;
    let index = PhraseIndex::from_json(json).expect("valid fixture");
    let payload = serialize_phrase_index_v3_blob_to_rkyv(&index.to_blob_v3()).expect("serialize");
    let view = access_phrase_blob_v3(&payload).expect("access");
    install_archived_phrase_index_v3(view).expect("empty slot accepts the view");
    let first_id = Ulid::from_string("01KPJ5S3N1DRFFD236Z4EZ03HJ").expect("valid ULID");

    // Assert — every free function routes through the ArchivedV3 branch.
    assert_eq!(phrase_index_len(), 2);
    assert_eq!(get_all_index_ids().len(), 2);
    assert_eq!(index_version().0, 1);
    assert_eq!(get_phrases_by_token("hello").len(), 1);
    assert_eq!(get_chunk_id(&first_id), Some(0));
}
