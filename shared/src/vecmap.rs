use std::iter::FromIterator;

use serde::{Deserialize, Deserializer, Serialize, Serializer};

/// Seralizes a map as a vec.
pub fn serialize<'a, T, K, V, S>(target: T, ser: S) -> Result<S::Ok, S::Error>
where
    S: Serializer,
    T: IntoIterator<Item = (&'a K, &'a V)>,
    K: Serialize + 'a,
    V: Serialize + 'a,
{
    let container: Vec<_> = target.into_iter().collect();

    serde::Serialize::serialize(&container, ser)
}
/// Deserializes a map as a vec.
pub fn deserialize<'de, T, K, V, D>(des: D) -> Result<T, D::Error>
where
    D: Deserializer<'de>,
    T: FromIterator<(K, V)>,
    K: Deserialize<'de>,
    V: Deserialize<'de>,
{
    let container: Vec<_> = serde::Deserialize::deserialize(des)?;

    Ok(T::from_iter(container))
}
