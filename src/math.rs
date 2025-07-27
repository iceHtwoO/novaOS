pub fn polar_to_cartesian(r: f32, theta_rad: f32) -> (f32, f32) {
    let x = r * libm::cosf(theta_rad);
    let y = r * libm::sinf(theta_rad);
    (x, y)
}
