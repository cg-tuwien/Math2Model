# Exporter
The exporter allows exporting of parametric models into two common 3D file formats: GLTF Binary/GLB and OBJ.
The export is based on subdividing the 2D input parameters into subregions of parameters depending on the export configuration.
These subregions will always divide 1:4, i.e., one input region into four output regions uniformly.

## LOD Configuration

The following options can be set under the `LOD Configuration` category:

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

## File Configuration

The following options can be set under the `File Configuration` category:

### File Format
Select either GLTF Binary/GLB or OBJ file format for the download

### Merge Model Files
This toggle controls If all models should be merged into the same file, or if they should be downloaded one after the other (This may result in downloading many files)

### Include UVs
If UVs should be included in model exports, UVs are used for texturing the model and may assist in reapplying textures the same way they are applied in Math2Model

### Normal Direction
Which directions normals should be facing, if output models look strange, flipping the normals from default to inverse or the other way around may correct it. If the model is not closed and should be visible from both sides, double sided normals may be the best solution, at the cost of an around 30% increased file size.

## Download
Start the download process, this will display a progress bar. This process may take a while, if it appears to freeze during export of a complex model with many instances it should be left to run and will terminate eventually. If it appears to be stuck, the cancel button should be used to end downloading.