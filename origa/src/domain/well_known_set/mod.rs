use std::collections::HashMap;

use serde::{Deserialize, Serialize};

use crate::domain::{
    OrigaError,
    value_objects::{JapaneseLevel, NativeLanguage},
};

const JLPT_N1_RAW: &str = include_str!("./jltp_n1.json");
const JLPT_N2_RAW: &str = include_str!("./jltp_n2.json");
const JLPT_N3_RAW: &str = include_str!("./jltp_n3.json");
const JLPT_N4_RAW: &str = include_str!("./jltp_n4.json");
const JLPT_N5_RAW: &str = include_str!("./jltp_n5.json");

// Migii N5 files (20 lessons)
const MIGII_N5_1_RAW: &str = include_str!("./migii/n5/migii_n5_1.json");
const MIGII_N5_2_RAW: &str = include_str!("./migii/n5/migii_n5_2.json");
const MIGII_N5_3_RAW: &str = include_str!("./migii/n5/migii_n5_3.json");
const MIGII_N5_4_RAW: &str = include_str!("./migii/n5/migii_n5_4.json");
const MIGII_N5_5_RAW: &str = include_str!("./migii/n5/migii_n5_5.json");
const MIGII_N5_6_RAW: &str = include_str!("./migii/n5/migii_n5_6.json");
const MIGII_N5_7_RAW: &str = include_str!("./migii/n5/migii_n5_7.json");
const MIGII_N5_8_RAW: &str = include_str!("./migii/n5/migii_n5_8.json");
const MIGII_N5_9_RAW: &str = include_str!("./migii/n5/migii_n5_9.json");
const MIGII_N5_10_RAW: &str = include_str!("./migii/n5/migii_n5_10.json");
const MIGII_N5_11_RAW: &str = include_str!("./migii/n5/migii_n5_11.json");
const MIGII_N5_12_RAW: &str = include_str!("./migii/n5/migii_n5_12.json");
const MIGII_N5_13_RAW: &str = include_str!("./migii/n5/migii_n5_13.json");
const MIGII_N5_14_RAW: &str = include_str!("./migii/n5/migii_n5_14.json");
const MIGII_N5_15_RAW: &str = include_str!("./migii/n5/migii_n5_15.json");
const MIGII_N5_16_RAW: &str = include_str!("./migii/n5/migii_n5_16.json");
const MIGII_N5_17_RAW: &str = include_str!("./migii/n5/migii_n5_17.json");
const MIGII_N5_18_RAW: &str = include_str!("./migii/n5/migii_n5_18.json");
const MIGII_N5_19_RAW: &str = include_str!("./migii/n5/migii_n5_19.json");
const MIGII_N5_20_RAW: &str = include_str!("./migii/n5/migii_n5_20.json");

// Migii N4 files (11 lessons)
const MIGII_N4_1_RAW: &str = include_str!("./migii/n4/migii_n4_1.json");
const MIGII_N4_2_RAW: &str = include_str!("./migii/n4/migii_n4_2.json");
const MIGII_N4_3_RAW: &str = include_str!("./migii/n4/migii_n4_3.json");
const MIGII_N4_4_RAW: &str = include_str!("./migii/n4/migii_n4_4.json");
const MIGII_N4_5_RAW: &str = include_str!("./migii/n4/migii_n4_5.json");
const MIGII_N4_6_RAW: &str = include_str!("./migii/n4/migii_n4_6.json");
const MIGII_N4_7_RAW: &str = include_str!("./migii/n4/migii_n4_7.json");
const MIGII_N4_8_RAW: &str = include_str!("./migii/n4/migii_n4_8.json");
const MIGII_N4_9_RAW: &str = include_str!("./migii/n4/migii_n4_9.json");
const MIGII_N4_10_RAW: &str = include_str!("./migii/n4/migii_n4_10.json");
const MIGII_N4_11_RAW: &str = include_str!("./migii/n4/migii_n4_11.json");

// Migii N3 files (31 lessons)
const MIGII_N3_1_RAW: &str = include_str!("./migii/n3/migii_n3_1.json");
const MIGII_N3_2_RAW: &str = include_str!("./migii/n3/migii_n3_2.json");
const MIGII_N3_3_RAW: &str = include_str!("./migii/n3/migii_n3_3.json");
const MIGII_N3_4_RAW: &str = include_str!("./migii/n3/migii_n3_4.json");
const MIGII_N3_5_RAW: &str = include_str!("./migii/n3/migii_n3_5.json");
const MIGII_N3_6_RAW: &str = include_str!("./migii/n3/migii_n3_6.json");
const MIGII_N3_7_RAW: &str = include_str!("./migii/n3/migii_n3_7.json");
const MIGII_N3_8_RAW: &str = include_str!("./migii/n3/migii_n3_8.json");
const MIGII_N3_9_RAW: &str = include_str!("./migii/n3/migii_n3_9.json");
const MIGII_N3_10_RAW: &str = include_str!("./migii/n3/migii_n3_10.json");
const MIGII_N3_11_RAW: &str = include_str!("./migii/n3/migii_n3_11.json");
const MIGII_N3_12_RAW: &str = include_str!("./migii/n3/migii_n3_12.json");
const MIGII_N3_13_RAW: &str = include_str!("./migii/n3/migii_n3_13.json");
const MIGII_N3_14_RAW: &str = include_str!("./migii/n3/migii_n3_14.json");
const MIGII_N3_15_RAW: &str = include_str!("./migii/n3/migii_n3_15.json");
const MIGII_N3_16_RAW: &str = include_str!("./migii/n3/migii_n3_16.json");
const MIGII_N3_17_RAW: &str = include_str!("./migii/n3/migii_n3_17.json");
const MIGII_N3_18_RAW: &str = include_str!("./migii/n3/migii_n3_18.json");
const MIGII_N3_19_RAW: &str = include_str!("./migii/n3/migii_n3_19.json");
const MIGII_N3_20_RAW: &str = include_str!("./migii/n3/migii_n3_20.json");
const MIGII_N3_21_RAW: &str = include_str!("./migii/n3/migii_n3_21.json");
const MIGII_N3_22_RAW: &str = include_str!("./migii/n3/migii_n3_22.json");
const MIGII_N3_23_RAW: &str = include_str!("./migii/n3/migii_n3_23.json");
const MIGII_N3_24_RAW: &str = include_str!("./migii/n3/migii_n3_24.json");
const MIGII_N3_25_RAW: &str = include_str!("./migii/n3/migii_n3_25.json");
const MIGII_N3_26_RAW: &str = include_str!("./migii/n3/migii_n3_26.json");
const MIGII_N3_27_RAW: &str = include_str!("./migii/n3/migii_n3_27.json");
const MIGII_N3_28_RAW: &str = include_str!("./migii/n3/migii_n3_28.json");
const MIGII_N3_29_RAW: &str = include_str!("./migii/n3/migii_n3_29.json");
const MIGII_N3_30_RAW: &str = include_str!("./migii/n3/migii_n3_30.json");
const MIGII_N3_31_RAW: &str = include_str!("./migii/n3/migii_n3_31.json");

// Migii N2 files (31 lessons)
const MIGII_N2_1_RAW: &str = include_str!("./migii/n2/migii_n2_1.json");
const MIGII_N2_2_RAW: &str = include_str!("./migii/n2/migii_n2_2.json");
const MIGII_N2_3_RAW: &str = include_str!("./migii/n2/migii_n2_3.json");
const MIGII_N2_4_RAW: &str = include_str!("./migii/n2/migii_n2_4.json");
const MIGII_N2_5_RAW: &str = include_str!("./migii/n2/migii_n2_5.json");
const MIGII_N2_6_RAW: &str = include_str!("./migii/n2/migii_n2_6.json");
const MIGII_N2_7_RAW: &str = include_str!("./migii/n2/migii_n2_7.json");
const MIGII_N2_8_RAW: &str = include_str!("./migii/n2/migii_n2_8.json");
const MIGII_N2_9_RAW: &str = include_str!("./migii/n2/migii_n2_9.json");
const MIGII_N2_10_RAW: &str = include_str!("./migii/n2/migii_n2_10.json");
const MIGII_N2_11_RAW: &str = include_str!("./migii/n2/migii_n2_11.json");
const MIGII_N2_12_RAW: &str = include_str!("./migii/n2/migii_n2_12.json");
const MIGII_N2_13_RAW: &str = include_str!("./migii/n2/migii_n2_13.json");
const MIGII_N2_14_RAW: &str = include_str!("./migii/n2/migii_n2_14.json");
const MIGII_N2_15_RAW: &str = include_str!("./migii/n2/migii_n2_15.json");
const MIGII_N2_16_RAW: &str = include_str!("./migii/n2/migii_n2_16.json");
const MIGII_N2_17_RAW: &str = include_str!("./migii/n2/migii_n2_17.json");
const MIGII_N2_18_RAW: &str = include_str!("./migii/n2/migii_n2_18.json");
const MIGII_N2_19_RAW: &str = include_str!("./migii/n2/migii_n2_19.json");
const MIGII_N2_20_RAW: &str = include_str!("./migii/n2/migii_n2_20.json");
const MIGII_N2_21_RAW: &str = include_str!("./migii/n2/migii_n2_21.json");
const MIGII_N2_22_RAW: &str = include_str!("./migii/n2/migii_n2_22.json");
const MIGII_N2_23_RAW: &str = include_str!("./migii/n2/migii_n2_23.json");
const MIGII_N2_24_RAW: &str = include_str!("./migii/n2/migii_n2_24.json");
const MIGII_N2_25_RAW: &str = include_str!("./migii/n2/migii_n2_25.json");
const MIGII_N2_26_RAW: &str = include_str!("./migii/n2/migii_n2_26.json");
const MIGII_N2_27_RAW: &str = include_str!("./migii/n2/migii_n2_27.json");
const MIGII_N2_28_RAW: &str = include_str!("./migii/n2/migii_n2_28.json");
const MIGII_N2_29_RAW: &str = include_str!("./migii/n2/migii_n2_29.json");
const MIGII_N2_30_RAW: &str = include_str!("./migii/n2/migii_n2_30.json");
const MIGII_N2_31_RAW: &str = include_str!("./migii/n2/migii_n2_31.json");

// Migii N1 files (56 lessons)
const MIGII_N1_1_RAW: &str = include_str!("./migii/n1/migii_n1_1.json");
const MIGII_N1_2_RAW: &str = include_str!("./migii/n1/migii_n1_2.json");
const MIGII_N1_3_RAW: &str = include_str!("./migii/n1/migii_n1_3.json");
const MIGII_N1_4_RAW: &str = include_str!("./migii/n1/migii_n1_4.json");
const MIGII_N1_5_RAW: &str = include_str!("./migii/n1/migii_n1_5.json");
const MIGII_N1_6_RAW: &str = include_str!("./migii/n1/migii_n1_6.json");
const MIGII_N1_7_RAW: &str = include_str!("./migii/n1/migii_n1_7.json");
const MIGII_N1_8_RAW: &str = include_str!("./migii/n1/migii_n1_8.json");
const MIGII_N1_9_RAW: &str = include_str!("./migii/n1/migii_n1_9.json");
const MIGII_N1_10_RAW: &str = include_str!("./migii/n1/migii_n1_10.json");
const MIGII_N1_11_RAW: &str = include_str!("./migii/n1/migii_n1_11.json");
const MIGII_N1_12_RAW: &str = include_str!("./migii/n1/migii_n1_12.json");
const MIGII_N1_13_RAW: &str = include_str!("./migii/n1/migii_n1_13.json");
const MIGII_N1_14_RAW: &str = include_str!("./migii/n1/migii_n1_14.json");
const MIGII_N1_15_RAW: &str = include_str!("./migii/n1/migii_n1_15.json");
const MIGII_N1_16_RAW: &str = include_str!("./migii/n1/migii_n1_16.json");
const MIGII_N1_17_RAW: &str = include_str!("./migii/n1/migii_n1_17.json");
const MIGII_N1_18_RAW: &str = include_str!("./migii/n1/migii_n1_18.json");
const MIGII_N1_19_RAW: &str = include_str!("./migii/n1/migii_n1_19.json");
const MIGII_N1_20_RAW: &str = include_str!("./migii/n1/migii_n1_20.json");
const MIGII_N1_21_RAW: &str = include_str!("./migii/n1/migii_n1_21.json");
const MIGII_N1_22_RAW: &str = include_str!("./migii/n1/migii_n1_22.json");
const MIGII_N1_23_RAW: &str = include_str!("./migii/n1/migii_n1_23.json");
const MIGII_N1_24_RAW: &str = include_str!("./migii/n1/migii_n1_24.json");
const MIGII_N1_25_RAW: &str = include_str!("./migii/n1/migii_n1_25.json");
const MIGII_N1_26_RAW: &str = include_str!("./migii/n1/migii_n1_26.json");
const MIGII_N1_27_RAW: &str = include_str!("./migii/n1/migii_n1_27.json");
const MIGII_N1_28_RAW: &str = include_str!("./migii/n1/migii_n1_28.json");
const MIGII_N1_29_RAW: &str = include_str!("./migii/n1/migii_n1_29.json");
const MIGII_N1_30_RAW: &str = include_str!("./migii/n1/migii_n1_30.json");
const MIGII_N1_31_RAW: &str = include_str!("./migii/n1/migii_n1_31.json");
const MIGII_N1_32_RAW: &str = include_str!("./migii/n1/migii_n1_32.json");
const MIGII_N1_33_RAW: &str = include_str!("./migii/n1/migii_n1_33.json");
const MIGII_N1_34_RAW: &str = include_str!("./migii/n1/migii_n1_34.json");
const MIGII_N1_35_RAW: &str = include_str!("./migii/n1/migii_n1_35.json");
const MIGII_N1_36_RAW: &str = include_str!("./migii/n1/migii_n1_36.json");
const MIGII_N1_37_RAW: &str = include_str!("./migii/n1/migii_n1_37.json");
const MIGII_N1_38_RAW: &str = include_str!("./migii/n1/migii_n1_38.json");
const MIGII_N1_39_RAW: &str = include_str!("./migii/n1/migii_n1_39.json");
const MIGII_N1_40_RAW: &str = include_str!("./migii/n1/migii_n1_40.json");
const MIGII_N1_41_RAW: &str = include_str!("./migii/n1/migii_n1_41.json");
const MIGII_N1_42_RAW: &str = include_str!("./migii/n1/migii_n1_42.json");
const MIGII_N1_43_RAW: &str = include_str!("./migii/n1/migii_n1_43.json");
const MIGII_N1_44_RAW: &str = include_str!("./migii/n1/migii_n1_44.json");
const MIGII_N1_45_RAW: &str = include_str!("./migii/n1/migii_n1_45.json");
const MIGII_N1_46_RAW: &str = include_str!("./migii/n1/migii_n1_46.json");
const MIGII_N1_47_RAW: &str = include_str!("./migii/n1/migii_n1_47.json");
const MIGII_N1_48_RAW: &str = include_str!("./migii/n1/migii_n1_48.json");
const MIGII_N1_49_RAW: &str = include_str!("./migii/n1/migii_n1_49.json");
const MIGII_N1_50_RAW: &str = include_str!("./migii/n1/migii_n1_50.json");
const MIGII_N1_51_RAW: &str = include_str!("./migii/n1/migii_n1_51.json");
const MIGII_N1_52_RAW: &str = include_str!("./migii/n1/migii_n1_52.json");
const MIGII_N1_53_RAW: &str = include_str!("./migii/n1/migii_n1_53.json");
const MIGII_N1_54_RAW: &str = include_str!("./migii/n1/migii_n1_54.json");
const MIGII_N1_55_RAW: &str = include_str!("./migii/n1/migii_n1_55.json");
const MIGII_N1_56_RAW: &str = include_str!("./migii/n1/migii_n1_56.json");

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum WellKnownSets {
    JlptN1,
    JlptN2,
    JlptN3,
    JlptN4,
    JlptN5,
    MigiiN5Lesson1,
    MigiiN5Lesson2,
    MigiiN5Lesson3,
    MigiiN5Lesson4,
    MigiiN5Lesson5,
    MigiiN5Lesson6,
    MigiiN5Lesson7,
    MigiiN5Lesson8,
    MigiiN5Lesson9,
    MigiiN5Lesson10,
    MigiiN5Lesson11,
    MigiiN5Lesson12,
    MigiiN5Lesson13,
    MigiiN5Lesson14,
    MigiiN5Lesson15,
    MigiiN5Lesson16,
    MigiiN5Lesson17,
    MigiiN5Lesson18,
    MigiiN5Lesson19,
    MigiiN5Lesson20,
    MigiiN4Lesson1,
    MigiiN4Lesson2,
    MigiiN4Lesson3,
    MigiiN4Lesson4,
    MigiiN4Lesson5,
    MigiiN4Lesson6,
    MigiiN4Lesson7,
    MigiiN4Lesson8,
    MigiiN4Lesson9,
    MigiiN4Lesson10,
    MigiiN4Lesson11,
    MigiiN3Lesson1,
    MigiiN3Lesson2,
    MigiiN3Lesson3,
    MigiiN3Lesson4,
    MigiiN3Lesson5,
    MigiiN3Lesson6,
    MigiiN3Lesson7,
    MigiiN3Lesson8,
    MigiiN3Lesson9,
    MigiiN3Lesson10,
    MigiiN3Lesson11,
    MigiiN3Lesson12,
    MigiiN3Lesson13,
    MigiiN3Lesson14,
    MigiiN3Lesson15,
    MigiiN3Lesson16,
    MigiiN3Lesson17,
    MigiiN3Lesson18,
    MigiiN3Lesson19,
    MigiiN3Lesson20,
    MigiiN3Lesson21,
    MigiiN3Lesson22,
    MigiiN3Lesson23,
    MigiiN3Lesson24,
    MigiiN3Lesson25,
    MigiiN3Lesson26,
    MigiiN3Lesson27,
    MigiiN3Lesson28,
    MigiiN3Lesson29,
    MigiiN3Lesson30,
    MigiiN3Lesson31,
    MigiiN2Lesson1,
    MigiiN2Lesson2,
    MigiiN2Lesson3,
    MigiiN2Lesson4,
    MigiiN2Lesson5,
    MigiiN2Lesson6,
    MigiiN2Lesson7,
    MigiiN2Lesson8,
    MigiiN2Lesson9,
    MigiiN2Lesson10,
    MigiiN2Lesson11,
    MigiiN2Lesson12,
    MigiiN2Lesson13,
    MigiiN2Lesson14,
    MigiiN2Lesson15,
    MigiiN2Lesson16,
    MigiiN2Lesson17,
    MigiiN2Lesson18,
    MigiiN2Lesson19,
    MigiiN2Lesson20,
    MigiiN2Lesson21,
    MigiiN2Lesson22,
    MigiiN2Lesson23,
    MigiiN2Lesson24,
    MigiiN2Lesson25,
    MigiiN2Lesson26,
    MigiiN2Lesson27,
    MigiiN2Lesson28,
    MigiiN2Lesson29,
    MigiiN2Lesson30,
    MigiiN2Lesson31,
    MigiiN1Lesson1,
    MigiiN1Lesson2,
    MigiiN1Lesson3,
    MigiiN1Lesson4,
    MigiiN1Lesson5,
    MigiiN1Lesson6,
    MigiiN1Lesson7,
    MigiiN1Lesson8,
    MigiiN1Lesson9,
    MigiiN1Lesson10,
    MigiiN1Lesson11,
    MigiiN1Lesson12,
    MigiiN1Lesson13,
    MigiiN1Lesson14,
    MigiiN1Lesson15,
    MigiiN1Lesson16,
    MigiiN1Lesson17,
    MigiiN1Lesson18,
    MigiiN1Lesson19,
    MigiiN1Lesson20,
    MigiiN1Lesson21,
    MigiiN1Lesson22,
    MigiiN1Lesson23,
    MigiiN1Lesson24,
    MigiiN1Lesson25,
    MigiiN1Lesson26,
    MigiiN1Lesson27,
    MigiiN1Lesson28,
    MigiiN1Lesson29,
    MigiiN1Lesson30,
    MigiiN1Lesson31,
    MigiiN1Lesson32,
    MigiiN1Lesson33,
    MigiiN1Lesson34,
    MigiiN1Lesson35,
    MigiiN1Lesson36,
    MigiiN1Lesson37,
    MigiiN1Lesson38,
    MigiiN1Lesson39,
    MigiiN1Lesson40,
    MigiiN1Lesson41,
    MigiiN1Lesson42,
    MigiiN1Lesson43,
    MigiiN1Lesson44,
    MigiiN1Lesson45,
    MigiiN1Lesson46,
    MigiiN1Lesson47,
    MigiiN1Lesson48,
    MigiiN1Lesson49,
    MigiiN1Lesson50,
    MigiiN1Lesson51,
    MigiiN1Lesson52,
    MigiiN1Lesson53,
    MigiiN1Lesson54,
    MigiiN1Lesson55,
    MigiiN1Lesson56,
}

pub fn load_well_known_set(set: &WellKnownSets) -> Result<WellKnownSet, OrigaError> {
    let raw = match set {
        WellKnownSets::JlptN1 => JLPT_N1_RAW,
        WellKnownSets::JlptN2 => JLPT_N2_RAW,
        WellKnownSets::JlptN3 => JLPT_N3_RAW,
        WellKnownSets::JlptN4 => JLPT_N4_RAW,
        WellKnownSets::JlptN5 => JLPT_N5_RAW,
        WellKnownSets::MigiiN5Lesson1 => MIGII_N5_1_RAW,
        WellKnownSets::MigiiN5Lesson2 => MIGII_N5_2_RAW,
        WellKnownSets::MigiiN5Lesson3 => MIGII_N5_3_RAW,
        WellKnownSets::MigiiN5Lesson4 => MIGII_N5_4_RAW,
        WellKnownSets::MigiiN5Lesson5 => MIGII_N5_5_RAW,
        WellKnownSets::MigiiN5Lesson6 => MIGII_N5_6_RAW,
        WellKnownSets::MigiiN5Lesson7 => MIGII_N5_7_RAW,
        WellKnownSets::MigiiN5Lesson8 => MIGII_N5_8_RAW,
        WellKnownSets::MigiiN5Lesson9 => MIGII_N5_9_RAW,
        WellKnownSets::MigiiN5Lesson10 => MIGII_N5_10_RAW,
        WellKnownSets::MigiiN5Lesson11 => MIGII_N5_11_RAW,
        WellKnownSets::MigiiN5Lesson12 => MIGII_N5_12_RAW,
        WellKnownSets::MigiiN5Lesson13 => MIGII_N5_13_RAW,
        WellKnownSets::MigiiN5Lesson14 => MIGII_N5_14_RAW,
        WellKnownSets::MigiiN5Lesson15 => MIGII_N5_15_RAW,
        WellKnownSets::MigiiN5Lesson16 => MIGII_N5_16_RAW,
        WellKnownSets::MigiiN5Lesson17 => MIGII_N5_17_RAW,
        WellKnownSets::MigiiN5Lesson18 => MIGII_N5_18_RAW,
        WellKnownSets::MigiiN5Lesson19 => MIGII_N5_19_RAW,
        WellKnownSets::MigiiN5Lesson20 => MIGII_N5_20_RAW,
        WellKnownSets::MigiiN4Lesson1 => MIGII_N4_1_RAW,
        WellKnownSets::MigiiN4Lesson2 => MIGII_N4_2_RAW,
        WellKnownSets::MigiiN4Lesson3 => MIGII_N4_3_RAW,
        WellKnownSets::MigiiN4Lesson4 => MIGII_N4_4_RAW,
        WellKnownSets::MigiiN4Lesson5 => MIGII_N4_5_RAW,
        WellKnownSets::MigiiN4Lesson6 => MIGII_N4_6_RAW,
        WellKnownSets::MigiiN4Lesson7 => MIGII_N4_7_RAW,
        WellKnownSets::MigiiN4Lesson8 => MIGII_N4_8_RAW,
        WellKnownSets::MigiiN4Lesson9 => MIGII_N4_9_RAW,
        WellKnownSets::MigiiN4Lesson10 => MIGII_N4_10_RAW,
        WellKnownSets::MigiiN4Lesson11 => MIGII_N4_11_RAW,
        WellKnownSets::MigiiN3Lesson1 => MIGII_N3_1_RAW,
        WellKnownSets::MigiiN3Lesson2 => MIGII_N3_2_RAW,
        WellKnownSets::MigiiN3Lesson3 => MIGII_N3_3_RAW,
        WellKnownSets::MigiiN3Lesson4 => MIGII_N3_4_RAW,
        WellKnownSets::MigiiN3Lesson5 => MIGII_N3_5_RAW,
        WellKnownSets::MigiiN3Lesson6 => MIGII_N3_6_RAW,
        WellKnownSets::MigiiN3Lesson7 => MIGII_N3_7_RAW,
        WellKnownSets::MigiiN3Lesson8 => MIGII_N3_8_RAW,
        WellKnownSets::MigiiN3Lesson9 => MIGII_N3_9_RAW,
        WellKnownSets::MigiiN3Lesson10 => MIGII_N3_10_RAW,
        WellKnownSets::MigiiN3Lesson11 => MIGII_N3_11_RAW,
        WellKnownSets::MigiiN3Lesson12 => MIGII_N3_12_RAW,
        WellKnownSets::MigiiN3Lesson13 => MIGII_N3_13_RAW,
        WellKnownSets::MigiiN3Lesson14 => MIGII_N3_14_RAW,
        WellKnownSets::MigiiN3Lesson15 => MIGII_N3_15_RAW,
        WellKnownSets::MigiiN3Lesson16 => MIGII_N3_16_RAW,
        WellKnownSets::MigiiN3Lesson17 => MIGII_N3_17_RAW,
        WellKnownSets::MigiiN3Lesson18 => MIGII_N3_18_RAW,
        WellKnownSets::MigiiN3Lesson19 => MIGII_N3_19_RAW,
        WellKnownSets::MigiiN3Lesson20 => MIGII_N3_20_RAW,
        WellKnownSets::MigiiN3Lesson21 => MIGII_N3_21_RAW,
        WellKnownSets::MigiiN3Lesson22 => MIGII_N3_22_RAW,
        WellKnownSets::MigiiN3Lesson23 => MIGII_N3_23_RAW,
        WellKnownSets::MigiiN3Lesson24 => MIGII_N3_24_RAW,
        WellKnownSets::MigiiN3Lesson25 => MIGII_N3_25_RAW,
        WellKnownSets::MigiiN3Lesson26 => MIGII_N3_26_RAW,
        WellKnownSets::MigiiN3Lesson27 => MIGII_N3_27_RAW,
        WellKnownSets::MigiiN3Lesson28 => MIGII_N3_28_RAW,
        WellKnownSets::MigiiN3Lesson29 => MIGII_N3_29_RAW,
        WellKnownSets::MigiiN3Lesson30 => MIGII_N3_30_RAW,
        WellKnownSets::MigiiN3Lesson31 => MIGII_N3_31_RAW,
        WellKnownSets::MigiiN2Lesson1 => MIGII_N2_1_RAW,
        WellKnownSets::MigiiN2Lesson2 => MIGII_N2_2_RAW,
        WellKnownSets::MigiiN2Lesson3 => MIGII_N2_3_RAW,
        WellKnownSets::MigiiN2Lesson4 => MIGII_N2_4_RAW,
        WellKnownSets::MigiiN2Lesson5 => MIGII_N2_5_RAW,
        WellKnownSets::MigiiN2Lesson6 => MIGII_N2_6_RAW,
        WellKnownSets::MigiiN2Lesson7 => MIGII_N2_7_RAW,
        WellKnownSets::MigiiN2Lesson8 => MIGII_N2_8_RAW,
        WellKnownSets::MigiiN2Lesson9 => MIGII_N2_9_RAW,
        WellKnownSets::MigiiN2Lesson10 => MIGII_N2_10_RAW,
        WellKnownSets::MigiiN2Lesson11 => MIGII_N2_11_RAW,
        WellKnownSets::MigiiN2Lesson12 => MIGII_N2_12_RAW,
        WellKnownSets::MigiiN2Lesson13 => MIGII_N2_13_RAW,
        WellKnownSets::MigiiN2Lesson14 => MIGII_N2_14_RAW,
        WellKnownSets::MigiiN2Lesson15 => MIGII_N2_15_RAW,
        WellKnownSets::MigiiN2Lesson16 => MIGII_N2_16_RAW,
        WellKnownSets::MigiiN2Lesson17 => MIGII_N2_17_RAW,
        WellKnownSets::MigiiN2Lesson18 => MIGII_N2_18_RAW,
        WellKnownSets::MigiiN2Lesson19 => MIGII_N2_19_RAW,
        WellKnownSets::MigiiN2Lesson20 => MIGII_N2_20_RAW,
        WellKnownSets::MigiiN2Lesson21 => MIGII_N2_21_RAW,
        WellKnownSets::MigiiN2Lesson22 => MIGII_N2_22_RAW,
        WellKnownSets::MigiiN2Lesson23 => MIGII_N2_23_RAW,
        WellKnownSets::MigiiN2Lesson24 => MIGII_N2_24_RAW,
        WellKnownSets::MigiiN2Lesson25 => MIGII_N2_25_RAW,
        WellKnownSets::MigiiN2Lesson26 => MIGII_N2_26_RAW,
        WellKnownSets::MigiiN2Lesson27 => MIGII_N2_27_RAW,
        WellKnownSets::MigiiN2Lesson28 => MIGII_N2_28_RAW,
        WellKnownSets::MigiiN2Lesson29 => MIGII_N2_29_RAW,
        WellKnownSets::MigiiN2Lesson30 => MIGII_N2_30_RAW,
        WellKnownSets::MigiiN2Lesson31 => MIGII_N2_31_RAW,
        WellKnownSets::MigiiN1Lesson1 => MIGII_N1_1_RAW,
        WellKnownSets::MigiiN1Lesson2 => MIGII_N1_2_RAW,
        WellKnownSets::MigiiN1Lesson3 => MIGII_N1_3_RAW,
        WellKnownSets::MigiiN1Lesson4 => MIGII_N1_4_RAW,
        WellKnownSets::MigiiN1Lesson5 => MIGII_N1_5_RAW,
        WellKnownSets::MigiiN1Lesson6 => MIGII_N1_6_RAW,
        WellKnownSets::MigiiN1Lesson7 => MIGII_N1_7_RAW,
        WellKnownSets::MigiiN1Lesson8 => MIGII_N1_8_RAW,
        WellKnownSets::MigiiN1Lesson9 => MIGII_N1_9_RAW,
        WellKnownSets::MigiiN1Lesson10 => MIGII_N1_10_RAW,
        WellKnownSets::MigiiN1Lesson11 => MIGII_N1_11_RAW,
        WellKnownSets::MigiiN1Lesson12 => MIGII_N1_12_RAW,
        WellKnownSets::MigiiN1Lesson13 => MIGII_N1_13_RAW,
        WellKnownSets::MigiiN1Lesson14 => MIGII_N1_14_RAW,
        WellKnownSets::MigiiN1Lesson15 => MIGII_N1_15_RAW,
        WellKnownSets::MigiiN1Lesson16 => MIGII_N1_16_RAW,
        WellKnownSets::MigiiN1Lesson17 => MIGII_N1_17_RAW,
        WellKnownSets::MigiiN1Lesson18 => MIGII_N1_18_RAW,
        WellKnownSets::MigiiN1Lesson19 => MIGII_N1_19_RAW,
        WellKnownSets::MigiiN1Lesson20 => MIGII_N1_20_RAW,
        WellKnownSets::MigiiN1Lesson21 => MIGII_N1_21_RAW,
        WellKnownSets::MigiiN1Lesson22 => MIGII_N1_22_RAW,
        WellKnownSets::MigiiN1Lesson23 => MIGII_N1_23_RAW,
        WellKnownSets::MigiiN1Lesson24 => MIGII_N1_24_RAW,
        WellKnownSets::MigiiN1Lesson25 => MIGII_N1_25_RAW,
        WellKnownSets::MigiiN1Lesson26 => MIGII_N1_26_RAW,
        WellKnownSets::MigiiN1Lesson27 => MIGII_N1_27_RAW,
        WellKnownSets::MigiiN1Lesson28 => MIGII_N1_28_RAW,
        WellKnownSets::MigiiN1Lesson29 => MIGII_N1_29_RAW,
        WellKnownSets::MigiiN1Lesson30 => MIGII_N1_30_RAW,
        WellKnownSets::MigiiN1Lesson31 => MIGII_N1_31_RAW,
        WellKnownSets::MigiiN1Lesson32 => MIGII_N1_32_RAW,
        WellKnownSets::MigiiN1Lesson33 => MIGII_N1_33_RAW,
        WellKnownSets::MigiiN1Lesson34 => MIGII_N1_34_RAW,
        WellKnownSets::MigiiN1Lesson35 => MIGII_N1_35_RAW,
        WellKnownSets::MigiiN1Lesson36 => MIGII_N1_36_RAW,
        WellKnownSets::MigiiN1Lesson37 => MIGII_N1_37_RAW,
        WellKnownSets::MigiiN1Lesson38 => MIGII_N1_38_RAW,
        WellKnownSets::MigiiN1Lesson39 => MIGII_N1_39_RAW,
        WellKnownSets::MigiiN1Lesson40 => MIGII_N1_40_RAW,
        WellKnownSets::MigiiN1Lesson41 => MIGII_N1_41_RAW,
        WellKnownSets::MigiiN1Lesson42 => MIGII_N1_42_RAW,
        WellKnownSets::MigiiN1Lesson43 => MIGII_N1_43_RAW,
        WellKnownSets::MigiiN1Lesson44 => MIGII_N1_44_RAW,
        WellKnownSets::MigiiN1Lesson45 => MIGII_N1_45_RAW,
        WellKnownSets::MigiiN1Lesson46 => MIGII_N1_46_RAW,
        WellKnownSets::MigiiN1Lesson47 => MIGII_N1_47_RAW,
        WellKnownSets::MigiiN1Lesson48 => MIGII_N1_48_RAW,
        WellKnownSets::MigiiN1Lesson49 => MIGII_N1_49_RAW,
        WellKnownSets::MigiiN1Lesson50 => MIGII_N1_50_RAW,
        WellKnownSets::MigiiN1Lesson51 => MIGII_N1_51_RAW,
        WellKnownSets::MigiiN1Lesson52 => MIGII_N1_52_RAW,
        WellKnownSets::MigiiN1Lesson53 => MIGII_N1_53_RAW,
        WellKnownSets::MigiiN1Lesson54 => MIGII_N1_54_RAW,
        WellKnownSets::MigiiN1Lesson55 => MIGII_N1_55_RAW,
        WellKnownSets::MigiiN1Lesson56 => MIGII_N1_56_RAW,
    };

    serde_json::from_str(raw).map_err(|e| OrigaError::WellKnownSetParseError {
        reason: format!("Error parse stored value: {e}"),
    })
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WellKnownSet {
    level: JapaneseLevel,
    words: Vec<String>,
    content: HashMap<NativeLanguage, WellKnownSetContent>,
}

impl WellKnownSet {
    pub fn words(&self) -> &[String] {
        &self.words
    }

    pub fn content(&self, lang: &NativeLanguage) -> &WellKnownSetContent {
        &self.content[lang]
    }

    pub fn level(&self) -> &JapaneseLevel {
        &self.level
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WellKnownSetContent {
    title: String,
    description: String,
}

impl WellKnownSetContent {
    pub fn title(&self) -> &str {
        &self.title
    }

    pub fn description(&self) -> &str {
        &self.description
    }
}
