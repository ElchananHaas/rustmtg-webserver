use crate::game::*;
use common::log::LogEntry;
use common::log::LogPermEntry;

impl Game {
    pub fn log_perm_entry(&self, id: CardId, event: LogPermEntry) {
        let name = if let Some(card) = self.cards.get(id) {
            card.name.clone()
        } else {
            String::from("")
        };
        self.get_log().push(LogEntry::PermEntry { id, name, event });
    }
    pub fn get_log(&self) -> std::sync::MutexGuard<'_, Vec<LogEntry>> {
        self.log.as_ref().lock().unwrap()
    }
}
