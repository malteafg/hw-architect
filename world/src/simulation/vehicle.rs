use glam::Vec3;
use serde::{Deserialize, Serialize};
use utils::id::{SegmentId, VehicleId};

use std::time::Duration;

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
    /// Represents how far along in meters the vehicle has travelled along this segment in meters.
    pub dist_travelled: f32,
    /// Current speed of vehicle given in m/s
    pub speed: f32,
}

impl VehicleAi {
    pub fn has_reached_end(&self, max_length: f32) -> bool {
        if self.dist_travelled > max_length {
            self.dist_travelled -= max_length;
            return true;
        }
        false
    }

    /// Updates the vehicles dist_travelled. Returns true if the vehicle has surpassed the given
    /// length.
    pub fn travel(&mut self, dt: Duration) {
        let dist_to_travel = dt.as_secs_f32() * self.speed;
        self.dist_travelled += dist_to_travel;
    }
}

#[derive(Copy, Clone, Debug, Serialize, Deserialize)]
pub struct VehicleLoc {
    pub pos: Vec3,
    pub yrot: f32,
    pub curr_segment: SegmentId,
}
