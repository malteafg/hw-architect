use super::vehicle::{Vehicle, VehicleLoc};
use crate::roads::RoadGraph;

use utils::id::{SegmentId, VehicleId};

use fixedbitset::FixedBitSet;

use std::{
    collections::{BTreeMap, HashMap, VecDeque},
    time::Duration,
};

/// The f32 represents how far along in meters the vehicle has travelled along this segment.
/// Maybe wrap the value in an Arc, when doing parallelism.
type VehicleLocMap = HashMap<SegmentId, Vec<VecDeque<(VehicleId, f32)>>>;

// Maybe do the ai in such a way that the road graph is explored with backwards_refs. Then we do
// not need the vehicle_tracker_swap. This is complicated with intersections maybe, but then
// intersections should simply we simulated first?
pub struct SimHandler {
    vehicles: HashMap<VehicleId, Vehicle>,
    /// Maybe add random shrink_to_fit, such that the memory will not become too large.
    vehicle_tracker: VehicleLocMap,
    vehicle_tracker_swap: VehicleLocMap,
    /// Always assert that the point is not a duplicate when inserting. No duplicate keys!
    vehicle_locs: BTreeMap<f32, (VehicleId, VehicleLoc)>,
    segments_to_dispatch: FixedBitSet,
}
// in the backwards pass, when a segment is processed, it should be sufficient to report data about
// the vehicles that have reached the smallest distance for each lane, and probably those vehicles'
// speed.

impl SimHandler {
    pub fn spawn_vehicle(&mut self, segment: SegmentId, lane: u8) {}
    pub fn remove_vehicle(&mut self, vehicle: VehicleId) {}
    pub fn update(&mut self, dt: Duration, road_graph: &RoadGraph) {
        // (2) clone segments_to_dispatch. This contains all segments that need dispatching
        // (3) create a list of segment ids from from backwards_refs of road_graph.ending_nodes
        //
        // start looping until (2) is empty
        // dispatch segments from (3) and remove them from that list. remove from (2) as well.
        // dispatching a segment always implies that the segment will be processed
        // when a dispatch returns add the backwards segments of the processed segments to (3) if
        // they still exist in (2)
        // if at some point (3) is empty but (2) is not, just dispatch randomly from (2) using a
        // fake sim of the forward segments from the segment we are processing
        //
    }

    pub fn add_segment(&mut self, segment: SegmentId) {}

    pub fn remove_segment(&mut self, segment: SegmentId) {
        self.vehicle_tracker.remove(&segment);
        self.vehicle_tracker_swap.remove(&segment);
        // self.segments_to_dispatch.set(segment, false);
    }
}
