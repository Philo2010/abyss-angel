use sea_orm::DbErr;

use crate::snowgrave::find_point_distance::check_pass;
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

    let max_count = counts.values().copied().max()?;
    let tied: Vec<&T> = counts
        .iter()
        .filter(|&(_, &c)| c == max_count)
        .map(|(v, _)| *v)
        .collect();

    // If there's a tie, no clear consensus — mark all non-excluded as disagreeing
    if tied.len() > 1 {
        let all_indexes = data
            .iter()
            .enumerate()
            .filter(|(i, _)| !exclude.contains(i))
            .map(|(i, _)| i)
            .collect();
        return Some((tied[0].clone(), all_indexes));
    }

    let mode_ref = tied[0];

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
    field_name: &str,
) -> Result<f32, DbErr>
where
    F: Fn(&M) -> f32,
{
    let values: Vec<f32> = models.iter().map(|m| extractor(m)).collect();

    let res = check_pass(&values);

    let before = crazy.len();
    crazy.extend(res.failed.iter());
    if crazy.len() != before {
        eprintln!("[average_field] {field_name}: added {:?} → crazy={:?}", res.failed, crazy);
    }

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
    field_name: &str,
) -> Option<T>
where
    T: Eq + Hash + Clone,
    F: Fn(&M) -> T,
{
    let values: Vec<T> = models.iter().map(|m| extractor(m)).collect();

    let mode = match find_disagreeing_indexes(&values, crazy) {
        Some(a) => {
            a
        },
        None => {
            return None;
        },
    };

    let before = crazy.len();
    crazy.extend(mode.1.iter());
    if crazy.len() != before {
        eprintln!("[consensus_field] {field_name}: added {:?} → crazy={:?}", mode.1, crazy);
    }
    Some(mode.0)
}