use ratatui::{
    style::{Color, Stylize},
    text::Line,
};
use serde::{Deserialize, Serialize};

pub type PlayerId = u64;

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Player {
    pub id: PlayerId,
    pub name: String,
    pub direction: i32,
    pub score: usize,
}

impl Player {
    pub fn pretty_print_scoreboard(
        &self,
        is_playing: PlayerId,
        name_color: Color,
    ) -> Line<'static> {
        Line::from(vec![
            format!(
                "{}{}",
                if self.id == is_playing { "(*)" } else { "" },
                self.name
            )
            .fg(name_color),
            format!(" score:").white(),
            format!(" {}", self.score).white().bold(),
            format!(" ({})", self.direction).white().bold(),
        ])
    }
}
