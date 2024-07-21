use super::curve_tool_spec::{CurveAction, CurveActionResult, CurveToolSpecInternal};

use curves::{Curve, CurveInfo, Straight};
use utils::PosOrLoc;

#[derive(Default)]
pub struct StraightTool;

impl CurveToolSpecInternal for StraightTool {
    fn left_click(&mut self, first: PosOrLoc, last: PosOrLoc) -> CurveActionResult {
        match self.compute_curve(first, last) {
            Ok(CurveAction::Render(curve, _curve_info)) => Ok(CurveAction::Construct(curve)),
            curve_result => curve_result,
        }
    }

    fn right_click(&mut self, _first: PosOrLoc, _last: PosOrLoc) -> CurveActionResult {
        Ok(CurveAction::Nothing)
    }

    fn compute_curve(&mut self, first: PosOrLoc, last: PosOrLoc) -> CurveActionResult {
        use PosOrLoc::*;
        match (first, last) {
            (Pos(first_pos), Pos(last_pos)) => {
                Ok(Curve::<Straight>::from_free(first_pos, last_pos).into())
            }
            (Loc(first), Pos(last_pos)) => {
                Ok(Curve::<Straight>::from_first_locked(first.into(), last_pos).into())
            }
            (Pos(first_pos), Loc(last)) => Curve::<Straight>::from_last_locked(first_pos, last)
                .map(|c| CurveAction::Render(c.into(), CurveInfo::Satisfied)),
            (Loc(first), Loc(last)) => Curve::<Straight>::from_both_locked(first, last)
                .map(|c| CurveAction::Render(c.into(), CurveInfo::Satisfied)),
        }
    }
}
