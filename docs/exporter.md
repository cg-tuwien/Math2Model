# Exporter
The exporter allows exporting of parametric models to common 3D file formats.
The export is based on subdividing the plane in coordinate space into subplanes depending on the export configuration.
These subplanes are known as "Patches", and the subdivisions will always divide a patch into 4 equally sized patches in the input coordinate space.

## LOD Configuration
### Preview
The export preview can be toggled, when enabled it will filter for the meshes that were selected for export in the main viewport.
The meshes will be displayed at the level of detail that was set in the exporter window, though unstitched, making it only a rough preview of how the final export will look.

### Target Model
Target model to download and preview.

### Min Size
The minimum size of an exported patch in world space, if a patch is smaller than this, it will not be divided further, even if other criteria would demand it.
This is the easiest way of dynamically limiting the meshes size, and preserves only as much detail as needed, while avoiding creation of extremely small patches.

### Max Curvature
Limit the maximum curvature that a patch would skip. This is based on the cosine similarity of the normals of potential subpatches. If subpatch normals would diverge more than this value, the patch will be subdivided.

### Planarity Criterium
Set the minimal planarity of each patch that is rendered, planarity is calculated via principal component analysis of 32 equally spread sample points of a patch and ranges from 0, no planarity at all required, to 1, where patches must be perfect planes to not be subdivided. (Internally, the planarity criterium ranges from from 0.95 to 1, but a remapping is applied to use more human friendly range of reasonable values.)

### Divsion Steps
How many times the patch should be subdivided into 4, this number is doubled, so the upper limit for patch count is 4 to the power of (div steps * 2), unless the LOD stages criteria above cause an early exit.

## Export Settings
### File Format
Choose between GLTF Binary/GLB and OBJ file format to download

### Merge Model Files
If all models should be merged into the same file, or if they should be downloaded one after the other

### Include UVs
If UVs should be included in model exports, UVs are used for texturing the model

### Normal Direction
Which directions normals should be facing, if your model looks strange, try flipping from default to inverse or the other way around. If your model is not closed and you want it to be visible from both sides, try double sided normals, though this will increase file size by a margin of about 30% 

## Download
Start the download process, this will display a progress bar. If the download freezes for a few seconds to a minute, leave it be, it should continue eventually once GPU readback of data is complete. Otherwise, use the cancel button to end downloading.