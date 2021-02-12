use std::hash::Hash;
use std::collections::HashMap;
use std::convert::identity;

pub(crate) fn group_by<I, F, K: Hash + Eq + Clone, V>(vec: I, key_by: F) -> HashMap<K, Vec<V>>
    where
        F: Fn(&V) -> K,
        I: IntoIterator<Item = V>
{
    map_group_by(vec, key_by, identity)
}

pub(crate) fn map_group_by<I, F, G, K: Hash + Eq + Clone, O, V>(vec: I, key_by: F, mut map: G) -> HashMap<K, Vec<V>>
    where
        F: Fn(&O) -> K,
        G: FnMut(O) -> V,
        I: IntoIterator<Item = O>
{
    let mut out = HashMap::new();
    for e in vec {
        let key = key_by(&e);
        if !out.contains_key(&key) {
            out.insert(key.clone(), Vec::new());
        }
        out.get_mut(&key).unwrap().push(map(e));
    }
    out
}

macro_rules! guard_syn {
    ($exp:expr) => {
        match $exp {
            Ok(t) => t,
            Err(e) => return e.to_compile_error().into(),
        }
    }
}
