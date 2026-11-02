pub fn desk_bounds(
    x: f64,
    y: f64,
    available_width: f64,
    available_height: f64,
) -> (f64, f64, f64, f64) {
    let width = (available_width - 32.0).clamp(1.0, 940.0);
    let height = (available_height - 32.0).clamp(1.0, 730.0);
    (
        x + (available_width - width) / 2.0,
        y + (available_height - height) / 2.0,
        width,
        height,
    )
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn desk_fits_small_and_negative_origin_work_areas() {
        assert_eq!(
            desk_bounds(-1280.0, 0.0, 1280.0, 720.0),
            (-1110.0, 16.0, 940.0, 688.0)
        );
        let (x, y, w, h) = desk_bounds(0.0, 0.0, 640.0, 480.0);
        assert_eq!((x, y, w, h), (16.0, 16.0, 608.0, 448.0));
    }
}
