use teloxide::types::{InlineKeyboardButton, InlineKeyboardMarkup};

pub struct Paginator<'a, T> {
    items: &'a [T],
    per_page: usize,
    columns: usize,
    current_page: usize,
    bottom_rows: Vec<Vec<InlineKeyboardButton>>,
    callback_prefix: String,
    total_items: Option<usize>,
    callback_formatter: Option<Box<dyn Fn(usize) -> String + 'a>>,
}

pub trait ItemsBuild<'a, T> {
    fn build<F>(&self, button_mapper: F) -> InlineKeyboardMarkup
    where
        F: Fn(&T) -> InlineKeyboardButton;
}

pub trait FrameBuild {
    fn build(&self) -> InlineKeyboardMarkup;
}

impl<'a> Paginator<'a, ()> {
    pub fn new(module_key: &'a str, total_items: usize) -> Self {
        Self {
            items: &[],
            total_items: Some(total_items),
            per_page: 1,
            columns: 1,
            current_page: 0,
            bottom_rows: Vec::new(),
            callback_prefix: module_key.to_string(),
            callback_formatter: None,
        }
    }
}

impl<'a, T> Paginator<'a, T> {
    pub fn from(module_key: &'a str, items: &'a [T]) -> Self {
        Self {
            items,
            per_page: 12,
            columns: 3,
            current_page: 0,
            bottom_rows: Vec::new(),
            callback_prefix: module_key.to_string(),
            total_items: None,
            callback_formatter: None,
        }
    }
}

impl<'a, T> ItemsBuild<'a, T> for Paginator<'a, T> {
    fn build<F>(&self, button_mapper: F) -> InlineKeyboardMarkup
    where
        F: Fn(&T) -> InlineKeyboardButton,
    {
        let total_items = self.total_items.unwrap_or(self.items.len());
        if total_items == 0 {
            return InlineKeyboardMarkup::new(self.bottom_rows.clone());
        }

        // let total_pages = (total_items + self.per_page - 1) / self.per_page;
        let total_pages = total_items.div_ceil(self.per_page);
        let page = self.current_page.min(total_pages - 1);

        let mut keyboard: Vec<Vec<InlineKeyboardButton>> = if !self.items.is_empty() {
            let start = page * self.per_page;
            let end = (start + self.per_page).min(self.items.len());

            let page_items = &self.items[start..end];

            page_items
                .iter()
                .map(button_mapper)
                .collect::<Vec<_>>()
                .chunks(self.columns)
                .map(|chunk| chunk.to_vec())
                .collect()
        } else {
            Vec::new()
        };

        let mut nav_row = Vec::new();
        if page > 0 {
            let prev_page = page - 1;

            nav_row.push(InlineKeyboardButton::callback(
                "⬅️",
                self.callback_formatter.as_ref().map_or_else(
                    || format!("{}:page:{}", self.callback_prefix, prev_page),
                    |f| f(prev_page),
                ),
            ));
        }

        nav_row.push(InlineKeyboardButton::callback(
            format!("{}/{}", page + 1, total_pages),
            "noop",
        ));

        if page + 1 < total_pages {
            let next_page = page + 1;

            nav_row.push(InlineKeyboardButton::callback(
                "➡️",
                self.callback_formatter.as_ref().map_or_else(
                    || format!("{}:page:{}", self.callback_prefix, next_page),
                    |f| f(next_page),
                ),
            ));
        }

        if nav_row.len() > 1 {
            keyboard.push(nav_row);
        }

        keyboard.extend(self.bottom_rows.clone());

        InlineKeyboardMarkup::new(keyboard)
    }
}

impl<'a> FrameBuild for Paginator<'a, ()> {
    fn build(&self) -> InlineKeyboardMarkup {
        ItemsBuild::build(self, |_| unreachable!())
    }
}

impl<'a, T> Paginator<'a, T> {
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

    pub fn set_callback_formatter<F>(mut self, formatter: F) -> Self
    where
        F: Fn(usize) -> String + 'a,
    {
        self.callback_formatter = Some(Box::new(formatter));
        self
    }

    pub fn set_total_items(mut self, total: usize) -> Self {
        self.total_items = Some(total);
        self
    }
}
