// Represents maximum number of parameters that we can insert into postgres in
// one go. To get the number of rows that we can insert in one chunk, we have to
// divide MAX_PARAM_SIZE by the number of columns in the given table.
pub const MAX_PARAM_SIZE: u16 = u16::MAX;
