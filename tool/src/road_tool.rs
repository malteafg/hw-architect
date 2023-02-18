mod generator;

use generator::RoadGeneratorTool;
use gfx_api::GfxRoadData;
use glam::*;
use simulation::road_network::{self as network, RoadGen};
use utils::id::{NodeId, SegmentId};
use utils::input;
use std::cell::RefCell;
use std::collections::HashMap;
use std::rc::Rc;

#[derive(Clone, Copy, PartialEq, Eq)]
enum Mode {
    SelectPos,
    SelectDir,
    Build,
    Bulldoze,
}

pub struct ToolState {
    gfx_handle: Rc<RefCell<dyn GfxRoadData>>,
    road_graph: network::RoadGraph,
    sel_road_type: network::RoadType,
    sel_node: Option<network::SnapConfig>,
    snapped_node: Option<network::SnapConfig>,
    road_generator: generator::RoadGeneratorTool,
    ground_pos: Vec3,
    mode: Mode,
    node_id_count: u32,
    segment_id_count: u32,
}

impl ToolState {
    pub fn new(gfx_handle: Rc<RefCell<dyn GfxRoadData>>) -> ToolState {
        ToolState {
            gfx_handle,
            road_graph: network::RoadGraph::default(),
            sel_road_type: network::RoadType {
                no_lanes: 3,
                curve_type: network::CurveType::Straight,
            },
            sel_node: None,
            snapped_node: None,
            road_generator: RoadGeneratorTool::default(),
            ground_pos: Vec3::new(0.0, 0.0, 0.0),
            mode: Mode::SelectPos,
            node_id_count: 0,
            segment_id_count: 0,
        }
    }

    pub fn process_keyboard(&mut self, key: input::KeyAction) {
        use input::Action::*;
        let (action, pressed) = key;
        if pressed {
            return;
        }
        match self.mode {
            Mode::Bulldoze => {
                if action == ToggleBulldoze {
                    self.reset_snap(Mode::SelectPos);
                }
            }
            _ => match action {
                CycleRoadType => self.switch_curve_type(),
                ToggleBulldoze => {
                    self.mode = Mode::Bulldoze;
                    self.gfx_handle.borrow_mut().set_road_tool_mesh(Some(generator::empty_mesh()));
                }
                OneLane => self.switch_lane_no(1),
                TwoLane => self.switch_lane_no(2),
                ThreeLane => self.switch_lane_no(3),
                FourLane => self.switch_lane_no(4),
                FiveLane => self.switch_lane_no(5),
                SixLane => self.switch_lane_no(6),
                _ => {}
            },
        }
    }

    fn generate_node_id(&mut self) -> NodeId {
        let node_id = self.node_id_count;
        self.node_id_count += 1;
        NodeId(node_id)
    }

    fn generate_segment_id(&mut self) -> SegmentId {
        let segment_id = self.segment_id_count;
        self.segment_id_count += 1;
        SegmentId(segment_id)
    }

    fn switch_lane_no(&mut self, no_lanes: u8) {
        if self.sel_road_type.no_lanes == no_lanes {
            return;
        };
        self.sel_road_type.no_lanes = no_lanes;
        self.road_generator.update_no_lanes(no_lanes);
        if self.sel_node.is_none() {
            self.check_snapping();
        } else {
            self.reset_snap(Mode::SelectPos);
        }
    }

    fn switch_curve_type(&mut self) {
        use network::CurveType::*;
        match self.sel_road_type.curve_type {
            Straight => {
                self.sel_road_type.curve_type = Curved;
                self.road_generator.update_curve_type(Curved);
                if let Mode::Build = self.mode {
                    self.check_snapping();
                }
            }
            Curved => {
                self.sel_road_type.curve_type = Straight;
                self.road_generator.update_curve_type(Straight);
                if let Mode::Build = self.mode {
                    if self.sel_node.is_none() {
                        self.road_generator.unlock_dir();
                        self.mode = Mode::SelectDir;
                    }
                    self.check_snapping();
                }
            }
        }
    }

    pub fn mouse_input(&mut self, event: input::MouseEvent) {
        use input::MouseEvent;

        match event {
            MouseEvent::Click(button) if button == input::Mouse::Left => self.left_click(),
            MouseEvent::Click(button) if button == input::Mouse::Right => {
                self.right_click();
            }
            _ => {}
        }
    }

    fn left_click(&mut self) {
        use network::CurveType;
        match self.mode {
            Mode::SelectPos => match self.snapped_node.clone() {
                Some(snapped_node) => {
                    self.select_node(snapped_node);
                }
                None => {
                    self.road_generator =
                        RoadGeneratorTool::new(self.ground_pos, None, self.sel_road_type, false);
                    self.gfx_handle.borrow_mut().set_road_tool_mesh(self.road_generator.get_mesh());

                    self.mode = Mode::SelectDir;
                }
            },
            Mode::SelectDir => match self.sel_road_type.curve_type {
                CurveType::Straight => self.build_road(),
                CurveType::Curved => {
                    self.road_generator.lock_dir(self.ground_pos);
                    self.road_generator.update_pos(self.ground_pos);
                    self.gfx_handle.borrow_mut().set_road_tool_mesh(self.road_generator.get_mesh());

                    self.mode = Mode::Build;
                }
            },
            Mode::Build => self.build_road(),
            Mode::Bulldoze => {
                let segment_id = self.road_graph.get_segment_inside(self.ground_pos);
                if let Some(id) = segment_id {
                    if self.road_graph.remove_segment(id) {
                        self.gfx_handle.borrow_mut().remove_road_meshes(vec![id])
                    }
                }
            }
        }
    }

    fn right_click(&mut self) {
        use network::CurveType;
        match self.mode {
            Mode::SelectPos | Mode::Bulldoze => {
                #[cfg(debug_assertions)]
                {
                    self.road_graph.debug_segment_from_pos(self.ground_pos);
                    self.road_graph.debug_node_from_pos(self.ground_pos);
                }
            }
            Mode::SelectDir => self.reset_snap(Mode::SelectPos),
            Mode::Build => match (self.sel_road_type.curve_type, self.sel_node.clone()) {
                (CurveType::Curved, None) => {
                    self.road_generator.unlock_dir();
                    self.reset_snap(Mode::SelectDir)
                }
                _ => self.reset_snap(Mode::SelectPos),
            },
        }
    }

    fn select_node(&mut self, snapped_node: network::SnapConfig) {
        let node = self.road_graph.get_node(snapped_node.node_id);

        self.road_generator = generator::RoadGeneratorTool::new(
            snapped_node.pos,
            Some(node.get_dir()),
            self.sel_road_type,
            snapped_node.reverse,
        );
        self.road_generator.update_pos(self.ground_pos);
        self.gfx_handle.borrow_mut().set_road_tool_mesh(self.road_generator.get_mesh());

        self.sel_node = Some(snapped_node);
        self.snapped_node = None;
        self.mode = Mode::Build;
    }

    /// Constructs the road that is being generated.
    fn build_road(&mut self) {
        let road_meshes = self.road_generator.get_road_meshes();
        let road_generator = self.road_generator.extract();

        // Id gen and other stuff is just temporary
        let segments = road_generator.clone().extract().1;
        let num_segment_ids = segments.len();
        let segment_ids: Vec<SegmentId> = (0..num_segment_ids)
            .map(|_| self.generate_segment_id())
            .collect();

        let mut num_node_ids = segments.len() - 1;
        if self.snapped_node.is_none() {
            num_node_ids += 1;
        };
        if self.sel_node.is_none() {
            num_node_ids += 1;
        };
        let node_ids = (0..num_node_ids).map(|_| self.generate_node_id()).collect();

        let new_node = self.road_graph.add_road(
            road_generator,
            self.sel_node.clone(),
            self.snapped_node.clone(),
            node_ids,
            segment_ids.clone(),
        );

        let mut mesh_map = HashMap::new();
        for i in 0..num_segment_ids {
            mesh_map.insert(segment_ids[i], road_meshes[i].clone());
        }
        self.gfx_handle.borrow_mut().add_road_meshes(mesh_map);

        // TODO have add_road return new_node in such a way that is not necessary to check snapped_node
        if self.snapped_node.is_some() {
            self.reset_snap(Mode::SelectPos);
        } else if let Some(new_node) = new_node {
            self.select_node(new_node);
        } else {
            self.reset_snap(Mode::SelectPos);
        }
    }

    fn reset_snap(&mut self, new_mode: Mode) {
        if let Mode::SelectPos = new_mode {
            self.road_generator = RoadGeneratorTool::default();
        };
        self.sel_node = None;
        self.snapped_node = None;
        self.mode = new_mode;
        self.check_snapping();
    }

    fn update_no_snap(&mut self) {
        self.snapped_node = None;
        let empty_mesh = Some(generator::empty_mesh());

        match self.mode {
            Mode::SelectPos | Mode::Bulldoze => self.gfx_handle.borrow_mut().set_road_tool_mesh(empty_mesh),
            Mode::SelectDir => {
                self.road_generator.update_pos(self.ground_pos);
                self.gfx_handle.borrow_mut().set_road_tool_mesh(self.road_generator.get_mesh());
            }
            Mode::Build => {
                self.road_generator.update_pos(self.ground_pos);
                self.gfx_handle.borrow_mut().set_road_tool_mesh(self.road_generator.get_mesh());
            }
        };
    }

    fn update_snap(&mut self, snap_configs: Vec<network::SnapConfig>) {
        match self.mode {
            Mode::SelectPos => {
                let snap_config = &snap_configs[0];
                let road_generator = RoadGeneratorTool::new(
                    snap_config.pos,
                    Some(snap_config.dir),
                    self.sel_road_type,
                    snap_config.reverse,
                );
                self.snapped_node = Some(snap_config.clone());
                self.gfx_handle.borrow_mut().set_road_tool_mesh(road_generator.get_mesh());
            }
            Mode::SelectDir | Mode::Build => {
                for snap_config in snap_configs {
                    if self
                        .road_generator
                        .try_snap(snap_config.clone(), self.sel_node.is_some())
                        .is_some()
                    {
                        self.snapped_node = Some(snap_config);
                        self.gfx_handle.borrow_mut().set_road_tool_mesh(self.road_generator.get_mesh());
                        return;
                    }
                }
                self.update_no_snap();
            }
            Mode::Bulldoze => self.gfx_handle.borrow_mut().set_road_tool_mesh(Some(generator::empty_mesh())),
        };
    }

    fn check_snapping(&mut self) {
        if let Some((_snap_id, snap_configs)) = self
            .road_graph
            .get_node_snap_configs(self.ground_pos, self.sel_road_type.no_lanes)
        {
            if snap_configs.is_empty() {
                self.update_no_snap();
            } else {
                self.update_snap(snap_configs);
            }
        } else {
            self.update_no_snap();
        }
    }

    /// This function should be called whenever there is an update to where the mouse points on the
    /// ground. This includes mouse movement and camera movement.
    pub fn update_ground_pos(&mut self, ground_pos: Vec3) {
        self.ground_pos = ground_pos;
        self.check_snapping();

        self.mark_segment(self.road_graph.get_segment_inside(self.ground_pos));
    }

    fn mark_segment(&mut self, segment_id: Option<SegmentId>) {
        if let Some(id) = segment_id {
            if self.mode == Mode::Bulldoze {
                self.gfx_handle.borrow_mut().mark_road_segments(vec![id]);
                return
            }
        }
        self.gfx_handle.borrow_mut().mark_road_segments(vec![]);
    }
}

