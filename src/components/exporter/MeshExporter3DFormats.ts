
import { Document, WebIO } from '@gltf-transform/core';

export class MeshExporter3DFormats {
    meshBuffer: any[];
    public useUvs = false;
    public merge: boolean = false;
    constructor(
        meshBuffer: any[], merge: boolean
    ) {
        this.meshBuffer = meshBuffer;
        this.merge = merge;
    }

    public async exportModel(format: string) : Promise<any> {
        if (format == "obj") {
            return {data: this.objExport(this.meshBuffer), binary: false};
        }
        if(format == "glb") {
            return await {data: await this.gltfExport(this.meshBuffer,false), binary: false};
        }
        return "error";
    }

    /**
     * objExport
     */
    public objExport(meshes: any[]) : string {

        let mexpstring = "";
        let offset = 1;

        for(let i = 0; i < meshes.length; i++)
        {
            let buf = this.meshBuffer[i];
         
            let mexpstring = "o" + buf.name + "\n";
            for (let vert of buf.verts) {
                mexpstring += "v " + vert.x + " " + vert.y + " " + vert.z + "\n";
            }
            if (this.useUvs) {
                for (let uv of buf.uvs) {
                    mexpstring += "vt " + uv.x + " " + uv.y + "\n";
                }
            }
            let t = buf.tris;
            for (let i = 0; i < t.length; i += 3) {
                mexpstring += "f " + (t[i] + offset) + " " + (t[i + 1] + offset) + " " + (t[i + 2] + offset) + "\n";
            }
            offset+=buf.verts.length;
        }
        return mexpstring;
    }

    /**
     * gltfExport
     */
    public async gltfExport(meshes: any[], binary: boolean) : Promise<Uint8Array> {
        const doc = new Document(); 

        const scene = doc.createScene();
        for(let x = 0; x < meshes.length; x++)
        {
            let meshBuf = meshes[x];
            const buffer = doc.createBuffer();

            let transformedVerts = new Float32Array(meshBuf.verts.length*3);
            for(let i = 0; i < meshBuf.verts.length; i++)
            {
                let v = meshBuf.verts[i];
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
            .setArray(new Uint32Array(meshBuf.tris))
            .setBuffer(buffer);

            const prim = doc.createPrimitive()
            .setAttribute('POSITION', position)
            .setIndices(indices);

            const mesh = doc.createMesh().addPrimitive(prim);
            mesh.setName(meshBuf.name);
            const node = doc.createNode().setMesh(mesh);
            scene.addChild(node);
        }
        const glb = await new WebIO().writeBinary(doc); 
        return glb;
    }
}