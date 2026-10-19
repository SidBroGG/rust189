use hashbrown::{HashMap, HashSet};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ScoreboardObjective {
    pub name: String,
    pub display_title: String,
    pub criteria_type: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ScoreboardTeam {
    pub name: String,
    pub display_name: String,
    pub prefix: String,
    pub suffix: String,
    pub members: HashSet<String>,
}

#[derive(Debug, Clone, Default)]
pub struct Scoreboard {
    pub objectives: HashMap<String, ScoreboardObjective>,
    /// Maps (objective_name, entry_name) -> score value
    pub scores: HashMap<(String, String), i32>,
    /// Display slots: 0 = List/Tab, 1 = Sidebar, 2 = BelowName
    pub display_slots: [Option<String>; 3],
    pub teams: HashMap<String, ScoreboardTeam>,
}

impl Scoreboard {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn add_objective(&mut self, name: String, display_title: String, criteria_type: String) {
        self.objectives.insert(
            name.clone(),
            ScoreboardObjective {
                name,
                display_title,
                criteria_type,
            },
        );
    }

    pub fn remove_objective(&mut self, name: &str) {
        self.objectives.remove(name);
        self.scores.retain(|(obj, _), _| obj != name);
        for slot in &mut self.display_slots {
            if slot.as_deref() == Some(name) {
                *slot = None;
            }
        }
    }

    pub fn update_objective_title(&mut self, name: &str, display_title: String) {
        if let Some(obj) = self.objectives.get_mut(name) {
            obj.display_title = display_title;
        }
    }

    pub fn set_display_slot(&mut self, position: usize, name: Option<String>) {
        if position < 3 {
            self.display_slots[position] = name;
        }
    }

    pub fn set_score(&mut self, objective: String, entry: String, value: i32) {
        self.scores.insert((objective, entry), value);
    }

    pub fn remove_score(&mut self, objective: &str, entry: &str) {
        self.scores.remove(&(objective.to_string(), entry.to_string()));
    }

    /// Returns the sidebar title and list of scores sorted in descending order by score value.
    pub fn get_sidebar_lines(&self) -> Option<(String, Vec<(String, i32)>)> {
        let objective_name = self.display_slots[1].as_ref()?;
        let obj = self.objectives.get(objective_name)?;

        let mut lines: Vec<(String, i32)> = self
            .scores
            .iter()
            .filter(|((o, _), _)| o == objective_name)
            .map(|((_, entry), &val)| (self.format_player_name(entry), val))
            .collect();

        // Sort by score descending; if equal, by entry name
        lines.sort_by(|a, b| b.1.cmp(&a.1).then_with(|| a.0.cmp(&b.0)));

        Some((obj.display_title.clone(), lines))
    }

    pub fn clear(&mut self) {
        self.objectives.clear();
        self.scores.clear();
        self.display_slots = [None, None, None];
        self.teams.clear();
    }

    pub fn create_team(&mut self, name: String) -> &mut ScoreboardTeam {
        self.teams.entry(name.clone()).or_insert_with(|| ScoreboardTeam {
            name,
            display_name: String::new(),
            prefix: String::new(),
            suffix: String::new(),
            members: HashSet::new(),
        })
    }

    pub fn remove_team(&mut self, name: &str) {
        self.teams.remove(name);
    }

    pub fn get_team(&self, name: &str) -> Option<&ScoreboardTeam> {
        self.teams.get(name)
    }

    pub fn get_team_mut(&mut self, name: &str) -> Option<&mut ScoreboardTeam> {
        self.teams.get_mut(name)
    }

    pub fn add_player_to_team(&mut self, team_name: &str, player: String) {
        for t in self.teams.values_mut() {
            t.members.remove(&player);
        }
        if let Some(team) = self.teams.get_mut(team_name) {
            team.members.insert(player);
        }
    }

    pub fn remove_player_from_team(&mut self, team_name: &str, player: &str) {
        if let Some(team) = self.teams.get_mut(team_name) {
            team.members.remove(player);
        }
    }

    pub fn get_player_team(&self, player_name: &str) -> Option<&ScoreboardTeam> {
        self.teams.values().find(|t| t.members.contains(player_name))
    }

    pub fn format_player_name(&self, player_name: &str) -> String {
        if let Some(team) = self.get_player_team(player_name) {
            format!("{}{}{}", team.prefix, player_name, team.suffix)
        } else {
            player_name.to_string()
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_scoreboard_sidebar() {
        let mut sb = Scoreboard::new();
        sb.add_objective(
            "sidebar_stats".into(),
            "§e§lBEDWARS".into(),
            "dummy".into(),
        );
        sb.set_display_slot(1, Some("sidebar_stats".into()));

        sb.set_score("sidebar_stats".into(), "Kills: 5".into(), 2);
        sb.set_score("sidebar_stats".into(), "Coins: 120".into(), 1);
        sb.set_score("sidebar_stats".into(), "§aServer".into(), 3);

        let (title, lines) = sb.get_sidebar_lines().expect("Sidebar should exist");
        assert_eq!(title, "§e§lBEDWARS");
        assert_eq!(lines.len(), 3);
        assert_eq!(lines[0].0, "§aServer");
        assert_eq!(lines[0].1, 3);
        assert_eq!(lines[1].0, "Kills: 5");
        assert_eq!(lines[1].1, 2);
        assert_eq!(lines[2].0, "Coins: 120");
        assert_eq!(lines[2].1, 1);
    }

    #[test]
    fn test_scoreboard_teams() {
        let mut sb = Scoreboard::new();
        let team = sb.create_team("vip_team".into());
        team.prefix = "§a[VIP] ".into();
        team.suffix = "§r".into();

        sb.add_player_to_team("vip_team", "Alex".into());
        assert_eq!(sb.format_player_name("Alex"), "§a[VIP] Alex§r");
        assert_eq!(sb.format_player_name("Steve"), "Steve");

        sb.remove_player_from_team("vip_team", "Alex");
        assert_eq!(sb.format_player_name("Alex"), "Alex");
    }
}
