use hashbrown::HashMap;
use uuid::Uuid;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TabEntry {
    pub uuid: Uuid,
    pub name: String,
    pub game_mode: i32,
    pub ping: i32,
    pub display_name: Option<String>,
}

#[derive(Debug, Clone, Default)]
pub struct TabList {
    pub entries: HashMap<Uuid, TabEntry>,
    pub header: Option<String>,
    pub footer: Option<String>,
}

impl TabList {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn add_or_update(
        &mut self,
        uuid: Uuid,
        name: String,
        game_mode: i32,
        ping: i32,
        display_name: Option<String>,
    ) {
        self.entries.insert(
            uuid,
            TabEntry {
                uuid,
                name,
                game_mode,
                ping,
                display_name,
            },
        );
    }

    pub fn update_game_mode(&mut self, uuid: &Uuid, game_mode: i32) {
        if let Some(entry) = self.entries.get_mut(uuid) {
            entry.game_mode = game_mode;
        }
    }

    pub fn update_ping(&mut self, uuid: &Uuid, ping: i32) {
        if let Some(entry) = self.entries.get_mut(uuid) {
            entry.ping = ping;
        }
    }

    pub fn update_display_name(&mut self, uuid: &Uuid, display_name: Option<String>) {
        if let Some(entry) = self.entries.get_mut(uuid) {
            entry.display_name = display_name;
        }
    }

    pub fn remove_player(&mut self, uuid: &Uuid) {
        self.entries.remove(uuid);
    }

    pub fn get_entry(&self, uuid: &Uuid) -> Option<&TabEntry> {
        self.entries.get(uuid)
    }

    pub fn get_sorted_entries(&self) -> Vec<&TabEntry> {
        let mut list: Vec<&TabEntry> = self.entries.values().collect();
        list.sort_by(|a, b| {
            let name_a = a.display_name.as_deref().unwrap_or(&a.name);
            let name_b = b.display_name.as_deref().unwrap_or(&b.name);
            name_a.cmp(name_b)
        });
        list
    }

    pub fn clear(&mut self) {
        self.entries.clear();
        self.header = None;
        self.footer = None;
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_tab_list_operations() {
        let mut tab = TabList::new();
        let u1 = Uuid::new_v4();
        let u2 = Uuid::new_v4();

        tab.add_or_update(u1, "Steve".into(), 0, 45, None);
        tab.add_or_update(u2, "Alex".into(), 1, 20, Some("§bAlex".into()));

        let sorted = tab.get_sorted_entries();
        assert_eq!(sorted.len(), 2);

        tab.update_ping(&u1, 15);
        assert_eq!(tab.entries.get(&u1).unwrap().ping, 15);

        tab.remove_player(&u1);
        assert_eq!(tab.entries.len(), 1);
    }
}
