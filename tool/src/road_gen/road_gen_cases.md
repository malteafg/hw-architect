# Straight
## None selected
* SelectPos
  * NoData

* SelectDir
  * Pos

* Build
  * N/A

* SelNode
  * Pos
  * Dir

# Curved
## None selected
* SelectPos
  * NoData
  
* SelectDir
  * Pos

* Build
  * Pos
  * Dir

* SelNode
  * Pos
  * Dir



## Straight Free dir SFD
* Start and end position
* No restrictions
* Project if too short

## Straight Locked dir SLD
* Start and end position
* Start direction
* Project if too short
* Projection from mouse to direction

## Double snap DS
* Start and end position
* Start and end direction
* Can fail for several reasons
  * Should default to the mode that is currently in use

## Circle Curve CC
* Start and end position
* Start direction
* Projects to 270 degrees and smallest curvature

## Circle Curve snap CCS
* Start and end position
* Direction from snap
* Cannot project
  * If failure, default to Circle Curve


Small stub that comes from snapping in select pos mode is not a road gen


reverse in road generator should mean that segments and nodes should be reversed when built

sel_node <-> side_locked
