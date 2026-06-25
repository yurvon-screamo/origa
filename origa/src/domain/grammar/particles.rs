use std::collections::HashSet;
use std::sync::OnceLock;

/// Closed set of Japanese grammatical particles (JLPT N5-N4 plus basic N3).
/// A token listed here never blocks tail-phrase eligibility: particles attach
/// to words rather than carrying standalone lexical meaning, so a learner who
/// has mastered the anchor vocabulary already understands the phrase.
///
/// Coverage is intentionally limited to N5-N4 + the common binding particles of
/// N3. Rarer N3-N1 connective particles (ものの, どころか, かぎり, …) appear
/// infrequently in the phrase index and would balloon the whitelist for marginal
/// eligibility gain; they are left for a future expansion when the index
/// surfaces real demand.
///
/// Order follows the conventional Japanese grammar split:
///   * case / binding particles (が, を, に, …)
///   * limiting particles (だけ, しか, ばかり, …)
///   * sentence-final / discourse particles (か, よ, ね, …)
pub(crate) const GRAMMATICAL_PARTICLES: &[&str] = &[
    // case / binding
    "が",
    "を",
    "に",
    "で",
    "と",
    "から",
    "より",
    "へ",
    "や",
    "の",
    "は",
    "も",
    // limiting / restrictive
    "こそ",
    "さえ",
    "すら",
    "でも",
    "しか",
    "だけ",
    "のみ",
    "ばかり",
    "まで",
    // discourse / sentence-final
    "など",
    "なんて",
    "って",
    "ほど",
    "くらい",
    "ぐらい",
    "か",
    "よ",
    "ね",
    "な",
    "わ",
    "ぜ",
    "ぞ",
    "さ",
    "え",
];

static PARTICLE_SET: OnceLock<HashSet<&'static str>> = OnceLock::new();

fn particle_lookup() -> &'static HashSet<&'static str> {
    PARTICLE_SET.get_or_init(|| GRAMMATICAL_PARTICLES.iter().copied().collect())
}

/// O(1) check whether `token` is a Japanese grammatical particle.
pub(crate) fn is_grammatical_particle(token: &str) -> bool {
    particle_lookup().contains(token)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn recognizes_case_particles() {
        for p in [
            "が", "を", "に", "で", "と", "から", "より", "へ", "や", "の", "は", "も",
        ] {
            assert!(is_grammatical_particle(p), "{p} should be a particle");
        }
    }

    #[test]
    fn recognizes_limiting_particles() {
        for p in [
            "こそ",
            "さえ",
            "すら",
            "でも",
            "しか",
            "だけ",
            "のみ",
            "ばかり",
            "まで",
        ] {
            assert!(is_grammatical_particle(p), "{p} should be a particle");
        }
    }

    #[test]
    fn recognizes_sentence_final_particles() {
        for p in [
            "など",
            "なんて",
            "って",
            "ほど",
            "くらい",
            "ぐらい",
            "か",
            "よ",
            "ね",
            "な",
            "わ",
            "ぜ",
            "ぞ",
            "さ",
            "え",
        ] {
            assert!(is_grammatical_particle(p), "{p} should be a particle");
        }
    }

    #[test]
    fn rejects_verbs_and_nouns() {
        assert!(
            !is_grammatical_particle("為る"),
            "為る is a verb, not a particle"
        );
        assert!(
            !is_grammatical_particle("無い"),
            "無い is an adjective, not a particle"
        );
        assert!(
            !is_grammatical_particle("猫"),
            "猫 is a noun, not a particle"
        );
        assert!(!is_grammatical_particle("test"), "test is not a particle");
        assert!(
            !is_grammatical_particle(""),
            "empty string is not a particle"
        );
    }

    #[test]
    fn lookup_is_idempotent() {
        let first = particle_lookup();
        let second = particle_lookup();
        assert!(first.contains("は"));
        assert_eq!(first.len(), GRAMMATICAL_PARTICLES.len());
        assert!(
            std::ptr::eq(first, second),
            "the OnceLock must return the same HashSet reference on repeated calls"
        );
    }
}
