import { vec3, mat4 } from "webgpu-matrix";
import { Document, WebIO } from "@gltf-transform/core";

export class MeshExporter3DFormats {
  meshBuffer: any[];
  public useUvs = false;
  public merge: boolean = false;
  public name: string = "";
  public progressToMake: number = 0;
  public exportProgress: any = {value:0};
  
  constructor(meshBuffer: any[]) {
    this.meshBuffer = meshBuffer;
  }

  public async exportModel(
    format: string
  ): Promise<{
    data: Uint8Array | string;
    binary: boolean;
    error?: boolean | undefined;
    errors?: string[] | undefined;
    extraFile?: { data: string | Uint8Array; fileExtension: string } | undefined;
  }> {
    if (format == "obj") {
      let file = this.objExport(this.meshBuffer);
      return {
        data: file[0],
        binary: false,
        extraFile: { data: file[1], fileExtension: "mtl" },
      };
    }
    if (format == "glb") {
      let outputData = await this.gltfExport(this.meshBuffer, false);
      let errors = outputData.errors;
      let data = outputData.data;
      return await {
        data: data,
        binary: false,
        error: data.length == 0 || this.meshBuffer.length == errors.length,
        errors: errors,
      };
    }
    return {data:"",binary:false,error:true};
  }

  /**
   * objExport
   */
  public objExport(meshes: any[]): [string, string] {
    let progressPerMesh = 1./meshes.length*this.progressToMake;
    
    let mexpstring = "mtllib " + this.name + ".mtl\n";
    let offset = 1;

    for (let i = 0; i < meshes.length; i++) {
      let buf = this.meshBuffer[i];

      mexpstring += "o " + buf.name + "\n";
      mexpstring += "usemtl Material" + i + "\n";
      let matrix = transformPointMatrix(buf.rotation, buf.position, buf.scale);
      let progressPerVert = progressPerMesh/buf.verts.length/3;
      for (let vert of buf.verts) {
        //        debugger;
        let vecResult = [vert.x, vert.y, vert.z];
        vec3.transformMat4(vecResult, matrix, vecResult);
        mexpstring +=
          "v " + vecResult[0] + " " + vecResult[1] + " " + vecResult[2] + "\n";
        this.exportProgress.value += progressPerVert;
      }
      if (this.useUvs) {
        for (let uv of buf.uvs) {
          mexpstring += "vt " + uv.x + " " + uv.y + "\n";
        }
        this.exportProgress.value+=progressPerMesh/3;
      }
      let progressPerTri = progressPerMesh/buf.tris.length/3;

      let t = buf.tris;
      for (let i = 0; i < t.length; i += 3) {
        mexpstring +=
          "f " +
          (t[i] + offset) +
          " " +
          (t[i + 1] + offset) +
          " " +
          (t[i + 2] + offset) +
          "\n";
        this.exportProgress.value+=progressPerTri;
      }
      offset += buf.verts.length;
    }
    let mtlFile = "";
    let i = 0;
    meshes.forEach((mesh) => {
      let mat = mesh.material;
      mtlFile += "newmtl Material" + i + "\n";
      mtlFile += "Kd " + mat.color.join(" ") + "\n";
      mtlFile += "Pm " + mat.metallic + "\n";
      mtlFile += "Pr " + mat.roughness + "\n";
      i++;
    });

    return [mexpstring, mtlFile];
  }

  /**
   * gltfExport
   */
  public async gltfExport(
    meshes: any[],
    binary: boolean
  ): Promise<{ data: Uint8Array; errors: string[] }> {
    const doc = new Document();

    //    debugger;
    const scene = doc.createScene();
    const buffer = doc.createBuffer();
    let errorModels = [];
    let progressPerMesh = 1./meshes.length*this.progressToMake;
    for (let x = 0; x < meshes.length; x++) {
      let meshBuf = meshes[x];
      if (!(meshBuf.verts.length > 0)) {
        errorModels.push(meshBuf.name);
        console.log("Vert buffer for ",meshBuf,"is empty, cant export");
        continue;
      }
      let inMaterial = meshBuf.material;
      if (!inMaterial) {
        alert("HELP I NEED A MATERIAL");
      }
      let materialName = "material" + x;
      let material = doc
        .createMaterial(materialName)
        .setBaseColorFactor([
          clamp01(inMaterial.color[0]),
          clamp01(inMaterial.color[1]),
          clamp01(inMaterial.color[2]),
          1,
        ])
        .setEmissiveFactor(
          [
            clamp01(inMaterial.emissive[0]),
            clamp01(inMaterial.emissive[1]),
            clamp01(inMaterial.emissive[2])
          ])
        .setMetallicFactor( clamp01(inMaterial.metallic))
        .setRoughnessFactor(clamp01(inMaterial.roughness));
      let transformedVerts = new Float32Array(meshBuf.verts.length * 3);
      let progressPerVert = progressPerMesh/meshBuf.verts.length/2;
      for (let i = 0; i < meshBuf.verts.length; i++) {
        let v = meshBuf.verts[i];
        transformedVerts[i * 3] = v.x;
        transformedVerts[i * 3 + 1] = v.y;
        transformedVerts[i * 3 + 2] = v.z;
        this.exportProgress.value+=progressPerVert;
      }
      let transformedUvs = new Float32Array(meshBuf.uvs.length * 2);
      if (this.useUvs)
        for (let i = 0; i < meshBuf.uvs.length; i++) {
          let uv = meshBuf.uvs[i];
          transformedUvs[i * 2] = uv.x;
          transformedUvs[i * 2 + 1] = uv.y;
        }
      const position = doc
        .createAccessor()
        .setType("VEC3")
        .setArray(new Float32Array(transformedVerts))
        .setBuffer(buffer);
      const indices = doc
        .createAccessor()
        .setType("SCALAR")
        .setArray(new Uint32Array(meshBuf.tris))
        .setBuffer(buffer);
      // console.log("Exporter received tris: ", meshBuf.tris.length + " For mesh " + meshBuf.name);

      const prim = doc
        .createPrimitive()
        .setAttribute("POSITION", position)
        .setIndices(indices);
      if (this.useUvs) {
        let uvs = doc
          .createAccessor()
          .setType("VEC2")
          .setArray(new Float32Array(transformedUvs))
          .setBuffer(buffer);
        prim.setAttribute("TEXCOORD_0", uvs);
      }
      prim.setMaterial(material);

      const mesh = doc.createMesh().addPrimitive(prim);
      mesh.setName(meshBuf.name);
      const node = doc.createNode().setMesh(mesh);
      node
        .setTranslation(meshBuf.position)
        .setScale([meshBuf.scale, meshBuf.scale, meshBuf.scale])
        .setRotation(eulerToQuaternion(meshBuf.rotation));
      scene.addChild(node);
    }
    const glb = await new WebIO().writeBinary(doc);
    return { data: glb, errors: errorModels };
  }
}

function eulerToQuaternion(
  euler: [number, number, number]
): [number, number, number, number] {
  const [x, y, z] = euler;

  const cx = Math.cos(x / 2);
  const sx = Math.sin(x / 2);
  const cy = Math.cos(y / 2);
  const sy = Math.sin(y / 2);
  const cz = Math.cos(z / 2);
  const sz = Math.sin(z / 2);

  return [
    sx * cy * cz - cx * sy * sz, // x
    cx * sy * cz + sx * cy * sz, // y
    cx * cy * sz - sx * sy * cz, // z
    cx * cy * cz + sx * sy * sz, // w
  ];
}
function transformPointMatrix(rotation: any, translation: any, scale: number) {
  //    debugger;
  let rotMatX = mat4.rotationX(rotation[0]);
  mat4.rotateY(rotMatX, rotation[1], rotMatX);
  mat4.rotateZ(rotMatX, rotation[2], rotMatX);
  let matrix = rotMatX;
  mat4.translate(matrix, translation, matrix); // Multiplication does not seem to function?
  mat4.scale(matrix, [scale, scale, scale], matrix);
  return matrix;
}
function clamp01(value: number): number
{
  return Math.min(1.,Math.max(0.,value));
}