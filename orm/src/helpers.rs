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
