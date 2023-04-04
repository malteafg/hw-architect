use glam::Vec3;
use serde::{Deserialize, Serialize};
use utils::id::{SegmentId, VehicleId};

// STATIC vehicle data:
// max_speed(u8), acceleration(u8), aggressiveness(u8) (this is used by ai)
// model(u128), color(96) (this is used by gfx)
//
// route is computed when spawned, maybe updated if bulldoze or bad traffic needed by ai
//
// DYNAMIC vehicle data:
// pos(96), yrot(96) should be written to in every update

#[derive(Copy, Clone, Debug, Serialize, Deserialize)]
pub struct StaticVehicleData {
    /// Given in KM/H
    /// maybe changed to pref_max_speed?
    max_speed: u8,
    /// todo figure out units
    acceleration: u8,
    deceleration: u8,
    aggressiveness: u8,
}

#[derive(Copy, Clone, Debug, Serialize, Deserialize)]
pub struct VehicleAi {
    /// The id of this vehicle.
    pub id: VehicleId,
    /// Represents how far along in meters the vehicle has travelled along this segment.
    pub dist_travelled: f32,
    /// Current speed of vehicle given in KM/H
    pub speed: u8,
}

#[derive(Copy, Clone, Debug, Serialize, Deserialize)]
pub struct VehicleLoc {
    pub pos: Vec3,
    pub yrot: f32,
    pub curr_segment: SegmentId,
}
