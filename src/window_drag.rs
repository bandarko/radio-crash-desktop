//! Physical desktop-coordinate tracking for the Windows borderless receiver.
//! No native move/size modal loop and no conversion through egui's logical points.
#[derive(Default)]
pub struct DragTrack {
    anchor: Option<([i32; 2], [i32; 2])>,
    last: [i32; 2],
}
impl DragTrack {
    pub fn begin(&mut self, cursor: [i32; 2], origin: [i32; 2]) {
        self.anchor = Some((cursor, origin));
        self.last = origin;
    }
    pub fn active(&self) -> bool {
        self.anchor.is_some()
    }
    pub fn cancel(&mut self) {
        self.anchor = None;
    }
    pub fn step(&mut self, cursor: [i32; 2], held: bool) -> Option<[i32; 2]> {
        if !held {
            self.cancel();
            return None;
        }
        let (start, origin) = self.anchor?;
        let target = std::array::from_fn(|i| {
            (origin[i] as i64 + cursor[i] as i64 - start[i] as i64)
                .clamp(i32::MIN as i64, i32::MAX as i64) as i32
        });
        if target == self.last {
            return None;
        }
        self.last = target;
        Some(target)
    }
}
#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn physical_grab_offset_survives_negative_desktop_coordinates_and_monitor_crossings() {
        let mut drag = DragTrack::default();
        drag.begin([350, 240], [100, 160]);
        for cursor in [
            [350, 240],
            [-1300, -600],
            [3000, 900],
            [-20, 10],
            [350, 240],
        ] {
            let position = drag.step(cursor, true).unwrap_or(drag.last);
            assert_eq!(
                [cursor[0] - position[0], cursor[1] - position[1]],
                [250, 80]
            );
        }
        // Input and output are physical desktop pixels, with no 100/125/150/200% factor.
        assert!(drag.active());
    }
    #[test]
    fn stationary_hold_is_nonblocking_release_and_focus_loss_stop_the_drag() {
        let mut drag = DragTrack::default();
        drag.begin([400, 300], [200, 100]);
        for _ in 0..1000 {
            assert_eq!(drag.step([400, 300], true), None);
        }
        assert_eq!(drag.step([420, 330], true), Some([220, 130]));
        assert_eq!(drag.step([420, 330], true), None);
        assert_eq!(drag.step([4000, 3000], false), None);
        assert!(!drag.active());
        assert_eq!(drag.step([5000, 4000], true), None);
        drag.begin([-40, 30], [-200, 0]);
        assert_eq!(drag.step([-20, 40], true), Some([-180, 10]));
        drag.cancel();
        assert!(!drag.active());
    }
    #[test]
    fn extreme_virtual_desktop_coordinates_do_not_overflow() {
        let mut drag = DragTrack::default();
        drag.begin([i32::MIN, i32::MAX], [100, -100]);
        assert_eq!(
            drag.step([i32::MAX, i32::MIN], true),
            Some([i32::MAX, i32::MIN])
        );
    }
}
