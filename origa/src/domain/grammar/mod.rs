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
use crate::domain::{OrigaError, PartOfSpeech, grammar::forms_adjective::adjective_remove_postfix};

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
