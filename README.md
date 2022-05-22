# hw-architecht

Primary goal:
- An intuitive powerful tool for building roads.

Secondary goals:
- Efficient, parallel ai for vehicles, that act dynamically on a changing road environment.
- Structured using ECS (entity, component, system)
- Decoupled graphics backend, such that it can be replaced without compromising the rest of the code.
- Implementation of marching cubes algorithm for terrain generation.

## dependencies

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
