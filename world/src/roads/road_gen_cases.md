# Straight
## SelectPos
* NoData
* Snap: Gen segment

## SelectDir
* Pos
* NoSnap: SFD
* Snap: CCS -> SFD

## Build N/A

## SelNode
* Pos
* Dir
* NoSnap: SLD
* Snap: DS -> SLD

# Curved
## SelectPos
* NoData
* Snap: Gen segment
  
## SelectDir
* Pos
* NoSnap: SFD
* Snap: CCS -> SFD

## Build
* Pos
* Dir
* NoSnap: CC
* Snap: DS (dir is set) -> CC

## SelNode
* Pos
* Dir
* NoSnap: CC
* Snap: DS -> CC

NoSnap: SFD, SLD, CC
Snap: CCS, DS

NoSel: SFD, CCS, CC, DS
Sel: CC, DS, SLD

SFD: NoSnap, NoSel
SLD: NoSnap, Sel
CC: NoSnap, NoSel, Sel
CCS: Snap, NoSel
DS: Snap, NoSel, Sel


# Generation types
## Straight Free dir SFD
* Start and end position
* No restrictions
* Project if too short

## Straight Locked dir SLD
* Start and end position
* Start direction
* Project if too short
* Projection from mouse to direction

## Circle Curve CC
* Start and end position
* Start direction
* Projects to 270 degrees and smallest curvature

## Circle Curve snap CCS
* Start and end position
* Direction from snap
* Cannot project
  * If failure, default to curve type in use

## Double snap DS
* Start and end position
* Start and end direction
* Can fail for several reasons
  * Should default to the mode that is currently in use


Small stub that comes from snapping in select pos mode is not a road gen


reverse in road generator should mean that segments and nodes should be reversed when built

sel_node <-> side_locked
