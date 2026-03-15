pub(crate) mod forms_adjective;
pub(crate) mod forms_verb;

use crate::dictionary::grammar::{FormatAction, GrammarRule};
use crate::domain::grammar::forms_adjective::{
    to_de_form, to_garu_form, to_katta_form, to_kereba_form, to_ku_form, to_kunai_form,
    to_kunakatta_form, to_kute_form, to_na_form, to_nara_form, to_nasasou_form, to_sou_form_iadj,
    to_sou_form_naadj, to_sugiru_form,
};
use crate::domain::grammar::forms_verb::{
    to_ba_form, to_causative_form, to_causative_passive_form, to_chau_form, to_imperative_form,
    to_main_view, to_masen_deshita_form, to_masen_form, to_mashita_form, to_mashou_form,
    to_masu_form, to_nai_form, to_nikui_form, to_o_kudasai_form, to_o_ni_narimasu_form,
    to_o_shimasu_form, to_passive_form, to_potential_form, to_sou_form_verb, to_stem_form,
    to_sugiru_form_verb, to_ta_form, to_tai_form, to_tara_form, to_te_form, to_teru_form,
    to_toku_form, to_volitional_form, to_yasui_form, to_zu_form,
};
use crate::domain::{grammar::forms_adjective::adjective_remove_postfix, OrigaError, PartOfSpeech};

impl GrammarRule {
    pub fn format(
        &self,
        source_word: &str,
        part_of_speech: &PartOfSpeech,
    ) -> Result<String, OrigaError> {
        let format_map = self.format_map().ok_or(OrigaError::GrammarFormatError {
            reason: "Rule has no format_map".to_string(),
        })?;

        let rules = format_map
            .get(part_of_speech)
            .ok_or(OrigaError::GrammarFormatError {
                reason: "Not supported part of speech".to_string(),
            })?;

        let result = rules.iter().try_fold(
            source_word.to_string(),
            |word, rule| -> Result<String, OrigaError> {
                match rule {
                    FormatAction::AdjectiveRemovePostfix {} => {
                        adjective_remove_postfix(&word, part_of_speech)
                    }
                    FormatAction::AdjectiveToKunai {} => to_kunai_form(&word, part_of_speech),
                    FormatAction::AdjectiveToKatta {} => to_katta_form(&word, part_of_speech),
                    FormatAction::AdjectiveToKunakatta {} => {
                        to_kunakatta_form(&word, part_of_speech)
                    }
                    FormatAction::AdjectiveToKute {} => to_kute_form(&word, part_of_speech),
                    FormatAction::AdjectiveToKu {} => to_ku_form(&word, part_of_speech),
                    FormatAction::AdjectiveToKereba {} => to_kereba_form(&word, part_of_speech),
                    FormatAction::AdjectiveToSou {} => to_sou_form_iadj(&word, part_of_speech),
                    FormatAction::AdjectiveToSugiru {} => to_sugiru_form(&word, part_of_speech),
                    FormatAction::AdjectiveToNa {} => to_na_form(&word, part_of_speech),
                    FormatAction::AdjectiveToDe {} => to_de_form(&word, part_of_speech),
                    FormatAction::AdjectiveToNara {} => to_nara_form(&word, part_of_speech),
                    FormatAction::AdjectiveToSouNa {} => to_sou_form_naadj(&word, part_of_speech),
                    FormatAction::AdjectiveToNasasou {} => to_nasasou_form(&word, part_of_speech),
                    FormatAction::AdjectiveToGaru {} => to_garu_form(&word, part_of_speech),

                    FormatAction::VerbToTeForm {} => Ok(to_te_form(&word)),
                    FormatAction::VerbToMainView {} => Ok(to_main_view(&word)),
                    FormatAction::VerbToMasu {} => Ok(to_masu_form(&word)),
                    FormatAction::VerbToMasen {} => Ok(to_masen_form(&word)),
                    FormatAction::VerbToMashita {} => Ok(to_mashita_form(&word)),
                    FormatAction::VerbToMasenDeshita {} => Ok(to_masen_deshita_form(&word)),
                    FormatAction::VerbToMashou {} => Ok(to_mashou_form(&word)),
                    FormatAction::VerbToStem {} => Ok(to_stem_form(&word)),
                    FormatAction::VerbToTa {} => Ok(to_ta_form(&word)),
                    FormatAction::VerbToNai {} => Ok(to_nai_form(&word)),
                    FormatAction::VerbToTara {} => Ok(to_tara_form(&word)),
                    FormatAction::VerbToBa {} => Ok(to_ba_form(&word)),
                    FormatAction::VerbToPotential {} => Ok(to_potential_form(&word)),
                    FormatAction::VerbToPassive {} => Ok(to_passive_form(&word)),
                    FormatAction::VerbToCausative {} => Ok(to_causative_form(&word)),
                    FormatAction::VerbToCausativePassive {} => Ok(to_causative_passive_form(&word)),
                    FormatAction::VerbToImperative {} => Ok(to_imperative_form(&word)),
                    FormatAction::VerbToVolitional {} => Ok(to_volitional_form(&word)),
                    FormatAction::VerbToSou {} => Ok(to_sou_form_verb(&word)),
                    FormatAction::VerbToZu {} => Ok(to_zu_form(&word)),
                    FormatAction::VerbToTai {} => Ok(to_tai_form(&word)),
                    FormatAction::VerbToYasui {} => Ok(to_yasui_form(&word)),
                    FormatAction::VerbToNikui {} => Ok(to_nikui_form(&word)),
                    FormatAction::VerbToSugiru {} => Ok(to_sugiru_form_verb(&word)),
                    FormatAction::VerbToChau {} => Ok(to_chau_form(&word)),
                    FormatAction::VerbToToku {} => Ok(to_toku_form(&word)),
                    FormatAction::VerbToTeru {} => Ok(to_teru_form(&word)),
                    FormatAction::VerbToONinarimasu {} => Ok(to_o_ni_narimasu_form(&word)),
                    FormatAction::VerbToOKudasai {} => Ok(to_o_kudasai_form(&word)),
                    FormatAction::VerbToOShimasu {} => Ok(to_o_shimasu_form(&word)),

                    FormatAction::AddPostfix { postfix } => Ok(word + postfix),
                    FormatAction::ReplacePostfix {
                        old_postfix,
                        new_postfix,
                    } => Ok(word.trim_end_matches(old_postfix).to_string() + new_postfix),
                    FormatAction::RemovePostfix { postfix } => {
                        Ok(word.trim_end_matches(postfix).to_string())
                    }
                }
            },
        )?;

        Ok(result)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::dictionary::grammar::{FormatAction, GrammarRule, GrammarRuleContent};
    use crate::domain::{JapaneseLevel, NativeLanguage, PartOfSpeech};
    use std::collections::HashMap;
    use ulid::Ulid;

    fn create_rule_with_format_map(
        format_map: HashMap<PartOfSpeech, Vec<FormatAction>>,
    ) -> GrammarRule {
        GrammarRule::new(
            Ulid::new(),
            JapaneseLevel::N5,
            HashMap::from([(
                NativeLanguage::English,
                GrammarRuleContent::new(
                    "Test Rule".to_string(),
                    "Test description".to_string(),
                    "# Test".to_string(),
                ),
            )]),
            Some(format_map),
        )
    }

    mod format_errors {
        use super::*;

        #[test]
        fn returns_error_when_format_map_is_none() {
            let rule = GrammarRule::new(Ulid::new(), JapaneseLevel::N5, HashMap::new(), None);

            let result = rule.format("高い", &PartOfSpeech::IAdjective);

            assert!(result.is_err());
            assert!(matches!(
                result.unwrap_err(),
                OrigaError::GrammarFormatError { .. }
            ));
        }

        #[test]
        fn returns_error_when_part_of_speech_not_supported() {
            let format_map = HashMap::from([(PartOfSpeech::Verb, vec![])]);
            let rule = create_rule_with_format_map(format_map);

            let result = rule.format("高い", &PartOfSpeech::IAdjective);

            assert!(result.is_err());
            assert!(matches!(
                result.unwrap_err(),
                OrigaError::GrammarFormatError { .. }
            ));
        }
    }

    mod i_adjective_forms {
        use super::*;

        #[test]
        fn removes_postfix() {
            let format_map = HashMap::from([(
                PartOfSpeech::IAdjective,
                vec![FormatAction::AdjectiveRemovePostfix {}],
            )]);
            let rule = create_rule_with_format_map(format_map);

            let result = rule.format("高い", &PartOfSpeech::IAdjective);

            assert_eq!(result.unwrap(), "高くなる");
        }

        #[test]
        fn converts_to_kunai() {
            let format_map = HashMap::from([(
                PartOfSpeech::IAdjective,
                vec![FormatAction::AdjectiveToKunai {}],
            )]);
            let rule = create_rule_with_format_map(format_map);

            let result = rule.format("高い", &PartOfSpeech::IAdjective);

            assert_eq!(result.unwrap(), "高くない");
        }

        #[test]
        fn converts_to_katta() {
            let format_map = HashMap::from([(
                PartOfSpeech::IAdjective,
                vec![FormatAction::AdjectiveToKatta {}],
            )]);
            let rule = create_rule_with_format_map(format_map);

            let result = rule.format("高い", &PartOfSpeech::IAdjective);

            assert_eq!(result.unwrap(), "高かった");
        }

        #[test]
        fn converts_to_kunakatta() {
            let format_map = HashMap::from([(
                PartOfSpeech::IAdjective,
                vec![FormatAction::AdjectiveToKunakatta {}],
            )]);
            let rule = create_rule_with_format_map(format_map);

            let result = rule.format("高い", &PartOfSpeech::IAdjective);

            assert_eq!(result.unwrap(), "高くなかった");
        }

        #[test]
        fn converts_to_kute() {
            let format_map = HashMap::from([(
                PartOfSpeech::IAdjective,
                vec![FormatAction::AdjectiveToKute {}],
            )]);
            let rule = create_rule_with_format_map(format_map);

            let result = rule.format("高い", &PartOfSpeech::IAdjective);

            assert_eq!(result.unwrap(), "高くて");
        }

        #[test]
        fn converts_to_ku() {
            let format_map = HashMap::from([(
                PartOfSpeech::IAdjective,
                vec![FormatAction::AdjectiveToKu {}],
            )]);
            let rule = create_rule_with_format_map(format_map);

            let result = rule.format("高い", &PartOfSpeech::IAdjective);

            assert_eq!(result.unwrap(), "高く");
        }

        #[test]
        fn converts_to_kereba() {
            let format_map = HashMap::from([(
                PartOfSpeech::IAdjective,
                vec![FormatAction::AdjectiveToKereba {}],
            )]);
            let rule = create_rule_with_format_map(format_map);

            let result = rule.format("高い", &PartOfSpeech::IAdjective);

            assert_eq!(result.unwrap(), "高ければ");
        }

        #[test]
        fn converts_to_sou() {
            let format_map = HashMap::from([(
                PartOfSpeech::IAdjective,
                vec![FormatAction::AdjectiveToSou {}],
            )]);
            let rule = create_rule_with_format_map(format_map);

            let result = rule.format("高い", &PartOfSpeech::IAdjective);

            assert_eq!(result.unwrap(), "高そう");
        }

        #[test]
        fn converts_to_sugiru() {
            let format_map = HashMap::from([(
                PartOfSpeech::IAdjective,
                vec![FormatAction::AdjectiveToSugiru {}],
            )]);
            let rule = create_rule_with_format_map(format_map);

            let result = rule.format("高い", &PartOfSpeech::IAdjective);

            assert_eq!(result.unwrap(), "高すぎる");
        }

        #[test]
        fn converts_to_nasasou() {
            let format_map = HashMap::from([(
                PartOfSpeech::IAdjective,
                vec![FormatAction::AdjectiveToNasasou {}],
            )]);
            let rule = create_rule_with_format_map(format_map);

            let result = rule.format("高い", &PartOfSpeech::IAdjective);

            assert_eq!(result.unwrap(), "高なさそう");
        }

        #[test]
        fn converts_to_garu() {
            let format_map = HashMap::from([(
                PartOfSpeech::IAdjective,
                vec![FormatAction::AdjectiveToGaru {}],
            )]);
            let rule = create_rule_with_format_map(format_map);

            let result = rule.format("高い", &PartOfSpeech::IAdjective);

            assert_eq!(result.unwrap(), "高がる");
        }
    }

    mod na_adjective_forms {
        use super::*;

        #[test]
        fn removes_postfix() {
            let format_map = HashMap::from([(
                PartOfSpeech::NaAdjective,
                vec![FormatAction::AdjectiveRemovePostfix {}],
            )]);
            let rule = create_rule_with_format_map(format_map);

            let result = rule.format("静かな", &PartOfSpeech::NaAdjective);

            assert_eq!(result.unwrap(), "静かになる");
        }

        #[test]
        fn converts_to_na() {
            let format_map = HashMap::from([(
                PartOfSpeech::NaAdjective,
                vec![FormatAction::AdjectiveToNa {}],
            )]);
            let rule = create_rule_with_format_map(format_map);

            let result = rule.format("静かな", &PartOfSpeech::NaAdjective);

            assert_eq!(result.unwrap(), "静かな");
        }

        #[test]
        fn converts_to_de() {
            let format_map = HashMap::from([(
                PartOfSpeech::NaAdjective,
                vec![FormatAction::AdjectiveToDe {}],
            )]);
            let rule = create_rule_with_format_map(format_map);

            let result = rule.format("静かな", &PartOfSpeech::NaAdjective);

            assert_eq!(result.unwrap(), "静かで");
        }

        #[test]
        fn converts_to_nara() {
            let format_map = HashMap::from([(
                PartOfSpeech::NaAdjective,
                vec![FormatAction::AdjectiveToNara {}],
            )]);
            let rule = create_rule_with_format_map(format_map);

            let result = rule.format("静かな", &PartOfSpeech::NaAdjective);

            assert_eq!(result.unwrap(), "静かなら");
        }

        #[test]
        fn converts_to_sou_na() {
            let format_map = HashMap::from([(
                PartOfSpeech::NaAdjective,
                vec![FormatAction::AdjectiveToSouNa {}],
            )]);
            let rule = create_rule_with_format_map(format_map);

            let result = rule.format("静かな", &PartOfSpeech::NaAdjective);

            assert_eq!(result.unwrap(), "静かそう");
        }

        #[test]
        fn converts_to_nasasou() {
            let format_map = HashMap::from([(
                PartOfSpeech::NaAdjective,
                vec![FormatAction::AdjectiveToNasasou {}],
            )]);
            let rule = create_rule_with_format_map(format_map);

            let result = rule.format("静かな", &PartOfSpeech::NaAdjective);

            assert_eq!(result.unwrap(), "静かじゃなさそう");
        }

        #[test]
        fn converts_to_garu() {
            let format_map = HashMap::from([(
                PartOfSpeech::NaAdjective,
                vec![FormatAction::AdjectiveToGaru {}],
            )]);
            let rule = create_rule_with_format_map(format_map);

            let result = rule.format("静かな", &PartOfSpeech::NaAdjective);

            assert_eq!(result.unwrap(), "静かがる");
        }
    }

    mod verb_forms {
        use super::*;

        #[test]
        fn converts_to_te_form() {
            let format_map =
                HashMap::from([(PartOfSpeech::Verb, vec![FormatAction::VerbToTeForm {}])]);
            let rule = create_rule_with_format_map(format_map);

            let result = rule.format("行く", &PartOfSpeech::Verb);

            assert_eq!(result.unwrap(), "行って");
        }

        #[test]
        fn converts_to_main_view() {
            let format_map =
                HashMap::from([(PartOfSpeech::Verb, vec![FormatAction::VerbToMainView {}])]);
            let rule = create_rule_with_format_map(format_map);

            let result = rule.format("行く", &PartOfSpeech::Verb);

            assert_eq!(result.unwrap(), "行き");
        }

        #[test]
        fn converts_to_masu() {
            let format_map =
                HashMap::from([(PartOfSpeech::Verb, vec![FormatAction::VerbToMasu {}])]);
            let rule = create_rule_with_format_map(format_map);

            let result = rule.format("行く", &PartOfSpeech::Verb);

            assert_eq!(result.unwrap(), "行きます");
        }

        #[test]
        fn converts_to_masen() {
            let format_map =
                HashMap::from([(PartOfSpeech::Verb, vec![FormatAction::VerbToMasen {}])]);
            let rule = create_rule_with_format_map(format_map);

            let result = rule.format("行く", &PartOfSpeech::Verb);

            assert_eq!(result.unwrap(), "行きません");
        }

        #[test]
        fn converts_to_mashita() {
            let format_map =
                HashMap::from([(PartOfSpeech::Verb, vec![FormatAction::VerbToMashita {}])]);
            let rule = create_rule_with_format_map(format_map);

            let result = rule.format("行く", &PartOfSpeech::Verb);

            assert_eq!(result.unwrap(), "行きました");
        }

        #[test]
        fn converts_to_masen_deshita() {
            let format_map = HashMap::from([(
                PartOfSpeech::Verb,
                vec![FormatAction::VerbToMasenDeshita {}],
            )]);
            let rule = create_rule_with_format_map(format_map);

            let result = rule.format("行く", &PartOfSpeech::Verb);

            assert_eq!(result.unwrap(), "行きませんでした");
        }

        #[test]
        fn converts_to_mashou() {
            let format_map =
                HashMap::from([(PartOfSpeech::Verb, vec![FormatAction::VerbToMashou {}])]);
            let rule = create_rule_with_format_map(format_map);

            let result = rule.format("行く", &PartOfSpeech::Verb);

            assert_eq!(result.unwrap(), "行きましょう");
        }

        #[test]
        fn converts_to_stem() {
            let format_map =
                HashMap::from([(PartOfSpeech::Verb, vec![FormatAction::VerbToStem {}])]);
            let rule = create_rule_with_format_map(format_map);

            let result = rule.format("行く", &PartOfSpeech::Verb);

            assert_eq!(result.unwrap(), "行き");
        }

        #[test]
        fn converts_to_ta() {
            let format_map = HashMap::from([(PartOfSpeech::Verb, vec![FormatAction::VerbToTa {}])]);
            let rule = create_rule_with_format_map(format_map);

            let result = rule.format("行く", &PartOfSpeech::Verb);

            assert_eq!(result.unwrap(), "行った");
        }

        #[test]
        fn converts_to_nai() {
            let format_map =
                HashMap::from([(PartOfSpeech::Verb, vec![FormatAction::VerbToNai {}])]);
            let rule = create_rule_with_format_map(format_map);

            let result = rule.format("行く", &PartOfSpeech::Verb);

            assert_eq!(result.unwrap(), "行かない");
        }

        #[test]
        fn converts_to_tara() {
            let format_map =
                HashMap::from([(PartOfSpeech::Verb, vec![FormatAction::VerbToTara {}])]);
            let rule = create_rule_with_format_map(format_map);

            let result = rule.format("行く", &PartOfSpeech::Verb);

            assert_eq!(result.unwrap(), "行ったら");
        }

        #[test]
        fn converts_to_ba() {
            let format_map = HashMap::from([(PartOfSpeech::Verb, vec![FormatAction::VerbToBa {}])]);
            let rule = create_rule_with_format_map(format_map);

            let result = rule.format("行く", &PartOfSpeech::Verb);

            assert_eq!(result.unwrap(), "行けば");
        }

        #[test]
        fn converts_to_potential() {
            let format_map =
                HashMap::from([(PartOfSpeech::Verb, vec![FormatAction::VerbToPotential {}])]);
            let rule = create_rule_with_format_map(format_map);

            let result = rule.format("行く", &PartOfSpeech::Verb);

            assert_eq!(result.unwrap(), "行ける");
        }

        #[test]
        fn converts_to_passive() {
            let format_map =
                HashMap::from([(PartOfSpeech::Verb, vec![FormatAction::VerbToPassive {}])]);
            let rule = create_rule_with_format_map(format_map);

            let result = rule.format("行く", &PartOfSpeech::Verb);

            assert_eq!(result.unwrap(), "行かれる");
        }

        #[test]
        fn converts_to_causative() {
            let format_map =
                HashMap::from([(PartOfSpeech::Verb, vec![FormatAction::VerbToCausative {}])]);
            let rule = create_rule_with_format_map(format_map);

            let result = rule.format("行く", &PartOfSpeech::Verb);

            assert_eq!(result.unwrap(), "行かせる");
        }

        #[test]
        fn converts_to_causative_passive() {
            let format_map = HashMap::from([(
                PartOfSpeech::Verb,
                vec![FormatAction::VerbToCausativePassive {}],
            )]);
            let rule = create_rule_with_format_map(format_map);

            let result = rule.format("行く", &PartOfSpeech::Verb);

            assert_eq!(result.unwrap(), "行かされる");
        }

        #[test]
        fn converts_to_imperative() {
            let format_map =
                HashMap::from([(PartOfSpeech::Verb, vec![FormatAction::VerbToImperative {}])]);
            let rule = create_rule_with_format_map(format_map);

            let result = rule.format("行く", &PartOfSpeech::Verb);

            assert_eq!(result.unwrap(), "行け");
        }

        #[test]
        fn converts_to_volitional() {
            let format_map =
                HashMap::from([(PartOfSpeech::Verb, vec![FormatAction::VerbToVolitional {}])]);
            let rule = create_rule_with_format_map(format_map);

            let result = rule.format("行く", &PartOfSpeech::Verb);

            assert_eq!(result.unwrap(), "行こう");
        }

        #[test]
        fn converts_to_sou() {
            let format_map =
                HashMap::from([(PartOfSpeech::Verb, vec![FormatAction::VerbToSou {}])]);
            let rule = create_rule_with_format_map(format_map);

            let result = rule.format("行く", &PartOfSpeech::Verb);

            assert_eq!(result.unwrap(), "行きそう");
        }

        #[test]
        fn converts_to_zu() {
            let format_map = HashMap::from([(PartOfSpeech::Verb, vec![FormatAction::VerbToZu {}])]);
            let rule = create_rule_with_format_map(format_map);

            let result = rule.format("行く", &PartOfSpeech::Verb);

            assert_eq!(result.unwrap(), "行かず");
        }

        #[test]
        fn converts_to_tai() {
            let format_map =
                HashMap::from([(PartOfSpeech::Verb, vec![FormatAction::VerbToTai {}])]);
            let rule = create_rule_with_format_map(format_map);

            let result = rule.format("行く", &PartOfSpeech::Verb);

            assert_eq!(result.unwrap(), "行きたい");
        }

        #[test]
        fn converts_to_yasui() {
            let format_map =
                HashMap::from([(PartOfSpeech::Verb, vec![FormatAction::VerbToYasui {}])]);
            let rule = create_rule_with_format_map(format_map);

            let result = rule.format("行く", &PartOfSpeech::Verb);

            assert_eq!(result.unwrap(), "行きやすい");
        }

        #[test]
        fn converts_to_nikui() {
            let format_map =
                HashMap::from([(PartOfSpeech::Verb, vec![FormatAction::VerbToNikui {}])]);
            let rule = create_rule_with_format_map(format_map);

            let result = rule.format("行く", &PartOfSpeech::Verb);

            assert_eq!(result.unwrap(), "行きにくい");
        }

        #[test]
        fn converts_to_sugiru() {
            let format_map =
                HashMap::from([(PartOfSpeech::Verb, vec![FormatAction::VerbToSugiru {}])]);
            let rule = create_rule_with_format_map(format_map);

            let result = rule.format("行く", &PartOfSpeech::Verb);

            assert_eq!(result.unwrap(), "行きすぎる");
        }

        #[test]
        fn converts_to_chau() {
            let format_map =
                HashMap::from([(PartOfSpeech::Verb, vec![FormatAction::VerbToChau {}])]);
            let rule = create_rule_with_format_map(format_map);

            let result = rule.format("行く", &PartOfSpeech::Verb);

            assert_eq!(result.unwrap(), "行っちゃう");
        }

        #[test]
        fn converts_to_toku() {
            let format_map =
                HashMap::from([(PartOfSpeech::Verb, vec![FormatAction::VerbToToku {}])]);
            let rule = create_rule_with_format_map(format_map);

            let result = rule.format("行く", &PartOfSpeech::Verb);

            assert_eq!(result.unwrap(), "行っとく");
        }

        #[test]
        fn converts_to_teru() {
            let format_map =
                HashMap::from([(PartOfSpeech::Verb, vec![FormatAction::VerbToTeru {}])]);
            let rule = create_rule_with_format_map(format_map);

            let result = rule.format("行く", &PartOfSpeech::Verb);

            assert_eq!(result.unwrap(), "行ってる");
        }

        #[test]
        fn converts_to_o_ni_narimasu() {
            let format_map =
                HashMap::from([(PartOfSpeech::Verb, vec![FormatAction::VerbToONinarimasu {}])]);
            let rule = create_rule_with_format_map(format_map);

            let result = rule.format("行く", &PartOfSpeech::Verb);

            assert_eq!(result.unwrap(), "お行きになる");
        }

        #[test]
        fn converts_to_o_kudasai() {
            let format_map =
                HashMap::from([(PartOfSpeech::Verb, vec![FormatAction::VerbToOKudasai {}])]);
            let rule = create_rule_with_format_map(format_map);

            let result = rule.format("行く", &PartOfSpeech::Verb);

            assert_eq!(result.unwrap(), "お行きください");
        }

        #[test]
        fn converts_to_o_shimasu() {
            let format_map =
                HashMap::from([(PartOfSpeech::Verb, vec![FormatAction::VerbToOShimasu {}])]);
            let rule = create_rule_with_format_map(format_map);

            let result = rule.format("行く", &PartOfSpeech::Verb);

            assert_eq!(result.unwrap(), "お行きする");
        }

        #[test]
        fn irregular_verb_suru_conversions() {
            let format_map =
                HashMap::from([(PartOfSpeech::Verb, vec![FormatAction::VerbToTeForm {}])]);
            let rule = create_rule_with_format_map(format_map);

            let result = rule.format("する", &PartOfSpeech::Verb);

            assert_eq!(result.unwrap(), "して");
        }

        #[test]
        fn irregular_verb_kuru_conversions() {
            let format_map =
                HashMap::from([(PartOfSpeech::Verb, vec![FormatAction::VerbToTeForm {}])]);
            let rule = create_rule_with_format_map(format_map);

            let result = rule.format("くる", &PartOfSpeech::Verb);

            assert_eq!(result.unwrap(), "きて");
        }
    }

    mod postfix_operations {
        use super::*;

        #[test]
        fn adds_postfix() {
            let format_map = HashMap::from([(
                PartOfSpeech::Verb,
                vec![FormatAction::AddPostfix {
                    postfix: "たい".to_string(),
                }],
            )]);
            let rule = create_rule_with_format_map(format_map);

            let result = rule.format("行き", &PartOfSpeech::Verb);

            assert_eq!(result.unwrap(), "行きたい");
        }

        #[test]
        fn replaces_postfix() {
            let format_map = HashMap::from([(
                PartOfSpeech::Verb,
                vec![FormatAction::ReplacePostfix {
                    old_postfix: "く".to_string(),
                    new_postfix: "いて".to_string(),
                }],
            )]);
            let rule = create_rule_with_format_map(format_map);

            let result = rule.format("行く", &PartOfSpeech::Verb);

            assert_eq!(result.unwrap(), "行いて");
        }

        #[test]
        fn removes_postfix() {
            let format_map = HashMap::from([(
                PartOfSpeech::Verb,
                vec![FormatAction::RemovePostfix {
                    postfix: "く".to_string(),
                }],
            )]);
            let rule = create_rule_with_format_map(format_map);

            let result = rule.format("行く", &PartOfSpeech::Verb);

            assert_eq!(result.unwrap(), "行");
        }
    }

    mod chained_operations {
        use super::*;

        #[test]
        fn chains_multiple_format_actions() {
            let format_map = HashMap::from([(
                PartOfSpeech::IAdjective,
                vec![
                    FormatAction::AdjectiveRemovePostfix {},
                    FormatAction::AddPostfix {
                        postfix: "です".to_string(),
                    },
                ],
            )]);
            let rule = create_rule_with_format_map(format_map);

            let result = rule.format("高い", &PartOfSpeech::IAdjective);

            assert_eq!(result.unwrap(), "高くなるです");
        }
    }
}
