use leptos::prelude::*;
use crate::components::forms::chip_group::ChipGroup;

#[component]
pub function JlptLevelFilter(
    #[prop(into, optional)] selected_level: Option<Signal<JlptLevel>>,
    #[prop(into, optional)] on_select: Option<Callback<JlptLevel>>,
    #[prop(into, optional)] show_counts: Option<bool>,
) -> impl IntoView {
    let selected = selected_level.unwrap_or_else(|| create_signal(JlptLevel::N5));
    let with_counts = show_counts.unwrap_or(true);
    
    let handle_select = move |level: JlptLevel| {
        if let Some(handler) = on_select {
            handler.run(level);
        }
    };
    
    // Mock counts - will be replaced with real data
    let level_counts = create_mocks();
    
    let chips = Signal::derive(move || {
        JlptLevel::ALL_LEVELS.iter().map(|&level| {
            crate::components::forms::chip_group::ChipItem {
                id: level.to_string(),
                label: level.to_string(),
                count: if with_counts { level_counts.get(&level).copied() } else { None },
                active: selected() == level,
                color: level_to_color(level),
            }
        }).collect::<Vec<_>>()
    });
    
    view! {
        <div class="jlpt-filter">
            <div class="filter-header">
                <h3 class="filter-title">Уровень JLPT</h3>
                <p class="filter-subtitle">Выберите уровень сложности кандзи</p>
            </div>
            
            <ChipGroup 
                chips=chips
                on_select=Callback::new(move |chip_id| {
                    if let Ok(level) = JlptLevel::from_str(&chip_id) {
                        handle_select(level);
                    }
                }) />
            
            // Progress indicator for selected level
            <div class="level-progress">
                <div class="progress-text">
                    "Уровень " 
                    {move || selected().to_string()}
                    " • "
                    {move || {
                        let level = selected();
                        level_counts.get(&level).unwrap_or(&0)
                    }}
                    " кандзи"
                </div>
                <div class="progress-bar">
                    <div class="progress-fill" style="width: 25%"></div>
                </div>
            </div>
        </div>
    }
}

#[derive(Clone, Copy, PartialEq, Debug)]
pub enum JlptLevel {
    N5,
    N4,
    N3,
    N2,
    N1,
}

impl JlptLevel {
    pub const ALL_LEVELS: [JlptLevel; 5] = [
        JlptLevel::N5,
        JlptLevel::N4,
        JlptLevel::N3,
        JlptLevel::N2,
        JlptLevel::N1,
    ];
    
    pub fn to_string(&self) -> &'static str {
        match self {
            JlptLevel::N5 => "N5",
            JlptLevel::N4 => "N4",
            JlptLevel::N3 => "N3",
            JlptLevel::N2 => "N2",
            JlptLevel::N1 => "N1",
        }
    }
    
    pub fn description(&self) -> &'static str {
        match self {
            JlptLevel::N5 => "Базовый уровень (самый простой)",
            JlptLevel::N4 => "Начальный уровень",
            JlptLevel::N3 => "Средний уровень",
            JlptLevel::N2 => "Продвинутый уровень",
            JlptLevel::N1 => "Экспертный уровень (самый сложный)",
        }
    }
    
    pub fn difficulty_color(&self) -> &'static str {
        match self {
            JlptLevel::N5 => "#5a8c5a",  // Green
            JlptLevel::N4 => "#66a182",  // Light green
            JlptLevel::N3 => "#b08d57",  // Yellow
            JlptLevel::N2 => "#b85450",  // Light red
            JlptLevel::N1 => "#8b2635",  // Dark red
        }
    }
}

impl std::fmt::Display for JlptLevel {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.to_string())
    }
}

impl std::str::FromStr for JlptLevel {
    type Err = ();
    
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "N5" => Ok(JlptLevel::N5),
            "N4" => Ok(JlptLevel::N4),
            "N3" => Ok(JlptLevel::N3),
            "N2" => Ok(JlptLevel::N2),
            "N1" => Ok(JlptLevel::N1),
            _ => Err(()),
        }
    }
}

fn level_to_color(level: JlptLevel) -> &'static str {
    level.difficulty_color()
}

fn create_mocks() -> std::collections::HashMap<JlptLevel, u32> {
    let mut counts = std::collections::HashMap::new();
    counts.insert(JlptLevel::N5, 103);
    counts.insert(JlptLevel::N4, 181);
    counts.insert(JlptLevel::N3, 369);
    counts.insert(JlptLevel::N2, 374);
    counts.insert(JlptLevel::N1, 1235);
    counts
}