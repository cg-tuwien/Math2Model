//// START sampleObject
fn sampleObject(input: vec2f) -> vec3f {
  let a = time;
  let b = screen;
  let c = mouse;
  let d = extra;
  return vec3(input, 0.0); 
}
//// END sampleObject
//// START getColor
fn getColor(input: vec2f) -> vec3f {
  if material.has_texture != 0u {
    return textureSample(t_diffuse, linear_sampler, input * material.texture_scale).rgb;
  } else {
    return material.color_roughness.rgb;
  }
}
//// END getColor

// following https://www.martinpalko.com/triplanar-mapping/
fn calculateTriplanarColor(input: vec3f, normal: vec3f) -> vec3f {
  if material.has_texture != 0u {
    let yuv = input.xz;
    let zuv = input.xy;
    let xuv = input.zy;

    let yDiff = textureSample (t_diffuse, linear_sampler, yuv).rgb;
    let xDiff = textureSample (t_diffuse, linear_sampler, xuv).rgb;
    let zDiff = textureSample (t_diffuse, linear_sampler, zuv).rgb;
    let blendWeights = normalize(abs(normal));

  	return xDiff * blendWeights.x + yDiff * blendWeights.y + zDiff * blendWeights.z;
  } else {
    return material.color_roughness.rgb;
  }
}

var<private> instance_id: u32;

////#include "./Common.wgsl"
//// AUTOGEN 6de14edf9918265eb2e1232f93b94c84430d0069898214379635e51c3d4c9550
struct EncodedPatch {
  u: u32,
  v: u32,
  instance: u32
};
struct Patch {
  min: vec2<f32>,
  max: vec2<f32>,
  instance: u32
};
struct Patches {
  patches_length: atomic<u32>,
  patches_capacity: u32,
  patches : array<EncodedPatch>,
};
struct PatchesRead { // Is currently needed, see https://github.com/gpuweb/gpuweb/discussions/4438
  patches_length: u32, // Same size and alignment as atomic<u32>. Should be legal, right?
  patches_capacity: u32,
  patches : array<EncodedPatch>,
};
struct RenderBuffer {
  patches_length: atomic<u32>,
  patches_capacity: u32,
  patches: array<EncodedPatch>,
};
struct RenderBufferRead {
  patches_length: u32,
  patches_capacity: u32,
  patches: array<EncodedPatch>,
};
struct DispatchIndirectArgs { // From https://docs.rs/wgpu/latest/wgpu/util/struct.DispatchIndirectArgs.html
  x: atomic<u32>,
  y: u32,
  z: u32,
};
fn ceil_div(a: u32, b: u32) -> u32 { return (a + b - 1u) / b; }
// Inspired from https://onrendering.com/data/papers/isubd/isubd.pdf
fn patch_u_child(u: u32, child_bit: u32) -> u32 {
  return (u << 1) | (child_bit & 1);
}
fn patch_top_child(encoded: EncodedPatch) -> EncodedPatch {
  return EncodedPatch(encoded.u, patch_u_child(encoded.v, 0u), encoded.instance);
}
fn patch_bottom_child(encoded: EncodedPatch) -> EncodedPatch {
  return EncodedPatch(encoded.u, patch_u_child(encoded.v, 1u), encoded.instance);
}
fn patch_left_child(encoded: EncodedPatch) -> EncodedPatch {
  return EncodedPatch(patch_u_child(encoded.u, 0u), encoded.v, encoded.instance);
}
fn patch_right_child(encoded: EncodedPatch) -> EncodedPatch {
  return EncodedPatch(patch_u_child(encoded.u, 1u), encoded.v, encoded.instance);
}
fn patch_top_left_child(encoded: EncodedPatch) -> EncodedPatch {
  return patch_top_child(patch_left_child(encoded));
}
fn patch_top_right_child(encoded: EncodedPatch) -> EncodedPatch {
  return patch_top_child(patch_right_child(encoded));
}
fn patch_bottom_left_child(encoded: EncodedPatch) -> EncodedPatch {
  return patch_bottom_child(patch_left_child(encoded));
}
fn patch_bottom_right_child(encoded: EncodedPatch) -> EncodedPatch {
  return patch_bottom_child(patch_right_child(encoded));
}
fn patch_decode(encoded: EncodedPatch) -> Patch {
  // First we go to the implicit 1u
  let leading_zeroes_u = countLeadingZeros(encoded.u);
  let u_bits = extractBits(encoded.u, 0u, 31u - leading_zeroes_u);
  let u_max_bits = u_bits + 1u; // The end position of the patch
  let leading_zeroes_v = countLeadingZeros(encoded.v);
  let v_bits = extractBits(encoded.v, 0u, 31u - leading_zeroes_v);
  let v_max_bits = v_bits + 1u;

  // And every bit after that describes if we go left or right
  // Conveniently, this is already what binary numbers do.
  // 0b0.1 == 0.5
  // 0b0.01 == 0.25
  // 0b0.11 == 0.75
  // And that directly corresponds to how floats work: mantissa * 2^exponent
  // So we can just convert the bits to a float
  // let u = f32(u_bits) * pow(2.0, -1.0 * f32(31 - leading_zeroes_u));
  // And that's equivalent to the size of a patch, see formula below
  let min_value = vec2f(
    f32(u_bits) / f32(1u << (31u - leading_zeroes_u)),
    f32(v_bits) / f32(1u << (31u - leading_zeroes_v))
  );
  let max_value = vec2f(
    f32(u_max_bits) / f32(1u << (31u - leading_zeroes_u)),
    f32(v_max_bits) / f32(1u << (31u - leading_zeroes_v))
  );
  
  // The size of the patch is 1 / 2^(31 - leading_zeroes)
  // let u_size = 1.0 / f32(2 << (31 - leading_zeroes_u));
  // let v_size = 1.0 / f32(2 << (31 - leading_zeroes_v));
  // But we care about this_patch.max == next_patch.min, 
  // so we need to do the floating point calculations more carefully
  
  return Patch(min_value, max_value, encoded.instance);
}

fn assert(condition: bool) {
  // TODO: Implement this
}
//// END OF AUTOGEN

////#include "./EvaluateImage.wgsl"
//// AUTOGEN ced6a506909abff01241bc184a7d6c3a0bede71b7a4d6b1f07430a81dbc637e9
struct Time {
  elapsed: f32,
  delta: f32,
  frame: u32,
}
struct Screen {
  resolution: vec2<u32>,
  inv_resolution: vec2<f32>,
}
struct Mouse {
  pos: vec2<f32>,
  buttons: u32,
}
struct Extra {
  hot_value: f32
}
fn mouse_held(button: u32) -> bool {
  return (mouse.buttons & button) != 0u;
}
// Group 0 is for constants that change once per frame at most
@group(0) @binding(0) var<uniform> time : Time;
@group(0) @binding(1) var<uniform> screen : Screen;
@group(0) @binding(2) var<uniform> mouse : Mouse;
@group(0) @binding(3) var<uniform> extra : Extra;

//// END OF AUTOGEN


alias Vec3Padded = vec4<f32>;

struct Camera {
    world_position: Vec3Padded,
    view: mat4x4<f32>,
    projection: mat4x4<f32>,
}

const LIGHT_TYPE_POINT: u32 = 0;
const LIGHT_TYPE_DIRECTIONAL: u32 = 1;

struct LightSource {
    // position_range.xyz is the position of the light in world space
    // position_range.w is the range of the light
    position_range: vec4f,
    // color.rgb is the color of the light
    // color.a is the intensity of the light
    color_intensity: vec4f,

    light_type: u32
}

struct Lights {
    ambient: Vec3Padded,
    // TODO: Directional light
    points_length: u32,
    points: array<LightSource>,
}

struct VertexInput {
    @location(0) position: vec3<f32>,
    @location(1) uv: vec2<f32>,
    @builtin(instance_index) instance_index: u32,
    @builtin(vertex_index) vertex_index: u32,
}

struct Model {
    model_similarity: mat4x4<f32>,
    object_id: u32
}

struct Material {
    // color.rgb is the color of the material
    // color.a is the roughness of the material
    color_roughness: vec4<f32>,
    // emissive_metallic.rgb is the emissive color of the material
    // emissive_metallic.a is the metallicness of the material
    emissive_metallic: vec4<f32>,
    // is a boolean
    has_texture: u32,
    texture_scale: vec2f
}

@group(0) @binding(4) var<uniform> camera: Camera;
@group(0) @binding(5) var<storage, read> lights: Lights;
@group(0) @binding(6) var linear_sampler: sampler;
@group(1) @binding(1) var<uniform> model: Model;
@group(1) @binding(2) var<storage, read> render_buffer: RenderBufferRead;
@group(1) @binding(3) var<uniform> material: Material;
@group(1) @binding(4) var t_diffuse: texture_2d<f32>;



/**
Code below is from https://github.com/KhronosGroup/glTF-Sample-Viewer/tree/ba079f5a0cf8de9b2371b18a4644ef53eb85ba89


                                 Apache License
                           Version 2.0, January 2004
                        http://www.apache.org/licenses/

   TERMS AND CONDITIONS FOR USE, REPRODUCTION, AND DISTRIBUTION

   1. Definitions.

      "License" shall mean the terms and conditions for use, reproduction,
      and distribution as defined by Sections 1 through 9 of this document.

      "Licensor" shall mean the copyright owner or entity authorized by
      the copyright owner that is granting the License.

      "Legal Entity" shall mean the union of the acting entity and all
      other entities that control, are controlled by, or are under common
      control with that entity. For the purposes of this definition,
      "control" means (i) the power, direct or indirect, to cause the
      direction or management of such entity, whether by contract or
      otherwise, or (ii) ownership of fifty percent (50%) or more of the
      outstanding shares, or (iii) beneficial ownership of such entity.

      "You" (or "Your") shall mean an individual or Legal Entity
      exercising permissions granted by this License.

      "Source" form shall mean the preferred form for making modifications,
      including but not limited to software source code, documentation
      source, and configuration files.

      "Object" form shall mean any form resulting from mechanical
      transformation or translation of a Source form, including but
      not limited to compiled object code, generated documentation,
      and conversions to other media types.

      "Work" shall mean the work of authorship, whether in Source or
      Object form, made available under the License, as indicated by a
      copyright notice that is included in or attached to the work
      (an example is provided in the Appendix below).

      "Derivative Works" shall mean any work, whether in Source or Object
      form, that is based on (or derived from) the Work and for which the
      editorial revisions, annotations, elaborations, or other modifications
      represent, as a whole, an original work of authorship. For the purposes
      of this License, Derivative Works shall not include works that remain
      separable from, or merely link (or bind by name) to the interfaces of,
      the Work and Derivative Works thereof.

      "Contribution" shall mean any work of authorship, including
      the original version of the Work and any modifications or additions
      to that Work or Derivative Works thereof, that is intentionally
      submitted to Licensor for inclusion in the Work by the copyright owner
      or by an individual or Legal Entity authorized to submit on behalf of
      the copyright owner. For the purposes of this definition, "submitted"
      means any form of electronic, verbal, or written communication sent
      to the Licensor or its representatives, including but not limited to
      communication on electronic mailing lists, source code control systems,
      and issue tracking systems that are managed by, or on behalf of, the
      Licensor for the purpose of discussing and improving the Work, but
      excluding communication that is conspicuously marked or otherwise
      designated in writing by the copyright owner as "Not a Contribution."

      "Contributor" shall mean Licensor and any individual or Legal Entity
      on behalf of whom a Contribution has been received by Licensor and
      subsequently incorporated within the Work.

   2. Grant of Copyright License. Subject to the terms and conditions of
      this License, each Contributor hereby grants to You a perpetual,
      worldwide, non-exclusive, no-charge, royalty-free, irrevocable
      copyright license to reproduce, prepare Derivative Works of,
      publicly display, publicly perform, sublicense, and distribute the
      Work and such Derivative Works in Source or Object form.

   3. Grant of Patent License. Subject to the terms and conditions of
      this License, each Contributor hereby grants to You a perpetual,
      worldwide, non-exclusive, no-charge, royalty-free, irrevocable
      (except as stated in this section) patent license to make, have made,
      use, offer to sell, sell, import, and otherwise transfer the Work,
      where such license applies only to those patent claims licensable
      by such Contributor that are necessarily infringed by their
      Contribution(s) alone or by combination of their Contribution(s)
      with the Work to which such Contribution(s) was submitted. If You
      institute patent litigation against any entity (including a
      cross-claim or counterclaim in a lawsuit) alleging that the Work
      or a Contribution incorporated within the Work constitutes direct
      or contributory patent infringement, then any patent licenses
      granted to You under this License for that Work shall terminate
      as of the date such litigation is filed.

   4. Redistribution. You may reproduce and distribute copies of the
      Work or Derivative Works thereof in any medium, with or without
      modifications, and in Source or Object form, provided that You
      meet the following conditions:

      (a) You must give any other recipients of the Work or
          Derivative Works a copy of this License; and

      (b) You must cause any modified files to carry prominent notices
          stating that You changed the files; and

      (c) You must retain, in the Source form of any Derivative Works
          that You distribute, all copyright, patent, trademark, and
          attribution notices from the Source form of the Work,
          excluding those notices that do not pertain to any part of
          the Derivative Works; and

      (d) If the Work includes a "NOTICE" text file as part of its
          distribution, then any Derivative Works that You distribute must
          include a readable copy of the attribution notices contained
          within such NOTICE file, excluding those notices that do not
          pertain to any part of the Derivative Works, in at least one
          of the following places: within a NOTICE text file distributed
          as part of the Derivative Works; within the Source form or
          documentation, if provided along with the Derivative Works; or,
          within a display generated by the Derivative Works, if and
          wherever such third-party notices normally appear. The contents
          of the NOTICE file are for informational purposes only and
          do not modify the License. You may add Your own attribution
          notices within Derivative Works that You distribute, alongside
          or as an addendum to the NOTICE text from the Work, provided
          that such additional attribution notices cannot be construed
          as modifying the License.

      You may add Your own copyright statement to Your modifications and
      may provide additional or different license terms and conditions
      for use, reproduction, or distribution of Your modifications, or
      for any such Derivative Works as a whole, provided Your use,
      reproduction, and distribution of the Work otherwise complies with
      the conditions stated in this License.

   5. Submission of Contributions. Unless You explicitly state otherwise,
      any Contribution intentionally submitted for inclusion in the Work
      by You to the Licensor shall be under the terms and conditions of
      this License, without any additional terms or conditions.
      Notwithstanding the above, nothing herein shall supersede or modify
      the terms of any separate license agreement you may have executed
      with Licensor regarding such Contributions.

   6. Trademarks. This License does not grant permission to use the trade
      names, trademarks, service marks, or product names of the Licensor,
      except as required for reasonable and customary use in describing the
      origin of the Work and reproducing the content of the NOTICE file.

   7. Disclaimer of Warranty. Unless required by applicable law or
      agreed to in writing, Licensor provides the Work (and each
      Contributor provides its Contributions) on an "AS IS" BASIS,
      WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or
      implied, including, without limitation, any warranties or conditions
      of TITLE, NON-INFRINGEMENT, MERCHANTABILITY, or FITNESS FOR A
      PARTICULAR PURPOSE. You are solely responsible for determining the
      appropriateness of using or redistributing the Work and assume any
      risks associated with Your exercise of permissions under this License.

   8. Limitation of Liability. In no event and under no legal theory,
      whether in tort (including negligence), contract, or otherwise,
      unless required by applicable law (such as deliberate and grossly
      negligent acts) or agreed to in writing, shall any Contributor be
      liable to You for damages, including any direct, indirect, special,
      incidental, or consequential damages of any character arising as a
      result of this License or out of the use or inability to use the
      Work (including but not limited to damages for loss of goodwill,
      work stoppage, computer failure or malfunction, or any and all
      other commercial damages or losses), even if such Contributor
      has been advised of the possibility of such damages.

   9. Accepting Warranty or Additional Liability. While redistributing
      the Work or Derivative Works thereof, You may choose to offer,
      and charge a fee for, acceptance of support, warranty, indemnity,
      or other liability obligations and/or rights consistent with this
      License. However, in accepting such obligations, You may act only
      on Your own behalf and on Your sole responsibility, not on behalf
      of any other Contributor, and only if You agree to indemnify,
      defend, and hold each Contributor harmless for any liability
      incurred by, or claims asserted against, such Contributor by reason
      of your accepting any such warranty or additional liability.

   END OF TERMS AND CONDITIONS

   APPENDIX: How to apply the Apache License to your work.

      To apply the Apache License to your work, attach the following
      boilerplate notice, with the fields enclosed by brackets "[]"
      replaced with your own identifying information. (Don't include
      the brackets!)  The text should be enclosed in the appropriate
      comment syntax for the file format. We also recommend that a
      file or class name and description of purpose be included on the
      same "printed page" as the copyright notice for easier
      identification within third-party archives.

   Copyright [yyyy] [name of copyright owner]

   Licensed under the Apache License, Version 2.0 (the "License");
   you may not use this file except in compliance with the License.
   You may obtain a copy of the License at

       http://www.apache.org/licenses/LICENSE-2.0

   Unless required by applicable law or agreed to in writing, software
   distributed under the License is distributed on an "AS IS" BASIS,
   WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
   See the License for the specific language governing permissions and
   limitations under the License.
*/











const M_PI = 3.14159265359;
fn clamped_dot(x: vec3f, y: vec3f) -> f32 {
    return clamp(dot(x, y), 0.0, 1.0);
}

// The following equation models the Fresnel reflectance term of the spec equation (aka F())
// Implementation of fresnel from [4], Equation 15
fn F_Schlick(f0: vec3f, f90: vec3f, VdotH: f32) -> vec3f
{
    return f0 + (f90 - f0) * pow(clamp(1.0 - VdotH, 0.0, 1.0), 5.0);
}

// Smith Joint GGX
// Note: Vis = G / (4 * NdotL * NdotV)
// see Eric Heitz. 2014. Understanding the Masking-Shadowing Function in Microfacet-Based BRDFs. Journal of Computer Graphics Techniques, 3
// see Real-Time Rendering. Page 331 to 336.
// see https://google.github.io/filament/Filament.md.html#materialsystem/specularbrdf/geometricshadowing(specularg)
fn V_GGX(NdotL: f32, NdotV: f32, alphaRoughness: f32) -> f32
{
    let alphaRoughnessSq: f32 = alphaRoughness * alphaRoughness;

    let GGXV: f32 = NdotL * sqrt(NdotV * NdotV * (1.0 - alphaRoughnessSq) + alphaRoughnessSq);
    let GGXL: f32 = NdotV * sqrt(NdotL * NdotL * (1.0 - alphaRoughnessSq) + alphaRoughnessSq);

    let GGX: f32 = GGXV + GGXL;
    if (GGX > 0.0)
    {
        return 0.5 / GGX;
    }
    return 0.0;
}

// The following equation(s) model the distribution of microfacet normals across the area being drawn (aka D())
// Implementation from "Average Irregularity Representation of a Roughened Surface for Ray Reflection" by T. S. Trowbridge, and K. P. Reitz
// Follows the distribution function recommended in the SIGGRAPH 2013 course notes from EPIC Games [1], Equation 3.
fn D_GGX(NdotH: f32, alphaRoughness: f32) -> f32
{
    let alphaRoughnessSq: f32 = alphaRoughness * alphaRoughness;
    let f: f32 = (NdotH * NdotH) * (alphaRoughnessSq - 1.0) + 1.0;
    return alphaRoughnessSq / (M_PI * f * f);
}

//https://github.com/KhronosGroup/glTF/tree/master/specification/2.0#acknowledgments AppendixB
fn BRDF_lambertian(f0: vec3f, f90: vec3f, diffuseColor: vec3f, specularWeight: f32, VdotH: f32) -> vec3f
{
    // see https://seblagarde.wordpress.com/2012/01/08/pi-or-not-to-pi-in-game-lighting-equation/
    return (1.0 - specularWeight * F_Schlick(f0, f90, VdotH)) * (diffuseColor );
}

//  https://github.com/KhronosGroup/glTF/tree/master/specification/2.0#acknowledgments AppendixB
fn BRDF_specularGGX(f0: vec3f, f90: vec3f, alphaRoughness: f32, specularWeight: f32, VdotH: f32, NdotL: f32, NdotV: f32, NdotH: f32) -> vec3f
{
    let F = F_Schlick(f0, f90, VdotH);
    let Vis = V_GGX(NdotL, NdotV, alphaRoughness);
    let D = D_GGX(NdotH, alphaRoughness);

    return specularWeight * F * Vis * D;
}

// https://github.com/KhronosGroup/glTF/blob/master/extensions/2.0/Khronos/KHR_lights_punctual/README.md#range-property
fn getRangeAttenuation(range: f32, distance: f32) -> f32
{
    if (range <= 0.0)
    {
        // negative range means unlimited
        return 1.0 / pow(distance, 2.0);
    }
    return max(min(1.0 - pow(distance / range, 4.0), 1.0), 0.0) / pow(distance, 2.0);
}
fn getLighIntensity(light: LightSource, pointToLight: vec3f) -> vec3f
{
    var rangeAttenuation: f32 = 1.0;
    if(light.light_type != LIGHT_TYPE_DIRECTIONAL) {
      rangeAttenuation = getRangeAttenuation(light.position_range.w, length(pointToLight));
    }
    return rangeAttenuation * light.color_intensity.a * light.color_intensity.rgb;
}

struct MaterialInfo {
    baseColor: vec3f,
    f0: vec3f, // full reflectance color (n incidence angle)
    f90: vec3f, // reflectance color at grazing angle
    c_diff: vec3f,
    alphaRoughness: f32, // squared roughness value

    // KHR_materials_specular
    specularWeight: f32,
}
fn getMetallicRoughnessInfo(info: MaterialInfo, metallic: f32, roughness: f32) -> MaterialInfo
{
    var info1 = info;
    info1.alphaRoughness = roughness * roughness;
    // Achromatic f0 based on IOR.
    info1.c_diff = mix(info1.baseColor.rgb,  vec3f(0.0), metallic);
    info1.f0 = mix(info1.f0, info1.baseColor.rgb, metallic);
    return info1;
}


struct VertexOutput {
    @builtin(position) clip_position: vec4<f32>,
    @location(0) world_normal: vec3<f32>,
    @location(1) world_position: vec3<f32>,
    @location(2) texture_coords: vec2<f32>,
    @location(3) color: vec4<f32>,
}

const color_options = array<vec4f,8>(
    vec4f(1.0, 0.0, 0.0, 1.0),
    vec4f(0.0, 1.0, 0.0, 1.0),
    vec4f(0.0, 0.0, 1.0, 1.0),
    vec4f(1.0, 1.0, 0.0, 1.0),
    vec4f(1.0, 0.0, 1.0, 1.0),
    vec4f(0.0, 1.0, 1.0, 1.0),
    vec4f(1.0, 1.0, 1.0, 1.0),
    vec4f(0.5, 0.5, 0.5, 1.0),
);


@vertex
fn vs_main(
    in: VertexInput,
) -> VertexOutput {
    let quad = patch_decode(render_buffer.patches[in.instance_index]);
    let quad_point = mix(quad.min, quad.max, in.uv);
    instance_id = quad.instance;
    let pos = sampleObject(quad_point);
    let world_pos = model.model_similarity * vec4<f32>(pos, 1.0);


    var out: VertexOutput;
    out.clip_position = camera.projection * camera.view * world_pos;
    out.world_position = world_pos.xyz;
    out.texture_coords = quad_point;
    let normal = vec3<f32>(0.0, -1.0, 0.0); // TODO: We'll compute this later
    out.world_normal = (model.model_similarity * vec4<f32>(normal, 0.0)).xyz; // Only uniform scaling

    let i = in.instance_index % 8;
    if (i == 0) {
    out.color = color_options[0];
    } else if (i == 1) {
    out.color = color_options[1];
    } else if (i == 2) {
    out.color = color_options[2];
    } else if (i == 3) {
    out.color = color_options[3];
    } else if (i == 4) {
    out.color = color_options[4];
    } else if (i == 5) {
    out.color = color_options[5];
    } else if (i == 6) {
    out.color = color_options[6];
    } else if (i == 7) {
    out.color = color_options[7];
    }else {
    out.color = vec4<f32>(1.0, 1.0, 1.0, 1.0);
    }
    return out;
}

struct FragmentOutput {
  @location(0) color: vec4f,
  @location(1) object_id: u32,
}

@fragment
fn fs_main(in: VertexOutput) -> FragmentOutput {
    if(false) {
      // silly hack to get the auto layout to pick up on the uniforms
      let a = sampleObject(vec2f(0.0)); 
    }
    let v = normalize(camera.world_position.xyz - in.world_position);
    // let n = normalize(in.world_normal);
    let n = normalize(-cross(dpdxFine(in.world_position), dpdyFine(in.world_position)));

    var materialInfo = MaterialInfo(
        getColor(in.texture_coords),
        vec3f(0.04),
        vec3f(1.0),
        vec3f(0.0),
        1.0,
        1.0
    );
    materialInfo = getMetallicRoughnessInfo(materialInfo, material.emissive_metallic.a, material.color_roughness.a);

    var f_diffuse = vec3f(0.0);
    var f_specular = vec3f(0.0);
    for (var i: u32 = 0u; i < lights.points_length; i += 1u) {
        let light = lights.points[i];
        var pointToLight: vec3f;
        if (light.light_type != LIGHT_TYPE_DIRECTIONAL)
        {
            pointToLight = light.position_range.xyz - in.world_position;
        }
        else
        {
            pointToLight = -light.position_range.xyz;
        }


        let l = normalize(pointToLight); // Direction from surface point to light
        let h = normalize(l + v);        // Direction of the vector between l and v, called halfway vector
        let intensity: vec3f = getLighIntensity(light, pointToLight);
        let NdotL = clamped_dot(n, l);
        if(NdotL > 0.0) {
            let NdotV = clamped_dot(n, v);
            let NdotH = clamped_dot(n, h);
            let VdotH = clamped_dot(v, h);
            f_diffuse += intensity * NdotL *  BRDF_lambertian(
                materialInfo.f0, 
                materialInfo.f90, 
                materialInfo.c_diff, 
                materialInfo.specularWeight, 
                VdotH
            );
            f_specular += intensity * NdotL * BRDF_specularGGX(
                materialInfo.f0, 
                materialInfo.f90, 
                materialInfo.alphaRoughness, 
                materialInfo.specularWeight, 
                VdotH, 
                NdotL, 
                NdotV, 
                NdotH
            );
        }
    }

    let ambient: vec3f = lights.ambient.rgb * materialInfo.baseColor;

    let color = f_diffuse * 2.0 
        + f_specular * 2.0 
        + ambient 
        + material.emissive_metallic.rgb;

    var fragmentOutput: FragmentOutput;
    fragmentOutput.color = vec4f(color, 1.0);
    fragmentOutput.object_id = model.object_id;
    return fragmentOutput;
    // return in.color; TODO: Why does this cause z-buffer fighting?
}

 

 