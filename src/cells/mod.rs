mod flatten;
mod gen_apply;

pub use flatten::{flatten_list, flatten_list_cow, flatten_maintaining_prefix};
pub use gen_apply::{apply_cells, generate_cells, monad_apply, monad_cells};
