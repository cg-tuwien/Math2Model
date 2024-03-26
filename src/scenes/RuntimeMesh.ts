import {
    Vector3,
    Quaternion,
    HemisphericLight,
    MeshBuilder,
    Matrix,
    ShaderMaterial,
    ShaderLanguage,
    UniformBuffer,
    ComputeShader,
    GroundMesh,
} from "@babylonjs/core";
import vertexShader from "./ModelDisplayVirtualScene.vert.wgsl?raw";
import type {BaseScene} from "./BaseScene";
import type {SceneFiles} from "@/filesystem/scene-files";
import {assembleFullVertexShader} from "@/shaders/shader-processor";
import type {ModelDisplayVirtualScene} from "./ModelDisplayVirtualScene";
import type {ObjectData} from "@/scenes/ObjectData";
export class RuntimeMesh {
    public mathmodel: GroundMesh;
    constructor(scene: BaseScene, holdingScene: ModelDisplayVirtualScene, public files: SceneFiles, objData: ObjectData) {
        let mathmodel = holdingScene.addDisposable(
            MeshBuilder.CreateGround(
                "ground",
                {
                    width: 3.14159265359 * 2,
                    height: 3.14159265359 * 2,
                    subdivisions: 5,
                },
                scene
            )
        );
        mathmodel.position.y = 10.1;
        mathmodel.position = objData.position;
        mathmodel.rotationQuaternion = objData.rotation;
        mathmodel.name = objData.name;

        mathmodel.thinInstanceAddSelf();
        mathmodel.thinInstanceAdd(
            Matrix.Translation(3.14159265359, 0.0, 3.14159265359)
        );
        mathmodel.thinInstanceCount = 1;

        const customFragmentShader = holdingScene.readOrCreateFile(
            "customFragmentShader",
            () => `
    varying vNormal : vec3<f32>;
    varying vUV : vec2<f32>;
    @fragment
    fn main(input : FragmentInputs) -> FragmentOutputs {
        fragmentOutputs.color = vec4<f32>(input.vUV,1.0, 1.0);
    }
`
        );

        const customVertexShader = holdingScene.readOrCreateFile(
            "customVertexShader"+objData.name,
            () => vertexShader,
            assembleFullVertexShader
        );

        let shaderMaterial = holdingScene.addDisposable(
            new ShaderMaterial(
                "custom",
                scene,
                {
                    vertexSource: customVertexShader,
                    fragmentSource: customFragmentShader,
                },
                {
                    attributes: ["uv", "position", "normal"],
                    uniformBuffers: ["Scene", "Mesh", "instances"],
                    // uniforms: ["iTime", "iTimeDelta", "iFrame", "worldViewProjection"],
                    shaderLanguage: ShaderLanguage.WGSL,
                }
            )
        );
        shaderMaterial.backFaceCulling = false;
        shaderMaterial.wireframe = false;
        const myUBO = holdingScene.addDisposable(new UniformBuffer(scene.engine));
        myUBO.addUniform("iTime", 1);
        myUBO.addUniform("iTimeDelta", 1);
        myUBO.addUniform("iFrame", 1);
        myUBO.addUniform("width", 1);
        myUBO.update();
        shaderMaterial.setUniformBuffer("myUBO", myUBO);
        shaderMaterial.onBind = (m: any) => {
            let x = Math.floor(
                255 / Vector3.Distance(scene.camera.position, mathmodel.position)
            );
            let nextSquareNum = Math.pow(Math.ceil(Math.sqrt(x)), 2);
            myUBO.updateFloat("iTime", scene.time / 1000);
            myUBO.updateFloat("iTimeDelta", scene.deltaTime / 1000);
            myUBO.updateFloat("iFrame", scene.frame);
            myUBO.updateFloat("width", nextSquareNum);
            myUBO.update();
        };
        mathmodel.material = shaderMaterial;

        this.mathmodel = mathmodel;
    }
}