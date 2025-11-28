/// Normalize a 3D vector stored as `[f32; 3]`. Returns the input direction if its
/// length is zero, falling back to pointing down the Y axis.
#[inline]
pub fn normalize_vec3(vec: [f32; 3]) -> [f32; 3] {
    let len_sq = vec[0] * vec[0] + vec[1] * vec[1] + vec[2] * vec[2];
    if len_sq == 0.0 {
        return [0.0, -1.0, 0.0];
    }
    let inv_len = len_sq.sqrt().recip();
    [vec[0] * inv_len, vec[1] * inv_len, vec[2] * inv_len]
}

#[cfg(test)]
mod tests {
    use super::normalize_vec3;

    #[test]
    fn normalizes_non_zero_vector() {
        let v = normalize_vec3([3.0, 0.0, 4.0]);
        assert!((v[0] - 0.6).abs() < 1e-6);
        assert_eq!(v[1], 0.0);
        assert!((v[2] - 0.8).abs() < 1e-6);
    }

    #[test]
    fn falls_back_for_zero_vector() {
        assert_eq!(normalize_vec3([0.0, 0.0, 0.0]), [0.0, -1.0, 0.0]);
    }
}
