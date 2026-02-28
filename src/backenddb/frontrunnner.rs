use crate::backenddb::entrys::rebuilt::ActiveModel;
use std::{collections::HashMap, hash::Hash};



pub fn find_disagreeing_indexes<T>(data: &[T]) -> Option<(T, Vec<usize>)>
where
    T: Eq + Hash + Clone,
{
    if data.is_empty() {
        return None;
    }

    let mut counts: HashMap<&T, usize> = HashMap::new();

    for v in data {
        *counts.entry(v).or_insert(0) += 1;
    }

    let mode_ref = counts
        .into_iter()
        .max_by_key(|&(_, c)| c)
        .map(|(v, _)| v)?;

    let disagreeing = data
        .iter()
        .enumerate()
        .filter(|(_, v)| *v != mode_ref)
        .map(|(i, _)| i)
        .collect();

    Some((mode_ref.clone(), disagreeing))
}
