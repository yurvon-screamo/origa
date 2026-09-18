//! Добивание [GhostState] — параллельная FSRS механика закрепления
//! проваленных карт (successive relearning). Контракт и правила переходов:
//! `docs/plans/ghost-relearning.md` §3.1–3.2; термины — `docs/glossary.md`.
//!
//! Машина состояний — чистые переходы с инъекцией времени: методы принимают
//! `now`, ничего не читают из окружения. Вызов разрешён ТОЛЬКО из пути
//! явного показа карточки (`RatingContext::Explicit`); неявные рейтинги
//! (двойной рейтинг грамматики, «уже знаю», сидирование знакомства)
//! физически обходят `apply_ghost_transition`.

use chrono::{DateTime, Duration, Utc};
use serde::{Deserialize, Serialize};
use tracing::info;
use ulid::Ulid;

use super::{MemoryHistory, Rating};

/// TTL добивания: 30 дней без единого шага — списание (`GhostState::Expired`).
const GHOST_TTL_DAYS: i64 = 30;

/// Шаг лестницы добивания [GhostRung]. Позиция кодирует накопленные
/// успехи: First = 0, Second = 1, Third = 2; успех на Third закрывает.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum GhostRung {
    First,
    Second,
    Third,
}

impl GhostRung {
    /// Интервал до показа, который засчитывается на этой ступени:
    /// спавн/рестарт → First (12ч), успех на First → Second (1д),
    /// успех на Second → Third (3д). Успех на Third закрывает добивание.
    fn interval(self) -> Duration {
        match self {
            GhostRung::First => Duration::hours(12),
            GhostRung::Second => Duration::days(1),
            GhostRung::Third => Duration::days(3),
        }
    }

    fn next(self) -> Option<GhostRung> {
        match self {
            GhostRung::First => Some(GhostRung::Second),
            GhostRung::Second => Some(GhostRung::Third),
            GhostRung::Third => None,
        }
    }

    fn label(self) -> &'static str {
        match self {
            GhostRung::First => "first",
            GhostRung::Second => "second",
            GhostRung::Third => "third",
        }
    }
}

/// Состояние добивания [GhostState]. `Active` — живая лестница;
/// `Resolved` («добито») и `Expired` (TTL-списание) — терминальные
/// маркеры: переходов не совершают, существуют для LWW-мержа и
/// замещаются новым спавном.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum GhostState {
    Active {
        rung: GhostRung,
        due_at: DateTime<Utc>,
        last_transition_at: DateTime<Utc>,
    },
    Resolved {
        at: DateTime<Utc>,
    },
    Expired {
        at: DateTime<Utc>,
    },
}

impl GhostState {
    #[cfg(test)]
    pub(crate) fn active(
        rung: GhostRung,
        due_at: DateTime<Utc>,
        last_transition_at: DateTime<Utc>,
    ) -> Self {
        GhostState::Active {
            rung,
            due_at,
            last_transition_at,
        }
    }

    pub fn is_active(&self) -> bool {
        matches!(self, GhostState::Active { .. })
    }

    pub fn due_at(&self) -> Option<&DateTime<Utc>> {
        match self {
            GhostState::Active { due_at, .. } => Some(due_at),
            _ => None,
        }
    }

    /// Момент последнего шага — база LWW-мержа и TTL.
    pub fn last_transition_at(&self) -> DateTime<Utc> {
        match self {
            GhostState::Active {
                last_transition_at, ..
            } => *last_transition_at,
            GhostState::Resolved { at } | GhostState::Expired { at } => *at,
        }
    }
}

#[cfg(test)]
impl MemoryHistory {
    /// Установка добивания напрямую: time-travel для journey-тестов
    /// (открыть окно/состарить TTL) и сборка состояний отбора.
    pub(crate) fn set_ghost_for_test(&mut self, ghost: Option<GhostState>) {
        self.ghost = ghost;
    }
}

#[cfg(test)]
mod tests;

impl MemoryHistory {
    /// Активное (не списанное по TTL) добивание — карта исключается из
    /// core-отбора и padding: единственный канал показа — рука добиваний
    /// или избранное.
    pub fn has_active_ghost(&self, now: DateTime<Utc>) -> bool {
        match self.ghost {
            Some(GhostState::Active {
                last_transition_at, ..
            }) => now - last_transition_at <= Duration::days(GHOST_TTL_DAYS),
            _ => false,
        }
    }

    /// Готовность к руке добиваний [GhostHand]: активное добивание
    /// с открытым окном (`due_at <= now`) и не истёкшим TTL.
    pub fn ghost_hand_ready(&self, now: DateTime<Utc>) -> bool {
        match self.ghost {
            Some(GhostState::Active {
                due_at,
                last_transition_at,
                ..
            }) => due_at <= now && now - last_transition_at <= Duration::days(GHOST_TTL_DAYS),
            _ => false,
        }
    }

    /// Единственная точка входа машины добиваний. `was_new_before_rating` —
    /// новизна карты ДО применённого ревью: спавн невозможен на первом
    /// ревью карты (гейт «карта до первого ревью не спавнит»).
    pub(crate) fn apply_ghost_transition(
        &mut self,
        rating: Rating,
        now: DateTime<Utc>,
        was_new_before_rating: bool,
        card_id: &Ulid,
    ) {
        self.expire_stale_ghost(now, card_id);

        match rating {
            Rating::Good | Rating::Easy => self.on_ghost_success(now, card_id),
            Rating::Again | Rating::Hard => {
                self.on_ghost_failure(now, was_new_before_rating, card_id)
            },
        }
    }

    /// Ленивый TTL: списание добивания, молчащего дольше 30 дней.
    fn expire_stale_ghost(&mut self, now: DateTime<Utc>, card_id: &Ulid) {
        if let Some(GhostState::Active {
            last_transition_at, ..
        }) = &self.ghost
        {
            if now - last_transition_at > Duration::days(GHOST_TTL_DAYS) {
                info!(card_id = %card_id, "ghost_expired");
                self.ghost = Some(GhostState::Expired { at: now });
                self.consecutive_again = 0;
            }
        }
    }

    fn on_ghost_success(&mut self, now: DateTime<Utc>, card_id: &Ulid) {
        self.consecutive_again = 0;
        let Some(GhostState::Active { rung, due_at, .. }) = &self.ghost else {
            return;
        };
        if *due_at > now {
            return;
        }

        match rung.next() {
            Some(next_rung) => {
                info!(card_id = %card_id, rung = next_rung.label(), "ghost_advanced");
                self.ghost = Some(GhostState::Active {
                    rung: next_rung,
                    due_at: now + next_rung.interval(),
                    last_transition_at: now,
                });
            },
            None => {
                info!(card_id = %card_id, "ghost_resolved");
                self.ghost = Some(GhostState::Resolved { at: now });
            },
        }
    }

    fn on_ghost_failure(
        &mut self,
        now: DateTime<Utc>,
        was_new_before_rating: bool,
        card_id: &Ulid,
    ) {
        if let Some(GhostState::Active { due_at, .. }) = &self.ghost {
            if *due_at <= now {
                info!(card_id = %card_id, "ghost_restarted");
                self.ghost = Some(GhostState::Active {
                    rung: GhostRung::First,
                    due_at: now + GhostRung::First.interval(),
                    last_transition_at: now,
                });
            }
            return;
        }

        // Предспавн: копим подряд-провалы (терминальные состояния
        // замещаются новым спавном — новый цикл после новой серии).
        self.consecutive_again += 1;
        if self.consecutive_again >= 2 && !was_new_before_rating {
            info!(card_id = %card_id, "ghost_spawned");
            self.ghost = Some(GhostState::Active {
                rung: GhostRung::First,
                due_at: now + GhostRung::First.interval(),
                last_transition_at: now,
            });
            self.consecutive_again = 0;
        }
    }
}
