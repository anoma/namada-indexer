#[derive(Debug, Clone)]
pub enum OrderByDb {
    Asc,
    Desc,
}

#[macro_export]
macro_rules! asc_desc {
    ($order:expr, $column:expr) => {
        match $order {
            OrderByDb::Asc => Box::new($column.asc()),
            OrderByDb::Desc => Box::new($column.desc()),
        }
    };
}

/// Reverse the order of the column - we use this to get the ranking based on
/// the voting power
#[macro_export]
macro_rules! rev_asc_desc {
    ($order:expr, $column:expr) => {
        match $order {
            OrderByDb::Asc => Box::new($column.desc()),
            OrderByDb::Desc => Box::new($column.asc()),
        }
    };
}
