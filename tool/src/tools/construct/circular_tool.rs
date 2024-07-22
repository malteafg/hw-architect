use super::curve_tool_spec::{CurveAction, CurveActionResult, CurveToolSpecInternal};

use curves::{Circular, Curve, CurveInfo};
use utils::{DirXZ, PosOrLoc};

#[derive(Default)]
pub struct CircularTool {
    sel_dir: Option<DirXZ>,
}

impl CurveToolSpecInternal for CircularTool {
    fn left_click(&mut self, first: PosOrLoc, last: PosOrLoc) -> CurveActionResult {
        if self.sel_dir.is_none() && first.is_pos() {
            self.sel_dir = Some((last.pos() - first.pos()).into());
            self.compute_curve(first, last)
        } else {
            match self.compute_curve(first, last) {
                Ok(CurveAction::Render(curve, _curve_info)) => Ok(CurveAction::Construct(curve)),
                curve_result => curve_result,
            }
        }
    }

    fn right_click(&mut self, first: PosOrLoc, last: PosOrLoc) -> CurveActionResult {
        if self.sel_dir.is_some() {
            self.sel_dir = None;
            self.compute_curve(first, last)
        } else {
            Ok(CurveAction::Nothing)
        }
    }

    fn compute_curve(&mut self, mut first: PosOrLoc, last: PosOrLoc) -> CurveActionResult {
        use PosOrLoc::*;
        if let Some(dir) = self.sel_dir {
            first = Loc(utils::Loc::new(first.pos(), dir));
        }

        match (first, last) {
            (Pos(first_pos), Pos(last_pos)) => {
                let dir = last_pos - first_pos;
                Ok(CurveAction::Direction(
                    utils::Loc::new(first_pos, dir.into()),
                    last_pos,
                ))
            }
            (Loc(first), Pos(last_pos)) => {
                let (curve, info) = Curve::<Circular>::from_first_locked(first, last_pos);
                Ok(CurveAction::Render(curve.into(), info))
            }
            (Pos(first_pos), Loc(last)) => Curve::<Circular>::from_last_locked(first_pos, last)
                .map(|c| CurveAction::Render(c.into(), CurveInfo::Satisfied)),
            (Loc(first), Loc(last)) => Curve::<Circular>::from_both_locked(first, last)
                .map(|c| CurveAction::Render(c.into(), CurveInfo::Satisfied)),
        }
    }
}
