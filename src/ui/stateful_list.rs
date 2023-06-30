use tui::widgets::ListState;

pub struct StatefulList<T> {
    pub state: ListState,
    pub items: Vec<T>,
}

impl<T> StatefulList<T> {
    pub fn with_items(items: Vec<T>) -> StatefulList<T> {
        let mut state = ListState::default();
        state.select(items.first().map(|_| 0_usize));
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

    pub fn add_item(&mut self, item: T) {
        self.items.push(item)
    }

    /// Select the next item in the list, in cyclic manner
    pub fn next(&mut self) {
        // Check if there are any elements in the list and then proceed
        if !self.items.is_empty() {
            let next_selection = self
                .state
                .selected()
                .map(|selected_index| {
                    // Check if selection is at the end of the list
                    (selected_index + 1 == self.items.len())
                        .then_some(0) // If so, set the selection back to index 0
                        .unwrap_or(selected_index.saturating_add(1)) // else, increment the selection
                })
                .unwrap_or(0); // If no index is selected previously, select 0th index
            self.state.select(Some(next_selection));
        }
    }

    /// Select the previous element in the list, in cyclic manner.
    pub fn previous(&mut self) {
        // Check if there are any elements in the list and then proceed
        if !self.items.is_empty() {
            let select_previous = self
                .state
                .selected()
                .map(|selected_index| {
                    // Check if selection is at the beniggnningnngnnngggg of the list
                    (selected_index == 0)
                        .then_some(self.items.len() - 1) // If so, cycle back to last
                        .unwrap_or(selected_index.saturating_sub(1)) // else, decrement the selection
                })
                .unwrap_or(0); // If no index is selected previously, select 0th index
            self.state.select(Some(select_previous));
        }
    }

    pub fn unselect(&mut self) {
        self.state.select(None);
    }
}
