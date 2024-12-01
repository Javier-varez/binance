#[derive(serde::Deserialize, Clone, Copy, PartialEq, Eq)]
#[repr(transparent)]
pub struct LazyF64<'a>(pub &'a str);

#[derive(serde::Deserialize, Clone, Copy, PartialEq, Eq)]
#[repr(transparent)]
pub struct LazyU64<'a>(pub &'a str);

impl TryInto<u64> for LazyU64<'_> {
    type Error = ();
    fn try_into(self) -> Result<u64, Self::Error> {
        self.0.parse().map_err(|_| ())
    }
}

impl TryInto<f64> for LazyF64<'_> {
    type Error = ();
    fn try_into(self) -> Result<f64, Self::Error> {
        self.0.parse().map_err(|_| ())
    }
}

impl std::fmt::Debug for LazyF64<'_> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let result: Result<f64, _> = (*self).try_into();
        write!(f, "{:?}", result)
    }
}

impl std::fmt::Debug for LazyU64<'_> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let result: Result<u64, _> = (*self).try_into();
        write!(f, "{:?}", result)
    }
}
