# Exporter
The exporter allows exporting of parametric models to common 3D file formats.
The export is based on subdividing the plane in coordinate space into subplanes depending on the export configuration.
These subplanes are known as "Patches", and the subdivisions will always divide a patch into 4 equally sized patches in the input coordinate space.

## Preview
The export preview can be toggled, when enabled it will filter for the meshes that were selected for export in the main viewport.
The meshes will be displayed at the level of detail that was set in the exporter window, though unstitched, making it only a rough preview of how the final export will look.

## Min Size
The minimum size of an exported patch in world space, if a patch is smaller than this, it will not be divided further, even if other criteria would demand it.
This is the easiest way of dynamically limiting the meshes size, and preserves only as much detail as needed, while avoiding creation of extremely small patches.

## Max Curvature
Limit the maximum curvature that a patch would skip. This is based on the cosine similarity of the normals of potential subpatches. If subpatch normals would diverge more than this value, the patch will be subdivided.

## Planarity Criterium
Set the minimal planarity of each patch that is rendered, planarity is calculated via principal component analysis of 32 equally spread sample points of a patch and ranges from 0, no planarity at all required, to 1, where patches must be perfect planes to not be subdivided. (Internally, the planarity criterium ranges from from 0.95 to 1, but a remapping is applied to use more human friendly range of reasonable values.)

## 