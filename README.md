# hw-architect

Primary goal:
- An intuitive powerful tool for building roads.

Secondary goals:
- Efficient, parallel ai for vehicles, that act dynamically on a changing road environment.
- Structured using ECS (entity, component, system)
- Decoupled graphics backend, such that it can be replaced without compromising the rest of the code.
- Implementation of marching cubes algorithm for terrain generation.

## building and running
### desktop
Use ```cargo build``` to (re)compile the code. To run hw-architect use ```cargo run```, which will also (re)compile the code. Install hw-architect locally using ```cargo install --path .```.

### web (wasm)
Install [wasm-pack](https://rustwasm.github.io/wasm-pack/installer/). Use ```wasm-pack build app/ --out-dir ../pkg --target web --dev``` to compile the code to wasm. In the root of the repo use ```python3 -m http.server``` and navigate to localhost:8000 in a browser.


## dependencies
[Rust](https://doc.rust-lang.org/book/)

ECS: 
- [specs](https://github.com/amethyst/specs)
- [tutorial](https://specs.amethyst.rs/docs/tutorials/)

Window and input handler:
- [winit](https://github.com/rust-windowing/winit)

Graphics
- [wgpu](https://github.com/gfx-rs/wgpu)
- [tutorial](https://sotrh.github.io/learn-wgpu/#what-is-wgpu)

Graphics (can be changed, open for other options):
- [vulkano](https://github.com/vulkano-rs/vulkano)
- [tutorial](https://vulkano.rs/guide/introduction)

## plan
### phase 0 - setup
- Create window using winit
- Draw triangle using vulkano
- Experiment with setting up specs
- Write a simple render system that acts upon components

### phase 1 - scala highway architect
- Build 1-4 lane highways
  - Curves, straights, with snapping to lanes
- Have cars follow lane paths
- Add lane markings to roads (probably a mesh for each stripe, who cares) 

### phase 2 - smarter cars
- Cars can dynamically act upon the road environment
  - If a vehicle drives slowly then vehicles simply switch lanes and pass it
- Pathfinding algorithm from source to destination
- Elevated roads (bridges)

### future phases
### phase x - road editing
- Tools for manipulating roads ala move it for cities skylines

### phase x - transition segments
- Advanced segments that change from the src node to the dst node
- Intersections
- Traffic lights

### phase x - marching cubes
- Tools for manipulating terrain using marching cubes algorithm
- Tunnels

### phase x - trains
- Train tracks and trains

## Structure
### Data
#### Simulation Data
- Road Graph
  - Road nodes
  - Lane connectors
- Cars Data
  - Position/Orientation
  - Collision data
  - Speed data
  - Location on road graph

#### World data
- Tree positions

#### Graphics data
- Road Mesh
- Terrain Mesh
- Car position, orientation and model
- Tree position, orientation and model
- Sphere position

### Functionality
#### Simulation
- Has flush function which returns a data structure to tool, that contains all
  changes since last flush
  - This includes cars, whose positions have changed, and changes to the road 
    graph.

#### Tool
- Contains the functionality to take data from simulation and world, and then
  translate and pass it to graphics. Maybe tool computes the graphics as well

#### Graphics
- Has functionality to take information from tool, compute the graphics, and
  store the data on the gpu
  - Store it in ram as well (or just the raw data that can generate gpu data),
    such that we can partition data in to chunks and not make large writes to
    gpu all at once
