pub fn viterbi<'a, const S: usize, O>(
    init: &[f32; S],
    trans: &[[f32; S]; S],
    emit: impl Fn(&'a O) -> [f32; S],
    obs: &'a [O],
) -> Vec<usize> {
    assert!(obs.len() > 1);
    let mut prob = vec![[0.0; S]; obs.len()];
    let mut prev = vec![[0; S]; obs.len()];

    let emissions = emit(&obs[0]);
    for s in 0..S {
        prob[0][s] = init[s] * emissions[s];
    }

    for t in 1..obs.len() {
        let emissions = emit(&obs[t]);
        for s in 0..S {
            for r in 0..S {
                let new_prob = prob[t - 1][r] * trans[r][s] * emissions[s];
                if new_prob > prob[t][s] {
                    prob[t][s] = new_prob;
                    prev[t][s] = r;
                }
            }
        }
    }

    let mut path = vec![0; obs.len()];
    let last_prob = prob[obs.len() - 1];
    let max_prob = (0..S)
        .max_by(|&s, &r| last_prob[s].total_cmp(&last_prob[r]))
        .expect("len > 0");

    path[obs.len() - 1] = max_prob;
    for t in (0..obs.len() - 2).rev() {
        path[t] = prev[t + 1][path[t + 1]];
    }
    path
}

#[test]
fn test_viterbi() {
    let path = viterbi(
        &[1.0, 0.0],
        &[[0.4, 0.3], [0.6, 0.7]],
        |&o| if o > 50 { [0.1, 0.9] } else { [0.9, 0.1] },
        &[1, 2, 4, 5, 100, 200, 300, 400, 3, 4],
    );

    assert_eq!(path, &[0, 0, 0, 0, 1, 1, 1, 1, 0, 0]);
}
