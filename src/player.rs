use ratatui::{style::Stylize, text::Line};

pub type PlayerId = u64;

#[derive(Debug, Clone, PartialEq)]
pub struct Player {
    pub id: PlayerId,
    pub name: String,
    pub direction: i32,
    pub score: usize,
}

impl Player {
    pub fn pretty_print_scoreboard(&self) -> Line<'static> {
        Line::from(vec![
            format!("player {:?}: ", self.id).into(),
            format!("{:?}", self.name).green(),
            format!(" score:").white(),
            format!(" {:?}", self.score).white().bold(),
        ])
    }
}
