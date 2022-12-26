mod flatten;
mod gen_apply;

pub use flatten::{flatten, flatten_list, flatten_partial};
pub use gen_apply::{apply_cells, generate_cells, monad_apply, monad_cells};
