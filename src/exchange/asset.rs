#[derive(PartialEq, Eq, Hash)]
pub struct Asset {
    pub name: &'static str,
    pub ticker: &'static str,
}