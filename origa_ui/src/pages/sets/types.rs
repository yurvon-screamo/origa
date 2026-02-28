use origa::domain::WellKnownSets;

#[derive(Clone, Debug, PartialEq)]
pub struct ImportState {
    pub set: WellKnownSets,
    pub title: String,
}

#[derive(Clone, Debug, PartialEq)]
pub struct ImportResult {
    pub is_success: bool,
    pub message: String,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum SetType {
    Jlpt,
    Migii,
}

impl SetType {
    pub fn label(&self) -> &'static str {
        match self {
            SetType::Jlpt => "JLPT",
            SetType::Migii => "Migii",
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum JlptLevel {
    N5,
    N4,
    N3,
    N2,
    N1,
}

impl JlptLevel {
    pub fn label(&self) -> &'static str {
        match self {
            JlptLevel::N5 => "N5",
            JlptLevel::N4 => "N4",
            JlptLevel::N3 => "N3",
            JlptLevel::N2 => "N2",
            JlptLevel::N1 => "N1",
        }
    }
}

#[derive(Clone, PartialEq)]
pub struct SetInfo {
    pub set: WellKnownSets,
    pub title: String,
    pub description: String,
    pub word_count: usize,
    pub set_type: SetType,
    pub level: JlptLevel,
}

pub fn classify_set(set: &WellKnownSets) -> (SetType, JlptLevel) {
    let name = format!("{:?}", set);
    let set_type = if name.starts_with("Jlpt") {
        SetType::Jlpt
    } else {
        SetType::Migii
    };
    let level = if name.contains("N5") {
        JlptLevel::N5
    } else if name.contains("N4") {
        JlptLevel::N4
    } else if name.contains("N3") {
        JlptLevel::N3
    } else if name.contains("N2") {
        JlptLevel::N2
    } else {
        JlptLevel::N1
    };
    (set_type, level)
}
