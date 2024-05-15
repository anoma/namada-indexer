use std::ops::Rem;

use serde::Serialize;

use crate::constant::ITEM_PER_PAGE;

#[derive(Clone, Debug, Serialize)]
pub struct PaginatedResponse<T: Serialize> {
    pub data: T,
    pub pagination: Pagination,
}

#[derive(Clone, Debug, Serialize)]
pub struct Pagination {
    pub page: u64,
    pub per_page: u64,
    pub total_pages: u64,
    pub total_items: u64,
}

impl<T> PaginatedResponse<T>
where
    T: Serialize,
{
    pub fn new(data: T, page: u64, total_items: u64) -> Self {
        Self {
            data,
            pagination: Pagination {
                page,
                per_page: ITEM_PER_PAGE,
                total_pages: if total_items == 0 {
                    0
                } else if total_items.rem(ITEM_PER_PAGE) == 0 {
                    total_items / ITEM_PER_PAGE
                } else {
                    (total_items / ITEM_PER_PAGE) + 1
                },
                total_items,
            },
        }
    }
}
