pub type PlayerId = u64;

#[derive(Debug, Clone, PartialEq)]
pub struct Player {
    pub id: PlayerId,
    pub name: String,
    pub direction: i32,
    pub score: usize,
}
