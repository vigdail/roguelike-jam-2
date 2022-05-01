#[derive(Clone, Copy, Debug, Hash, PartialEq, Eq)]
pub enum GameState {
    WaitingInput,
    PlayerTurn,
    MonsterTurn,
}
