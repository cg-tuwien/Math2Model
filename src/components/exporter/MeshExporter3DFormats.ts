
import { Document, WebIO } from '@gltf-transform/core';

export class MeshExporter3DFormats {
    vertices: any[];
    tris: number[];
    uvs: any[];
    public useUvs = false;
    constructor(
        vertices: any[],
        tris: any[],
        uvs: any[]
    ) {
        this.vertices = vertices;
        this.tris = tris;
        this.uvs = uvs;
    }

    public async exportModel(format: string) : Promise<any> {
        if (format == "obj") {
            return {data: this.objExport(), binary: false};
        }
        if(format == "glb") {
            return await {data: await this.gltfExport(false), binary: false};
        }
        return "error";
    }

    /**
     * objExport
     */
    public objExport() : string {
        let mexpstring = "o\n";

        for (let vert of this.vertices) {
            mexpstring += "v " + vert.x + " " + vert.y + " " + vert.z + "\n";
        }
        if (this.useUvs) {
            for (let uv of this.uvs) {
                mexpstring += "vt " + uv.x + " " + uv.y + "\n";
            }
        }
        let t = this.tris;
        for (let i = 0; i < t.length; i += 3) {
            mexpstring += "f " + (t[i] + 1) + " " + (t[i + 1] + 1) + " " + (t[i + 2] + 1) + "\n";
        }
        return mexpstring;
    }

    /**
     * gltfExport
     */
    public async gltfExport(binary: boolean) : Promise<Uint8Array> {
                
        const doc = new Document(); 

        const buffer = doc.createBuffer();

        let transformedVerts = new Float32Array(this.vertices.length*3);
        for(let i = 0; i < this.vertices.length; i++)
        {
            let v = this.vertices[i];
            transformedVerts[i*3] = v.x;
            transformedVerts[i*3+1] = v.y;
            transformedVerts[i*3+2] = v.z;
        }
        const position = doc.createAccessor()
        .setType('VEC3')
        .setArray(new Float32Array(transformedVerts))
        .setBuffer(buffer);
        const indices = doc.createAccessor()
        .setType('SCALAR')
        .setArray(new Uint32Array(this.tris))
        .setBuffer(buffer);

        const prim = doc.createPrimitive()
        .setAttribute('POSITION', position)
        .setIndices(indices);

        const mesh = doc.createMesh().addPrimitive(prim);
        const node = doc.createNode().setMesh(mesh);
        const scene = doc.createScene().addChild(node);

        const glb = await new WebIO().writeBinary(doc); 
        return glb;
    }
}