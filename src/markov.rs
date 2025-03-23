pub fn viterbi<'a, const S: usize, O>(
    init: &[f32; S],
    trans: &[[f32; S]; S],
    end: &[f32; S],
    emit: impl Fn(&'a O) -> [f32; S],
    obs: &'a [O],
) -> Vec<usize> {
    assert!(obs.len() > 0);
    let mut prob = vec![[0.0; S]; obs.len()];
    let mut prev = vec![[0; S]; obs.len()];

    let emissions = emit(&obs[0]);
    dbg!(&emissions);
    for s in 0..S {
        prob[0][s] = init[s] * emissions[s];
    }
    dbg!(&prob);

    for t in 1..obs.len() {
        let emissions = emit(&obs[t]);
        dbg!(&emissions);
        for s in 0..S {
            for r in 0..S {
                let new_prob = prob[t - 1][r] * trans[r][s] * emissions[s];
                if s == 9 {
                    dbg!(t, s, r, prob[t][s], new_prob);
                }
                if new_prob > prob[t][s] {
                    if s == 9 {
                        dbg!("updated prob");
                    }
                    prob[t][s] = new_prob;
                    prev[t][s] = r;
                }
            }
        }
    }

    let last_prob = prob[obs.len() - 1];
    let mut end_prob = 0.0;
    let mut end_prev = 0;
    for r in 0..S {
        let new_prob = last_prob[r] * end[r];
        if new_prob > end_prob {
            end_prob = new_prob;
            end_prev = r;
        }
    }

    dbg!(&prob);
    dbg!(&prev);

    let mut path = vec![0; obs.len()];
    dbg!(end_prev);

    path[obs.len() - 1] = end_prev;
    for t in (0..obs.len() - 1).rev() {
        path[t] = prev[t + 1][path[t + 1]];
    }
    path
}

#[test]
fn test_viterbi() {
    // basic tests that checks that the observations are reflected
    let path = viterbi(
        &[1.0, 0.0],
        &[[0.5, 0.4], [0.4, 0.5]],
        &[0.1, 0.1],
        |&o| if o > 50 { [0.1, 0.9] } else { [0.9, 0.1] },
        &[1, 2, 4, 5, 100, 200, 300, 400, 3, 4],
    );

    assert_eq!(path, &[0, 0, 0, 0, 1, 1, 1, 1, 0, 0]);
}

#[test]
fn test_viterbi_endstate() {
    // Staying at state 0 should be the only possible inference
    // if the END state is working properly
    let path = viterbi(
        &[1.0, 0.0, 0.0], // always start with state 0
        &[
            [0.0, 0.5, 0.5], // state 0: to state 1 or 2
            [0.0, 0.3, 0.3], // state 1: decays little by little towards state 2 or END
            [0.0, 0.0, 1.0], // state 2: always stay at state 2, never go to END
        ],
        &[0.0, 0.4, 0.0],        // The END state is reachable only from state 1
        |&_| [0.33, 0.33, 0.34], // no bias from observations
        &[8, 8, 8, 8, 8, 8, 8, 8, 8, 8, 8, 8], // any observation goes
    );

    assert_eq!(path, &[0, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1]);
}
