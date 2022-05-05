#[derive(Clone, Debug, Hash, PartialEq, Eq)]
pub enum GameState {
    WaitingInput,
    MonsterAi,
    Turn,
    Inventory,
    DropItemMenu,
}

#[derive(Clone, Debug, Hash, PartialEq, Eq)]
pub enum CurrentTurn {
    Player,
    Monster,
}

impl CurrentTurn {
    pub fn change(&mut self) {
        *self = match self {
            CurrentTurn::Player => CurrentTurn::Monster,
            CurrentTurn::Monster => CurrentTurn::Player,
        }
    }
}
