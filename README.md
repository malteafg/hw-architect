# hw-architect
An intuitive powerful tool for building roads.

## building and running
Use ```make build``` to (re)compile the code. To run hw-architect use ```make run```, which will
also (re)compile the code. To run hw-architect with backtrace in case of crashes use ```make
debug```. Install hw-architect locally using ```cargo install --path .```.

## next minor things to implement
### better collision detection for segments
Have Simon do this.

### position road arrows at exactly the middle of segments (not t=0.5)
- Have Simon do this.
- When done add markers for each lane when snapping to a node.

### figure out how to compute the spine points of parallel segments
- Maybe only compute the spine points of one of the segments and then create the other segments'
  spine points by adding the width to the first spine points, and computing the uniform spine
  points.
- It is very important parallel segments always have the exact same distance to each other.

### group material textures in a directory and then load them before loading models

### id manager reuses ids
- When an id is deleted from an id manager it should cache that id and use it when a new id is
  requested

## next major things to implement
### super nodes
- A super node should simply be an addition to the road_graph that points to the nodes that it
  contains. The underlying road graph should not be aware or interact with super nodes. Their
  purpose is to make it easier to construct advanced and bidirectional roads.
- First make it such that the construct tool can be used to make bidirectional roads (maybe with a
  median).
- Add snapping for when the select super node type matches exactly the super node type hovered over.
- Add snapping where the user can select an exact range of the lanes in any node and then that road
  type will be created and the user can build that road.
- Figure out how to extend or recreate a super node, by snapping segments to the side of any node.

### transition segments
Figure out which of the following three versions of transition segments should be used and in which
order to implement them. Probably just implement them in the order written.
1. Transition segments between nodes, where the node_type is exactly the same (except for no_lanes).
2. Transition segments between nodes, where the node_type differs, but no_lanes is exactly the same.
3. Transition segments between nodes, that satisfy both of the above criteria.

### euler spirals
Have Simon do this.

### cars following lane path
Cars should be spawnable at beginnings of nodes or by clicking on segments, and then they will
traverse the road until the end of a node.

### user interface
- Use wgpu-glyph for text rendering.
- When an input event is sent to the ui, it returns the input actions that the tool is already set
  up to handle.

## future things to implement
### parallel resource loading
Load shaders, models, textures, terrain and save game (when it becomes a thing) in parallel.
- Not as necessary anymore after using dev.opt-level=1

### multiplayer
This should be doable by simply providing a server and a client implementation of world-api. It
might be necessary to distinguish in world-api, functions that are called occasionally and functions
that are called often (on mouse movement). Functions that are called often might need to be set up
smarter such that the time to communicate with the server is not noticeable for the player.

### efficient data structure for querying trees
Knowledge learnt by doing this can then be applied to writing efficient data structures for other
stuff such as cars, nodes, segments and so on.

### some tool for upgrading/changing segments

### save games
- This should be doable by simply writing the World struct to a file, as this struct is already
  serializable.
- Add functionality then to compute the graphics from a World struct.
- Maybe also add functionality to save the graphics state. This will require some work as each
  renderer has data that needs to be saved, and this data is currently not close together in memory.

### collision detection between road segments
Have Simon do this.

### heights for roads
Road heights using parabolas.

### parallel pathfinding for cars

## far future things to implement
### marching cubes algorithm for terrain generation
### road intersections
### dynamically acting cars
Cars should be able to freely change lanes based on the traffic they see in front of them.

### something similar to the move it mod for cs
This maybe a part of doings the road heights

### trains, trains and lots of trains

<!-- Goals: -->
<!-- - Efficient, parallel ai for vehicles, that act dynamically on a changing road environment. -->
<!-- - Decoupled graphics backend, such that it can be replaced without compromising the rest of the -->
<!-- - Implementation of marching cubes algorithm for terrain generation. -->

<!-- ## dependencies -->
<!-- [Rust](https://doc.rust-lang.org/book/) -->

<!-- Window and input handler: -->
<!-- - [winit](https://github.com/rust-windowing/winit) -->

<!-- Graphics -->
<!-- - [wgpu](https://github.com/gfx-rs/wgpu) -->
<!-- - [tutorial](https://sotrh.github.io/learn-wgpu/#what-is-wgpu) -->

<!-- Graphics (can be changed, open for other options): -->
<!-- - [vulkano](https://github.com/vulkano-rs/vulkano) -->
<!-- - [tutorial](https://vulkano.rs/guide/introduction) -->

<!-- ## plan -->
<!-- ### phase 0 - setup -->
<!-- - Create window using winit -->
<!-- - Draw triangle using vulkano -->
<!-- - Experiment with setting up specs -->
<!-- - Write a simple render system that acts upon components -->

<!-- ### phase 1 - scala highway architect -->
<!-- - Build 1-4 lane highways -->
<!--   - Curves, straights, with snapping to lanes -->
<!-- - Have cars follow lane paths -->
<!-- - Add lane markings to roads (probably a mesh for each stripe, who cares)  -->

<!-- ### phase 2 - smarter cars -->
<!-- - Cars can dynamically act upon the road environment -->
<!--   - If a vehicle drives slowly then vehicles simply switch lanes and pass it -->
<!-- - Pathfinding algorithm from source to destination -->
<!-- - Elevated roads (bridges) -->

<!-- ### future phases -->
<!-- ### phase x - road editing -->
<!-- - Tools for manipulating roads ala move it for cities skylines -->

<!-- ### phase x - transition segments -->
<!-- - Advanced segments that change from the src node to the dst node -->
<!-- - Intersections -->
<!-- - Traffic lights -->

<!-- ### phase x - marching cubes -->
<!-- - Tools for manipulating terrain using marching cubes algorithm -->
<!-- - Tunnels -->

<!-- ### phase x - trains -->
<!-- - Train tracks and trains -->
