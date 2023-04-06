use super::vehicle::{StaticVehicleData, VehicleAi, VehicleLoc};
use crate::roads::RoadGraph;

use curves::SpinePoints;
use serde::{Deserialize, Serialize};
use utils::id::{IdMap, IdSet, SegmentId, VehicleId, MAX_NUM_ID};

use glam::Vec3;

use std::{collections::VecDeque, time::Duration};

const DEFAULT_VEHICLE_CAP: usize = 8;

#[derive(Debug, Clone, Serialize, Deserialize)]
struct LaneState {
    /// The VecDeque is sorted by the dist_travelled field inside VehicleAi in order from largest
    /// to smallest, ie. the vehicle that has reached the furtest (and thus should be simulated
    /// first) is located at the end.
    state: VecDeque<VehicleAi>,
    path: SpinePoints,
    /// Length of this lane given in meters.
    length: f32,

    /// Used to mark vehicles that need to move to new segments. Allocated here such that we do not
    /// do unnecessary memory allocations in each update.
    marker: Vec<u8>,
}

impl LaneState {
    fn new(path: SpinePoints) -> Self {
        let state = VecDeque::with_capacity(DEFAULT_VEHICLE_CAP);
        let length = path.compute_length();
        let marker = Vec::with_capacity(1);

        Self {
            state,
            path,
            length,
            marker,
        }
    }

    /// Maybe iterate over all vehicles first, and then cars will make decision, and then simulate
    /// can be called?
    fn iter_vehicles(&self) -> impl Iterator<Item = &VehicleAi> + '_ {
        self.state.iter()
    }

    /// Instead of returning a Vec maybe have local memory allocated and let the caller get a
    /// reference to that memory. This would avoid unnecessary memory allocations.
    /// # Panics
    ///
    /// Will crash if more than 256 vehicles exit this lane in this frame, but this is deemed
    /// unlikely.
    fn simulate(&mut self, dt: Duration) -> Vec<VehicleAi> {
        self.marker.clear();

        // simulate and return the vehicles that have exited this lane
        for (i, vehicle) in self.state.iter_mut().enumerate() {
            if vehicle.travel(dt, self.length) {
                self.marker.push(i as u8);
            }
        }

        vec![]
    }

    /// Must maintain the invariant that self.state is sorted.
    /// Returns the position and y_rot of the car inserted.
    fn insert(&mut self, id: VehicleId, vehicle_ai: VehicleAi) -> (Vec3, f32) {
        let num_vehicles = self.state.len();
        let lowest_f32 = self
            .state
            .get(num_vehicles - 1)
            .map(|v| v.dist_travelled)
            .unwrap_or(0.);
        if vehicle_ai.dist_travelled < lowest_f32 {
            self.state.push_back(vehicle_ai);
        }
        (Vec3::new(0., 0., 0.), 0.)
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

    /// Returns vehicles that need to be removed, because they have reached their destination.
    /// Maybe reconsider return type to avoid unnecessary allocations.
    fn simulate(&mut self, dt: Duration) -> Vec<VehicleId> {
        // iter over vehicles and make decisions for each vehicle
        // loop over segment_state and simulate all lane states
        // move done vehicles into their next segment, or remove if they are done
        // iter over vehicles and write their new positions to vehicles_loc
        vec![]
    }
}

// Maybe do the ai in such a way that the road graph is explored with backwards_refs. Then we do
// not need the vehicle_tracker_swap. This is complicated with intersections maybe, but then
// intersections should simply be simulated first?
//
// in the backwards pass, when a segment is processed, it should be sufficient to report data about
// the vehicles that have reached the smallest distance for each lane, and probably those vehicles'
// speed.
/// Maybe add random shrink_to_fit, such that the memory will not become too large.
///
/// Have two maps for each SegmentState, containing the front vehicles and back vehicles.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SimHandler {
    /// Sim needs to read from this, but StaticVehicleData can be read only.
    vehicles_data: IdMap<VehicleId, StaticVehicleData>,

    /// Sim needs to write to this, so VehicleLoc is mut.
    vehicles_loc: IdMap<VehicleId, VehicleLoc>,

    /// Maybe wrap the SegmentState in an Arc<RwLock<>>, when doing parallelism.
    vehicle_tracker: IdMap<SegmentId, SegmentState>,

    /// Represents all the segments currently in game. Must only be modified when a segment is
    /// added or removed.
    /// TODO make FixedBitSet type as IdSet in id.rs
    segments_to_dispatch: IdSet<SegmentId>,

    /// Memory allocated for highlighting when a segment has been processed in each update
    /// iteration.
    processed_segments: IdSet<SegmentId>,

    /// Memory allocated for indicating which vehicles need to be removed as a result of having
    /// reached their destination in this update iteration.
    vehicles_to_remove: Vec<VehicleId>,
}

impl Default for SimHandler {
    fn default() -> Self {
        let vehicles_data = IdMap::new();
        let vehicles_loc = IdMap::new();
        let vehicle_tracker = IdMap::new();
        let segments_to_dispatch = IdSet::new();
        let processed_segments = IdSet::new();
        let vehicles_to_remove = Vec::new();

        Self {
            vehicles_data,
            vehicles_loc,
            vehicle_tracker,
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
///
/// TODO add some phantom segmentstate in front such that the segment_state can be updated based on
/// what is in front of it.
///
/// Maybe do something about return type such that we do not have to allocate in each update.
fn process(dt: Duration, segment_state: &mut SegmentState) -> Vec<VehicleId> {
    segment_state.simulate(dt);
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
        // TODO remove this clone by having a swap buffer
        let segments_left = self.segments_to_dispatch.clone();
        // (3) create a list of segment ids from from backwards_refs of road_graph.ending_nodes
        let mut ready_to_dispatch = road_graph.get_ending_segments();

        // start looping until (2) is empty
        while !segments_left.is_empty() {
            // dispatch segments from (3) and remove them from that list. remove from (2) as well.
            let next_segment = ready_to_dispatch.pop();
            let Some((node_id, segment_id)) = next_segment else {
                // select some random segments and then process from there
        // if at some point (3) is empty but (2) is not, just dispatch randomly from (2) using a
        // fake sim of the forward segments from the segment we are processing
                continue;
            };

            self.segments_to_dispatch.remove(&segment_id);
            // let dst = self
            //     .vehicle_tracker_swap
            //     .get_mut(&segment_id)
            //     .expect("Segment state did not exist in vehicle tracker swap map");
            let segment_state = self.vehicle_tracker.get_mut(&segment_id);

            let mut result = process(dt, segment_state);
            self.vehicles_to_remove.append(&mut result);
            // when a dispatch returns add the backwards segments of the processed segments to (3) if
            // they still exist in (2)
            self.processed_segments.insert(&segment_id);

            let mut ready = true;
            for (_, required_segment) in road_graph.get_forwards_ref(&node_id) {
                if !self.processed_segments.contains(required_segment) {
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

        // remove vehicles that are done
        // TODO find a way to remove this unnecessary clone
        for id in self.vehicles_to_remove.clone().iter() {
            self.remove_vehicle(*id);
        }
    }

    pub fn add_segment(&mut self, segment: SegmentId, lane_paths: Vec<SpinePoints>) {
        self.vehicle_tracker
            .insert(&segment, SegmentState::new(lane_paths));
        self.segments_to_dispatch.insert(&segment);
    }

    pub fn remove_segment(&mut self, segment: SegmentId) {
        // TODO what about the vehicles in the segment?
        self.vehicle_tracker.remove(&segment);
        self.segments_to_dispatch.remove(&segment);
    }
}
