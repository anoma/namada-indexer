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

pub fn epoch_progress(current_block: i32, min_num_of_blocks: i32) -> f64 {
    // Calculate the block in the current epoch
    let block_in_current_epoch = current_block % min_num_of_blocks;

    // Calculate how much into the epoch we are
    block_in_current_epoch as f64 / min_num_of_blocks as f64
}

// Calculate the time between current epoch and arbitrary epoch
pub fn time_between_epochs(
    current_epoch_progress: f64,
    current_epoch: i32,
    to_epoch: i32,
    epoch_duration: i32,
) -> i32 {
    let epoch_time = (to_epoch - current_epoch) * epoch_duration;
    let extra_time = current_epoch_progress * epoch_duration as f64;

    epoch_time - extra_time as i32
}
