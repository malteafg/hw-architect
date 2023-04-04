use super::vehicle::{Vehicle, VehicleLoc};
use crate::roads::{LSegment, RoadGraph};

use curves::SpinePoints;
use serde::{Deserialize, Serialize};
use utils::id::{SegmentId, VehicleId, MAX_NUM_ID};

use fixedbitset::FixedBitSet;

use std::{
    collections::{BTreeMap, HashMap, VecDeque},
    time::Duration,
};

const DEFAULT_VEHICLE_CAP: usize = 8;

#[derive(Debug, Clone, Serialize, Deserialize)]
struct LaneState {
    /// The f32 represents how far along in meters the vehicle has travelled along this segment.
    /// The VecDeque should probably always be sorted.
    state: VecDeque<(VehicleId, f32)>,
    path: SpinePoints,
}

impl LaneState {
    fn new(path: SpinePoints) -> Self {
        let state = VecDeque::with_capacity(DEFAULT_VEHICLE_CAP);
        Self { state, path }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct SegmentState {
    lane_map: Vec<LaneState>,
}

impl SegmentState {
    fn new(lane_paths: Vec<SpinePoints>) -> Self {
        let mut lane_map = Vec::with_capacity(lane_paths.len());
        for lane_path in lane_paths.into_iter() {
            lane_map.push(LaneState::new(lane_path));
        }
        Self { lane_map }
    }
}

// Maybe do the ai in such a way that the road graph is explored with backwards_refs. Then we do
// not need the vehicle_tracker_swap. This is complicated with intersections maybe, but then
// intersections should simply we simulated first?
//
// in the backwards pass, when a segment is processed, it should be sufficient to report data about
// the vehicles that have reached the smallest distance for each lane, and probably those vehicles'
// speed.
/// Maybe add random shrink_to_fit, such that the memory will not become too large.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SimHandler {
    vehicles: HashMap<VehicleId, Vehicle>,
    /// Maybe wrap the SegmentState in an Arc<RefCell<>>, when doing parallelism.
    vehicle_tracker: HashMap<SegmentId, SegmentState>,
    /// Always assert that the point is not a duplicate when inserting. No duplicate keys!
    vehicle_locs: BTreeMap<VehicleId, VehicleLoc>,
    // vehicle_locs: BTreeMap<f32, (VehicleId, VehicleLoc)>,
    /// Represents all the segments currently in game. Must only be modified when a segment is
    /// added or removed.
    segments_to_dispatch: FixedBitSet,

    /// Memory allocated for highlighting when a segment has been processed in each update
    /// iteration.
    processed_segments: FixedBitSet,
    vehicles_to_remove: Vec<VehicleId>,
}

impl Default for SimHandler {
    fn default() -> Self {
        let vehicles = HashMap::new();
        let vehicle_tracker = HashMap::new();
        let vehicle_locs = BTreeMap::new();
        let segments_to_dispatch = FixedBitSet::with_capacity(MAX_NUM_ID);
        let processed_segments = FixedBitSet::with_capacity(MAX_NUM_ID);
        let vehicles_to_remove = Vec::new();

        Self {
            vehicles,
            vehicle_tracker,
            vehicle_locs,
            segments_to_dispatch,
            processed_segments,
            vehicles_to_remove,
        }
    }
}

/// Reference to LSegment can be replaced with std::sync::Arc. Maybe be more specific and only
/// reference lane paths.
/// Figure out exactly how the segment states should be passed when doing parallelism. It might be
/// fine to just give vehicle_tracker_swap wrapped in Arc<RefCell> to all threads.
/// When parallel the thread should send a request for more stuff to process.
fn process(
    dt: Duration,
    segment_state: &mut SegmentState,
    // dst: &mut SegmentState,
    // src: &SegmentState,
    segment: &LSegment,
) -> Vec<VehicleId> {
    vec![]
}

impl SimHandler {
    pub fn spawn_vehicle(&mut self, segment: SegmentId, lane: u8) {}
    fn remove_vehicle(&mut self, vehicle: VehicleId) {}

    pub fn update(&mut self, dt: Duration, road_graph: &RoadGraph) {
        // dispatching a segment always implies that the segment will be processed
        // clear the memory that we need for this iteration.
        self.vehicles_to_remove.clear();
        self.processed_segments.clear();

        // (2) clone segments_to_dispatch. This contains all segments that need dispatching
        let segments_left = self.segments_to_dispatch.clone();
        // (3) create a list of segment ids from from backwards_refs of road_graph.ending_nodes
        let mut ready_to_dispatch = road_graph.get_ending_segments();

        // start looping until (2) is empty
        while !segments_left.is_clear() {
            // dispatch segments from (3) and remove them from that list. remove from (2) as well.
            let next_segment = ready_to_dispatch.pop();
            let Some((node_id, segment_id)) = next_segment else {
                // select some random segments and then process from there
        // if at some point (3) is empty but (2) is not, just dispatch randomly from (2) using a
        // fake sim of the forward segments from the segment we are processing
                continue;
            };

            self.segments_to_dispatch.set(segment_id.usize(), false);
            let segment_ref = road_graph.get_segment(&segment_id);
            // let dst = self
            //     .vehicle_tracker_swap
            //     .get_mut(&segment_id)
            //     .expect("Segment state did not exist in vehicle tracker swap map");
            let segment_state = self
                .vehicle_tracker
                .get_mut(&segment_id)
                .expect("Segment state did not exist in vehicle tracker map");

            let mut result = process(dt, segment_state, segment_ref);
            self.vehicles_to_remove.append(&mut result);
            // when a dispatch returns add the backwards segments of the processed segments to (3) if
            // they still exist in (2)
            self.processed_segments.put(segment_id.usize());

            let mut ready = true;
            for (_, required_segment) in road_graph.get_forwards_ref(&node_id) {
                if !self.processed_segments.contains(required_segment.usize()) {
                    ready = false;
                    break;
                }
            }

            if ready {
                // todo add the forward data from the segments just processed
                road_graph
                    .get_backwards_ref(&node_id)
                    .iter()
                    .for_each(|p| ready_to_dispatch.push(*p));
            }
        }

        // std::mem::swap(&mut self.vehicle_tracker, &mut self.vehicle_tracker_swap);

        // remove vehicles that are done
        // TODO find a way to remove this unnecessary clone
        for id in self.vehicles_to_remove.clone().iter() {
            self.remove_vehicle(*id);
        }
    }

    pub fn add_segment(&mut self, segment: SegmentId, lane_paths: Vec<SpinePoints>) {
        self.vehicle_tracker
            .insert(segment, SegmentState::new(lane_paths));
        self.segments_to_dispatch.put(segment.usize());
    }

    pub fn remove_segment(&mut self, segment: SegmentId) {
        // TODO what about the vehicles in the segment?
        self.vehicle_tracker.remove(&segment);
        self.segments_to_dispatch.set(segment.usize(), false);
    }
}
