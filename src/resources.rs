#[derive(Clone, Debug, Hash, PartialEq, Eq)]
pub enum GameState {
    Gameplay,
    Inventory,
    DropItemMenu,
}
