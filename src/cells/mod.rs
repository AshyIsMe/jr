mod flatten;
mod gen_apply;

pub use flatten::fill_promote_reshape;
pub use gen_apply::{apply_cells, generate_cells, monad_apply, monad_cells};
