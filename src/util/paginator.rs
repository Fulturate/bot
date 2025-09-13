use teloxide::types::{InlineKeyboardButton, InlineKeyboardMarkup};

pub struct Paginator<'a, T> {
    module_key: &'a str,

    items: &'a [T],

    per_page: usize,

    columns: usize,

    current_page: usize,

    bottom_rows: Vec<Vec<InlineKeyboardButton>>,

    callback_prefix: String,
}

impl<'a, T> Paginator<'a, T> {
    pub fn new(module_key: &'a str, items: &'a [T]) -> Self {
        Self {
            module_key,
            items,
            per_page: 12,
            columns: 3,
            current_page: 0,
            bottom_rows: Vec::new(),
            callback_prefix: module_key.to_string(),
        }
    }

    pub fn per_page(mut self, per_page: usize) -> Self {
        self.per_page = per_page;
        self
    }

    pub fn columns(mut self, columns: usize) -> Self {
        self.columns = columns;
        self
    }

    pub fn current_page(mut self, page: usize) -> Self {
        self.current_page = page;
        self
    }

    pub fn add_bottom_row(mut self, row: Vec<InlineKeyboardButton>) -> Self {
        self.bottom_rows.push(row);
        self
    }

    pub fn set_callback_prefix(mut self, prefix: String) -> Self {
        self.callback_prefix = prefix;
        self
    }

    pub fn build<F>(&self, button_mapper: F) -> InlineKeyboardMarkup
    where
        F: Fn(&T) -> InlineKeyboardButton,
    {
        let total_items = self.items.len();
        if total_items == 0 {
            return InlineKeyboardMarkup::new(self.bottom_rows.clone());
        }

        let total_pages = (total_items + self.per_page - 1) / self.per_page;
        let page = self.current_page.min(total_pages - 1);

        let start = page * self.per_page;
        let end = (start + self.per_page).min(total_items);
        let page_items = &self.items[start..end];

        let mut keyboard: Vec<Vec<InlineKeyboardButton>> = page_items
            .iter()
            .map(button_mapper)
            .collect::<Vec<_>>()
            .chunks(self.columns)
            .map(|chunk| chunk.to_vec())
            .collect();

        let mut nav_row = Vec::new();
        if page > 0 {
            nav_row.push(InlineKeyboardButton::callback(
                "⬅️",
                format!("{}:page:{}", self.module_key, page - 1),
            ));
        }

        nav_row.push(InlineKeyboardButton::callback(
            format!("{}/{}", page + 1, total_pages),
            "noop",
        ));

        if page + 1 < total_pages {
            nav_row.push(InlineKeyboardButton::callback(
                "➡️",
                format!("{}:page:{}", self.module_key, page + 1),
            ));
        }

        if nav_row.len() > 1 {
            keyboard.push(nav_row);
        }

        keyboard.extend(self.bottom_rows.clone());

        InlineKeyboardMarkup::new(keyboard)
    }
}
