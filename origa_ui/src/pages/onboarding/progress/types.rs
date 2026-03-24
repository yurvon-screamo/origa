#[derive(Clone, Debug)]
pub struct DuolingoModule {
    pub module_number: usize,
    pub units: Vec<DuolingoUnit>,
}

#[derive(Clone, Debug)]
pub struct DuolingoUnit {
    pub id: String,
    pub unit_number: usize,
}

#[derive(Clone, Debug)]
pub struct MigiiLesson {
    pub id: String,
    pub lesson_number: usize,
}

#[derive(Clone, Debug)]
pub struct MinnaLesson {
    pub id: String,
    pub lesson_number: usize,
}
