const MAX_AREA_PASS: i32 = 70;

pub struct CheckReturn {
    pub passed: Vec<usize>,
    pub failed: Vec<usize>
}



pub fn check_pass(data_points: &[f32]) -> CheckReturn {
    // Pair score with original index
    let mut indexed: Vec<(usize, f32)> =
    data_points
        .iter()
        .enumerate()
        .map(|(i, &v)| (i, v))
        .collect();

    // Sort by score
    indexed.sort_by(|a, b| {
        match (a.1.is_nan(), b.1.is_nan()) {
            (true, true) => std::cmp::Ordering::Equal,
            (true, false) => std::cmp::Ordering::Greater,
            (false, true) => std::cmp::Ordering::Less,
            (false, false) => a.1.partial_cmp(&b.1).unwrap(),
        }
    });

    let n = indexed.len();
    let mut best_start = 0;
    let mut best_end = 0;
    let mut best_len = 0;

    let mut start = 0;
    let mut area = 0.0;

    for end in 0..n - 1 {
        area += indexed[end + 1].1 - indexed[end].1;

        while area > MAX_AREA_PASS as f32 {
            area -= indexed[start + 1].1 - indexed[start].1;
            start += 1;
        }

        let len = end + 1 - start + 1;
        if len > best_len {
            best_len = len;
            best_start = start;
            best_end = end + 1;
        }
    }

    // Convert back to ORIGINAL indexes
    let passed: Vec<usize> =
        indexed[best_start..=best_end]
            .iter()
            .map(|&(orig_index, _)| orig_index)
            .collect();

    let mut failed = Vec::new();
    for (orig_index, _) in indexed.iter() {
        if !passed.contains(orig_index) {
            failed.push(*orig_index);
        }
    }

    CheckReturn { passed, failed }
}