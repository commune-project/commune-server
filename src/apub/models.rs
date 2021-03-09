pub mod activities;
pub mod actor;

pub use activities::*;
pub use actor::*;

pub fn empty_string_or_none(value: String) -> Option<String> {
    if value == "" {
        None
    } else {
        Some(value)
    }
}

use serde::Deserialize;

#[derive(Deserialize, Debug, Eq, PartialEq)]
pub struct PagedCollection {
    page: Option<i64>,
}

impl PagedCollection {
    pub fn is_paged(&self) -> bool {
        self.page.is_some()
    }

    pub fn has_next(&self, total_items: i64, page_size: i64) -> bool {
        match self.page {
            None => false,
            Some(page) => total_items > page_size * page,
        }
    }

    pub fn has_prev(&self) -> bool {
        match self.page {
            None => false,
            Some(page) => page > 1,
        }
    }

    pub fn next_page_number(&self) -> i64 {
        match self.page {
            None => 0,
            Some(page) => page + 1,
        }
    }

    pub fn prev_page_number(&self) -> i64 {
        match self.page {
            None => 0,
            Some(page) => page - 1,
        }
    }

    pub fn page_number(&self) -> i64 {
        match self.page {
            None => 0,
            Some(page) => page,
        }
    }
}
