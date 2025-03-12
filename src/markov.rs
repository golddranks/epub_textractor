const TRANS: [[f32; 2]; 2] = [
    [0.4, 0.3],
    [0.6, 0.7]
];

const EMIT: [f32; 2] = [
    0.4,
    0.6
];

pub fn viterbi<const N: usize, O>(trans: [[f32; N]; N], emit: [f32; N], seq: &[O]) {

}
