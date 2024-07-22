use super::{circular_tool::CircularTool, straight_tool::StraightTool};

use curves::{
    Circular, CompositeCurveSum, Curve, CurveError, CurveInfo, CurveSpec, CurveSum, Straight,
};
use utils::{Loc, PosOrLoc};
use world_api::SnapConfig;

use std::marker::PhantomData;

use enum_dispatch::enum_dispatch;
use glam::Vec3;

#[derive(Debug, Clone)]
pub enum EndPoint {
    /// Position from which to build from. This must not be projected.
    New(Vec3),
    /// Location of the snapconfig to build from
    Old(SnapConfig),
}

impl From<EndPoint> for PosOrLoc {
    fn from(value: EndPoint) -> Self {
        match value {
            EndPoint::New(pos) => PosOrLoc::Pos(pos),
            EndPoint::Old(snap_config) => PosOrLoc::Loc(Loc::new(
                snap_config.pos(),
                snap_config.dir().flip(snap_config.is_reverse()).into(),
            )),
        }
    }
}

/// Describes the action that should be taken by the construct tool.
pub enum CurveAction {
    /// Construct the given curve as part of the world.
    Construct(CompositeCurveSum),
    /// Render the given curve while still constructing it.
    Render(CompositeCurveSum, CurveInfo),
    /// A direction needs to be chosen.
    Direction(Loc, Vec3),
    /// A control point needs to be chosen.
    ControlPoint(Vec3, Vec3),
    /// A small road stub should be rendered to indicate that the user can snap to this node.
    Stub(Loc),
    /// The curve builder has nothing to render.
    Nothing,
}

pub type CurveActionResult = Result<CurveAction, CurveError>;

impl<C: Into<CurveSum>> From<(C, CurveInfo)> for CurveAction {
    fn from((curve, curve_info): (C, CurveInfo)) -> Self {
        CurveAction::Render(CompositeCurveSum::Single(curve.into()), curve_info)
    }
}

pub trait CurveToolSpecInternal {
    /// The tool shall process a left click.
    fn left_click(&mut self, first: PosOrLoc, last: PosOrLoc) -> CurveActionResult;

    /// The tool shall process a right click.
    fn right_click(&mut self, first: PosOrLoc, last: PosOrLoc) -> CurveActionResult;

    /// Called whenever there the ground_pos has been updated due to a change in camera or
    /// cursor position.
    fn compute_curve(&mut self, first: PosOrLoc, last: PosOrLoc) -> CurveActionResult;
}

#[enum_dispatch]
pub trait CurveToolSpec {
    /// Selects the first point if it has not already been selected, otherwise delegates the
    /// call to instance.
    fn left_click(&mut self, ground_pos: Vec3) -> CurveActionResult;

    fn right_click(&mut self, ground_pos: Vec3) -> CurveActionResult;

    /// A node has been snapped to.
    fn update_snap(&mut self, snap_config: SnapConfig) -> CurveActionResult;

    /// A node is no longer snapped.
    fn update_no_snap(&mut self, ground_pos: Vec3) -> CurveActionResult;

    fn reset(&mut self, new_snap: Option<SnapConfig>);

    fn get_selected_node(&self) -> Option<SnapConfig>;

    fn get_snapped_node(&self) -> Option<SnapConfig>;

    fn is_building_reverse(&self, state_reverse: bool) -> bool;
}

#[enum_dispatch(CurveToolSpec)]
pub enum CurveToolSum {
    Straight(CurveTool<StraightTool, Curve<Straight>>),
    Circular(CurveTool<CircularTool, Curve<Circular>>),
}

pub struct CurveTool<CT, C: CurveSpec>
where
    CT: CurveToolSpecInternal,
{
    instance: CT,
    first_point: Option<EndPoint>,
    snapped_node: Option<SnapConfig>,
    _marker: PhantomData<C>,
}

impl<CT: Default, C: CurveSpec> Default for CurveTool<CT, C>
where
    CT: CurveToolSpecInternal,
{
    fn default() -> Self {
        Self {
            instance: CT::default(),
            first_point: None,
            snapped_node: None,
            _marker: PhantomData,
        }
    }
}

impl<CT, C: CurveSpec> CurveToolSpec for CurveTool<CT, C>
where
    CT: CurveToolSpecInternal,
{
    fn left_click(&mut self, ground_pos: Vec3) -> CurveActionResult {
        if let Some(first_point) = &self.first_point {
            let last_point = if let Some(snap_config) = self.snapped_node.clone() {
                EndPoint::Old(snap_config)
            } else {
                EndPoint::New(ground_pos)
            };
            let last_point: PosOrLoc = last_point.into();
            let last_point = last_point.flip(true);
            return self
                .instance
                .left_click(first_point.clone().into(), last_point);
        }

        if let Some(snap_config) = self.snapped_node.clone() {
            self.first_point = Some(EndPoint::Old(snap_config.clone()));
            return Ok(CurveAction::Stub(Loc::new(
                snap_config.pos(),
                snap_config.dir().into(),
            )));
        }

        let first_point = EndPoint::New(ground_pos);
        self.first_point = Some(first_point.clone());
        self.instance
            .compute_curve(first_point.into(), ground_pos.into())
    }

    fn right_click(&mut self, ground_pos: Vec3) -> CurveActionResult {
        let Some(first_point) = &self.first_point else {
            return Ok(CurveAction::Nothing);
        };

        let last_point = if let Some(snap_config) = self.snapped_node.clone() {
            EndPoint::Old(snap_config)
        } else {
            EndPoint::New(ground_pos)
        };

        match self
            .instance
            .right_click(first_point.clone().into(), last_point.into())
        {
            Ok(CurveAction::Nothing) => {
                self.first_point = None;
                return Ok(CurveAction::Nothing);
            }
            curve_result => return curve_result,
        }
    }

    fn update_snap(&mut self, snap_config: SnapConfig) -> CurveActionResult {
        self.snapped_node = Some(snap_config.clone());

        if let Some(first_point) = &self.first_point {
            let last_point: PosOrLoc = EndPoint::Old(snap_config).into();
            let last_point = last_point.flip(true);
            return self
                .instance
                .compute_curve(first_point.clone().into(), last_point);
        }

        Ok(CurveAction::Stub(Loc::new(
            snap_config.pos(),
            snap_config.dir().into(),
        )))
    }

    fn update_no_snap(&mut self, ground_pos: Vec3) -> CurveActionResult {
        self.snapped_node = None;

        if let Some(first_point) = &self.first_point {
            return self
                .instance
                .compute_curve(first_point.clone().into(), ground_pos.into());
        };

        Ok(CurveAction::Nothing)
    }

    fn reset(&mut self, new_snap: Option<SnapConfig>) {
        self.snapped_node = None;
        self.first_point = new_snap.map(|s| EndPoint::Old(s));
    }

    fn get_selected_node(&self) -> Option<SnapConfig> {
        self.first_point.clone().and_then(|x| match x {
            EndPoint::New(_) => None,
            EndPoint::Old(snap_config) => Some(snap_config),
        })
    }

    fn get_snapped_node(&self) -> Option<SnapConfig> {
        self.snapped_node.clone()
    }

    fn is_building_reverse(&self, state_reverse: bool) -> bool {
        match (&self.first_point, &self.snapped_node) {
            (Some(EndPoint::Old(first_snap)), None) => first_snap.is_reverse(),
            (_, Some(snap)) => !snap.is_reverse(),
            (_, _) => state_reverse,
        }
    }
}
