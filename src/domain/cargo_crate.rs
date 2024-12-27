#[derive(Debug, Eq, Hash, PartialEq)]
#[cfg_attr(test, derive(serde::Serialize))]
pub struct CargoCrate {
    pub name: String,
    pub version: String,
}
