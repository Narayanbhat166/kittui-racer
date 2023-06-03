use tui::widgets::ListState;

pub struct StatefulList<T> {
    pub state: ListState,
    pub items: Vec<T>,
}

impl<T> StatefulList<T> {
    pub fn with_items(items: Vec<T>) -> StatefulList<T> {
        let mut state = ListState::default();
        state.select(items.first().map(|_| 0 as usize));
        StatefulList { state, items }
    }

    pub fn add_items(&mut self, items: Vec<T>) {
        self.items.extend(items)
    }

    pub fn clear_and_insert_items(&mut self, items: Vec<T>) {
        self.items.clear();
        self.items.extend(items);
    }

    pub fn get_selected_item(&self) -> Option<&T> {
        self.state
            .selected()
            .and_then(|selected_index| self.items.get(selected_index))
    }

    pub fn new() -> Self {
        Self {
            state: ListState::default(),
            items: vec![],
        }
    }

    pub fn add_item(&mut self, item: T) {
        self.items.push(item)
    }

    pub fn next(&mut self) {
        let i = match self.state.selected() {
            Some(i) => {
                if i >= self.items.len() - 1 {
                    0
                } else {
                    i + 1
                }
            }
            None => 0,
        };
        self.state.select(Some(i));
    }

    pub fn previous(&mut self) {
        let i = match self.state.selected() {
            Some(i) => {
                if i == 0 {
                    self.items.len() - 1
                } else {
                    i - 1
                }
            }
            None => 0,
        };
        self.state.select(Some(i));
    }

    pub fn unselect(&mut self) {
        self.state.select(None);
    }
}
