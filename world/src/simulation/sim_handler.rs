use super::vehicle::{StaticVehicleData, VehicleAi, VehicleLoc};
use crate::roads::RoadGraph;

use curves::SpinePoints;
use serde::{Deserialize, Serialize};
use utils::id::{IdMap, IdSet, NodeId, SegmentId, UnsafeMap, VehicleId};

use glam::Vec3;

use std::{collections::VecDeque, time::Duration};

const DEFAULT_VEHICLE_CAP: usize = 8;

#[derive(Debug, Clone, Serialize, Deserialize)]
struct LaneState {
    /// The VecDeque is sorted by the dist_travelled field inside VehicleAi in order from largest
    /// to smallest, ie. the vehicle that has reached the furtest (and thus should be simulated
    /// first) is located at the beginning.
    state: VecDeque<VehicleAi>,
    path: SpinePoints,
    /// Length of this lane given in meters.
    length: f32,
}

impl LaneState {
    fn new(path: SpinePoints) -> Self {
        let state = VecDeque::with_capacity(DEFAULT_VEHICLE_CAP);
        let length = path.compute_length();

        Self {
            state,
            path,
            length,
        }
    }

    /// Maybe iterate over all vehicles first, and then cars will make decision, and then simulate
    /// can be called?
    fn iter_vehicles(&self) -> impl Iterator<Item = &VehicleAi> + '_ {
        self.state.iter()
    }

    /// Updates all the vehicles and returns the vehicle that has reached a new segment if such
    /// vehicle exists. If dt is sufficiently low there would never be several vehicles crossing at
    /// once, and if so they will just be transferred one frame later, which should only make their
    /// render location slightly wrong for a few frames, but not affect the logic otherwise.
    fn simulate(&mut self, dt: Duration, shortest_on_next: Option<f32>) -> Option<VehicleAi> {
        for vehicle in self.state.iter_mut() {
            vehicle.travel(dt)
        }
        if self.state[0].has_reached_end(self.length) {
            return Some(self.state.pop_front().unwrap());
        }
        None
    }

    /// Requires that the distance the vehicle has travelled is smaller than all other distances
    /// travelled in this segment.
    /// Returns the position and y_rot of the car inserted.
    fn insert(&mut self, id: VehicleId, vehicle_ai: VehicleAi) -> (Vec3, f32) {
        #[cfg(debug_assertions)]
        if let Some(v) = self.state.back() {
            assert!(vehicle_ai.dist_travelled < v.dist_travelled);
        }

        self.state.push_back(vehicle_ai);
        // TODO
        (Vec3::new(0., 0., 0.), 0.)

        // let num_vehicles = self.state.len();
        // let lowest_f32 = self
        //     .state
        //     .get(num_vehicles - 1)
        //     .map(|v| v.dist_travelled)
        //     .unwrap_or(0.);
        // if vehicle_ai.dist_travelled < lowest_f32 {}
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct SegmentState {
    lane_map: Vec<LaneState>,
    /// Represents the vehicle that needs to be transferred to their next segment.
    overflow_map: Vec<Option<VehicleAi>>,
}

impl SegmentState {
    fn new(lane_paths: Vec<SpinePoints>) -> Self {
        let mut lane_map = Vec::with_capacity(lane_paths.len());
        for lane_path in lane_paths.into_iter() {
            lane_map.push(LaneState::new(lane_path));
        }

        let overflow_map = Vec::new();
        Self {
            lane_map,
            overflow_map,
        }
    }

    /// Simulates each lane in this segment. Writes to overflow_map the vehicles that have reached
    /// the end of this segment for each lane.
    fn simulate(&mut self, dt: Duration, shortest_on_next: Vec<Option<f32>>) {
        #[cfg(debug_assertions)]
        {
            assert_eq!(self.lane_map.len(), self.overflow_map.len());
            assert_eq!(self.lane_map.len(), shortest_on_next.len());
        }
        // iter over vehicles and make decisions for each vehicle
        // loop over segment_state and simulate all lane states
        // move done vehicles into their next segment, or remove if they are done
        // iter over vehicles and write their new positions to vehicles_loc

        for i in 0..self.lane_map.len() {
            let shortest = shortest_on_next[i];
            let overflow = self.lane_map[i].simulate(dt, shortest);
            self.overflow_map[i] = overflow;
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct FrontConfig {
    /// The Vec of option represents the distance travelled of the vehicles in each lane that has
    /// travelled the smallest distance on the next segments.
    lanes: Vec<Option<f32>>,
    /// Something that dicates how segments are connected.
    node_config: u8,
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
    vehicles_data: IdMap<VehicleId, StaticVehicleData, UnsafeMap>,

    /// Sim needs to write to this, so VehicleLoc is mut.
    vehicles_loc: IdMap<VehicleId, VehicleLoc, UnsafeMap>,

    /// Maybe wrap the SegmentState in an Arc<RwLock<>>, when doing parallelism.
    vehicle_tracker: IdMap<SegmentId, (SegmentState, FrontConfig), UnsafeMap>,

    /// Represents all the segments currently in game. Must only be modified when a segment is
    /// added or removed.
    segments_to_dispatch: IdSet<SegmentId>,
    segments_to_dispatch_buffer: IdSet<SegmentId>,

    ending_segments: Vec<(NodeId, SegmentId)>,

    /// Memory allocated for the segments that are ready to be dispatched.
    ready_to_dispatch: Vec<(NodeId, SegmentId)>,

    /// Memory allocated for highlighting when a segment has been processed in each update
    /// iteration.
    processed_segments: IdSet<SegmentId>,

    /// Memory allocated for indicating which vehicles need to be removed as a result of having
    /// reached their destination in this update iteration.
    /// An IdSet could be used instead but it is probably not worth it as there will only be a few
    /// cars each iteration.
    vehicles_to_remove: Vec<VehicleId>,
}

impl Default for SimHandler {
    fn default() -> Self {
        let vehicles_data = IdMap::new();
        let vehicles_loc = IdMap::new();
        let vehicle_tracker = IdMap::new();

        let segments_to_dispatch = IdSet::new();
        let ending_segments = Vec::new();

        let segments_to_dispatch_buffer = IdSet::new();
        let ready_to_dispatch = Vec::new();

        let processed_segments = IdSet::new();
        let vehicles_to_remove = Vec::new();

        Self {
            vehicles_data,
            vehicles_loc,
            vehicle_tracker,

            segments_to_dispatch,
            ending_segments,

            segments_to_dispatch_buffer,
            ready_to_dispatch,

            processed_segments,
            vehicles_to_remove,
        }
    }
}

fn process(dt: Duration, segment_state: &mut SegmentState, shortest_on_next: Vec<Option<f32>>) {
    segment_state.simulate(dt, shortest_on_next);
    // send signal back and request more to process
}

impl SimHandler {
    pub fn spawn_vehicle(&mut self, segment: SegmentId, lane: u8) {}
    fn remove_vehicle(&mut self, vehicle: VehicleId) {}

    /// Dispatching a segment always implies that the segment will be processed
    pub fn update(&mut self, dt: Duration, road_graph: &RoadGraph) {
        // Prepare the memory that we need for this iteration.
        self.vehicles_to_remove.clear();
        self.processed_segments.clear();
        self.ready_to_dispatch.clear();
        self.segments_to_dispatch
            .write_into(&mut self.segments_to_dispatch_buffer);

        // List of ending segments that can always be processed first. In the future intersections
        // should also be some of the first things to get simulated.
        for ns in self.ending_segments.iter() {
            self.ready_to_dispatch.push(*ns);
        }

        // start looping until (2) is empty
        while !self.segments_to_dispatch_buffer.is_empty() {
            // dispatch segments from (3) and remove them from that list. remove from (2) as well.
            let next_segment = self.ready_to_dispatch.pop();
            let Some((node_id, segment_id)) = next_segment else {
                // TODO
                // select some random segments and then process from there
                // if at some point (3) is empty but (2) is not, just dispatch randomly from (2) using a
                // fake sim of the forward segments from the segment we are processing
                continue;
            };
            self.segments_to_dispatch.remove(segment_id);

            let (segment_state, front_config) = self.vehicle_tracker.get_mut(segment_id);
            process(dt, segment_state);
            // update the backwards segments from this segments with the vehicles new positions
            // update with the contents of segment_state.overflow_map
            // move vehicles to the next segments
            self.processed_segments.insert(segment_id);

            // check if the node that the processed segment backwards pointed to is ready
            let mut ready = true;
            for (_, required_segment) in road_graph.get_forwards_ref(node_id) {
                if !self.processed_segments.contains(*required_segment) {
                    ready = false;
                    break;
                }
            }
            if ready {
                // todo add the forward data from the segments just processed
                for ns in road_graph.get_backwards_ref(node_id).iter() {
                    self.ready_to_dispatch.push(*ns);
                }
            }
        }

        // remove vehicles that are done
        // TODO find a way to remove this unnecessary clone
        for id in self.vehicles_to_remove.clone().iter() {
            self.remove_vehicle(*id);
        }
    }

    pub fn add_segment(&mut self, segment: SegmentId, lane_paths: Vec<SpinePoints>) {
        // TODO update the front map
        self.vehicle_tracker
            .insert(segment, (SegmentState::new(lane_paths), vec![]));
        self.segments_to_dispatch.insert(segment);
    }

    pub fn remove_segment(&mut self, segment: SegmentId) {
        // TODO what about the vehicles in the segment?
        self.vehicle_tracker.remove(segment);
        self.segments_to_dispatch.remove(segment);
    }
}
