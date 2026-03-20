mod classify;
mod conjugations;
mod godan_tables;
mod irregulars;
mod te_ta;

#[cfg(test)]
pub use classify::{classify_verb, VerbGroup};
pub use conjugations::{
    to_ba_form, to_causative_form, to_causative_passive_form, to_chau_form, to_imperative_form,
    to_main_view, to_masen_deshita_form, to_masen_form, to_mashita_form, to_mashou_form,
    to_masu_form, to_nai_form, to_nikui_form, to_o_kudasai_form, to_o_ni_narimasu_form,
    to_o_shimasu_form, to_passive_form, to_potential_form, to_sou_form_verb, to_stem_form,
    to_sugiru_form_verb, to_tai_form, to_tara_form, to_teru_form, to_toku_form, to_volitional_form,
    to_yasui_form, to_zu_form,
};
pub use te_ta::{to_ta_form, to_te_form};

#[cfg(test)]
mod tests {
    use super::*;
    use rstest::rstest;

    #[rstest]
    #[case("する", VerbGroup::Irregular)]
    #[case("くる", VerbGroup::Irregular)]
    #[case("食べる", VerbGroup::Ichidan)]
    #[case("見る", VerbGroup::Ichidan)]
    #[case("行く", VerbGroup::Godan)]
    #[case("話す", VerbGroup::Godan)]
    fn classify_verb(#[case] input: &str, #[case] expected: VerbGroup) {
        assert_eq!(super::classify_verb(input), expected);
    }

    #[rstest]
    #[case("行く", "行って")]
    #[case("話す", "話して")]
    #[case("読む", "読んで")]
    #[case("書く", "書いて")]
    #[case("泳ぐ", "泳いで")]
    #[case("食べる", "食べて")]
    #[case("見る", "見て")]
    #[case("する", "して")]
    #[case("くる", "きて")]
    fn te_form(#[case] input: &str, #[case] expected: &str) {
        assert_eq!(to_te_form(input), expected);
    }

    #[rstest]
    #[case("行く", "行きます")]
    #[case("食べる", "食べます")]
    #[case("する", "します")]
    fn masu_form(#[case] input: &str, #[case] expected: &str) {
        assert_eq!(to_masu_form(input), expected);
    }

    #[rstest]
    #[case("行く", "行った")]
    #[case("話す", "話した")]
    #[case("読む", "読んだ")]
    #[case("書く", "書いた")]
    #[case("食べる", "食べた")]
    #[case("する", "した")]
    fn ta_form(#[case] input: &str, #[case] expected: &str) {
        assert_eq!(to_ta_form(input), expected);
    }

    #[rstest]
    #[case("行く", "行かない")]
    #[case("食べる", "食べない")]
    #[case("する", "しない")]
    #[case("くる", "こない")]
    fn nai_form(#[case] input: &str, #[case] expected: &str) {
        assert_eq!(to_nai_form(input), expected);
    }

    #[rstest]
    #[case("行く", "行ったら")]
    #[case("食べる", "食べたら")]
    #[case("する", "したら")]
    fn tara_form(#[case] input: &str, #[case] expected: &str) {
        assert_eq!(to_tara_form(input), expected);
    }

    #[rstest]
    #[case("行く", "行けば")]
    #[case("食べる", "食べれば")]
    #[case("する", "すれば")]
    fn ba_form(#[case] input: &str, #[case] expected: &str) {
        assert_eq!(to_ba_form(input), expected);
    }

    #[rstest]
    #[case("行く", "行ける")]
    #[case("食べる", "食べられる")]
    #[case("する", "できる")]
    fn potential_form(#[case] input: &str, #[case] expected: &str) {
        assert_eq!(to_potential_form(input), expected);
    }

    #[rstest]
    #[case("行く", "行かれる")]
    #[case("食べる", "食べられる")]
    #[case("する", "される")]
    fn passive_form(#[case] input: &str, #[case] expected: &str) {
        assert_eq!(to_passive_form(input), expected);
    }

    #[rstest]
    #[case("行く", "行かせる")]
    #[case("食べる", "食べさせる")]
    #[case("する", "させる")]
    fn causative_form(#[case] input: &str, #[case] expected: &str) {
        assert_eq!(to_causative_form(input), expected);
    }

    #[rstest]
    #[case("行く", "行け")]
    #[case("食べる", "食べろ")]
    #[case("する", "しろ")]
    fn imperative_form(#[case] input: &str, #[case] expected: &str) {
        assert_eq!(to_imperative_form(input), expected);
    }

    #[rstest]
    #[case("行く", "行こう")]
    #[case("食べる", "食べよう")]
    #[case("する", "しよう")]
    fn volitional_form(#[case] input: &str, #[case] expected: &str) {
        assert_eq!(to_volitional_form(input), expected);
    }

    #[rstest]
    #[case("行く", "行かず")]
    #[case("食べる", "食べず")]
    #[case("する", "せず")]
    fn zu_form(#[case] input: &str, #[case] expected: &str) {
        assert_eq!(to_zu_form(input), expected);
    }

    #[rstest]
    #[case("要る", VerbGroup::Godan)]
    #[case("入る", VerbGroup::Godan)]
    #[case("減る", VerbGroup::Godan)]
    #[case("茂る", VerbGroup::Godan)]
    #[case("喋る", VerbGroup::Godan)]
    #[case("遮る", VerbGroup::Godan)]
    #[case("悟る", VerbGroup::Godan)]
    fn godan_iru_eru_exceptions(#[case] input: &str, #[case] expected: VerbGroup) {
        assert_eq!(super::classify_verb(input), expected);
    }

    #[rstest]
    #[case("見る", VerbGroup::Ichidan)]
    #[case("居る", VerbGroup::Ichidan)]
    #[case("着る", VerbGroup::Ichidan)]
    #[case("寝る", VerbGroup::Ichidan)]
    #[case("経る", VerbGroup::Ichidan)]
    #[case("蹴る", VerbGroup::Ichidan)]
    fn ichidan_short_verbs(#[case] input: &str, #[case] expected: VerbGroup) {
        assert_eq!(super::classify_verb(input), expected);
    }

    #[rstest]
    #[case("要る", "要って", "要ります")]
    #[case("入る", "入って", "入ります")]
    #[case("喋る", "喋って", "喋ります")]
    fn godan_exceptions_conjugation(#[case] verb: &str, #[case] te: &str, #[case] masu: &str) {
        assert_eq!(to_te_form(verb), te);
        assert_eq!(to_masu_form(verb), masu);
    }
}
