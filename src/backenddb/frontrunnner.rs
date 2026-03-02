use sea_orm::DbErr;

use crate::{backenddb::entrys::rebuilt::ActiveModel, snowgrave::find_point_distance::check_pass};
use std::{collections::{HashMap, HashSet}, hash::Hash};



pub fn find_disagreeing_indexes<T>(
    data: &[T],
    exclude: &HashSet<usize>,
) -> Option<(T, Vec<usize>)>
where
    T: Eq + Hash + Clone,
{
    let mut counts: HashMap<&T, usize> = HashMap::new();

    for (i, v) in data.iter().enumerate() {
        if exclude.contains(&i) {
            continue;
        }
        *counts.entry(v).or_insert(0) += 1;
    }

    let mode_ref = counts
        .into_iter()
        .max_by_key(|&(_, c)| c)
        .map(|(v, _)| v)?;

    let disagreeing = data
        .iter()
        .enumerate()
        .filter(|(i, v)| !exclude.contains(i) && *v != mode_ref)
        .map(|(i, _)| i)
        .collect();

    Some((mode_ref.clone(), disagreeing))
}

pub fn average_field<M, F>(
    models: &[M],
    crazy: &mut HashSet<usize>,
    extractor: F,
) -> Result<f32, DbErr>
where
    F: Fn(&M) -> f32,
{
    let values: Vec<f32> = models.iter().map(|m| extractor(m)).collect();

    let res = check_pass(&values);

    crazy.extend(res.failed);

    if res.passed.is_empty() {
        return Err(DbErr::Custom(
            "[FRONTRUNNER][CHECK] No valid numeric values!".to_string(),
        ));
    }

    let total: f32 = res.passed.iter().map(|i| values[*i]).sum();

    Ok(total / res.passed.len() as f32)
}

pub fn consensus_field<M, T, F>(
    models: &[M],
    crazy: &mut HashSet<usize>,
    extractor: F,
) -> Result<T, DbErr>
where
    T: Eq + Hash + Clone,
    F: Fn(&M) -> T,
{
    let values: Vec<T> = models.iter().map(|m| extractor(m)).collect();

    let mode = find_disagreeing_indexes(&values, crazy)
        .ok_or_else(|| {
            DbErr::Custom("[FRONTRUNNER][CHECK] Array is not full!".to_string())
        })?;

    crazy.extend(mode.1);
    Ok(mode.0)
}