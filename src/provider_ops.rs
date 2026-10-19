pub(crate) struct Op {
    pub id: &'static str,
    pub native: &'static str,
    pub words: &'static [&'static str],
    pub effect: &'static str,
    pub mode: Mode,
    pub extra: &'static [(&'static str, bool)],
}
#[derive(Clone, Copy)]
pub(crate) enum Mode {
    Read,
    Plan(&'static str),
    Apply,
    Reconcile,
}
macro_rules! op {
    ($id:literal,$native:literal,$words:expr,$effect:literal,$mode:expr,$extra:expr) => {
        Op {
            id: concat!("wordpress.", $id),
            native: $native,
            words: $words,
            effect: $effect,
            mode: $mode,
            extra: $extra,
        }
    };
}
#[path = "provider_ops/plans.rs"]
mod plans;
#[path = "provider_ops/reads.rs"]
mod reads;
#[path = "provider_ops/writes.rs"]
mod writes;
pub(crate) fn ops() -> impl Iterator<Item = &'static Op> {
    reads::OPS.iter().chain(plans::OPS).chain(writes::OPS)
}
