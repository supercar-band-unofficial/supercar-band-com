import * as THREE from '/assets/js/threejs/three.module.js';

import { OrbitControls } from '/assets/js/threejs/jsm/controls/OrbitControls.js';
import { GLTFLoader } from '/assets/js/threejs/jsm/loaders/GLTFLoader.js';
import { RGBELoader } from '/assets/js/threejs/jsm/loaders/RGBELoader.js';

let camera, scene, renderer, controls, model;

let cameraPositions = {
    closed: {
        target: { x: 6.5, y: 6.2, z: -0.2 },
        position: { x: 6.5, y: 6.2, z: 25 },
    },
    open: {
        target: { x: 0, y: 6.2, z: -0.2 },
        position: { x: 0, y: 6.2, z: 35 },
    },
};
let modelTransitions = {
    closed: {
        booklet: {
            rotation: { x: 0, y: 0, z: 0 },
            pivot: new THREE.Vector3(0, 0, 0),
        },
        cd_case_front: {
            rotation: { x: 0, y: 0, z: 0 },
            pivot: new THREE.Vector3(0, 0, 0),
        },
    },
    open: {
        booklet: {
            rotation: { x: 0, y: -Math.PI, z: 0 },
            pivot: new THREE.Vector3(0, 0, 0),
        },
        cd_case_front: {
            rotation: { x: 0, y: -Math.PI, z: 0 },
            pivot: new THREE.Vector3(0, 0, 0),
        },
    }
};
let cameraTransitionTimeMax = 1.0;
let cameraTransitionTime = cameraTransitionTimeMax;
let cameraTransitionStartTimestamp = 0;
let cameraTransitionStartPosition = null;
let cameraTransitionStartTarget = null;
let modelTransitionStart = null;
let currentTransitionName = 'closed';
let previousTransitionName = 'closed';

// Image assets by name
let assets = {};
// Unique identifier for the currently loaded model
let bandAlbumPath = '';

let isCdCaseOpen = false;

async function loadTexture(url) {
    const loader = new THREE.TextureLoader();
    const texture = await loader.loadAsync(url);
    texture.flipY = false;
    texture.colorSpace = THREE.SRGBColorSpace;
    texture.generateMipmaps = true;
    return texture;
}

function handleLoadError() {
    document.getElementById('album-3d-viewer-loading').textContent = 'An error occurred when loading the 3D model. Sorry about that.';
    document.getElementById('album-3d-viewer-controls').removeAttribute('hidden');
}

async function loadModelViewer() {
    let modelViewerContainer = document.getElementById('album-3d-viewer');
    if (modelViewerContainer?.getAttribute('data-band-album') === bandAlbumPath) {
        modelViewerContainer?.removeAttribute('hidden');
        return;
    }
    modelViewerContainer?.remove();

    cameraTransitionStartTarget = null;
    modelTransitionStart = null;
    currentTransitionName = 'closed';
    previousTransitionName = 'closed';
    isCdCaseOpen = false;

    modelViewerContainer = document.createElement('div');
    modelViewerContainer.classList.add('album-3d-viewer');
    modelViewerContainer.setAttribute('id', 'album-3d-viewer');
    modelViewerContainer.setAttribute('data-band-album', bandAlbumPath);
    modelViewerContainer.innerHTML = `
        <div id="album-3d-viewer-loading" class="album-3d-viewer__loading">Loading 3D Model. Please Wait.</div>
		<div id="album-3d-viewer-controls" class="album-3d-viewer__controls" hidden>
			<button id="album-3d-viewer-open-close-button">CD Case: Closed</button>
			<button id="album-3d-viewer-cd-button">CD: Visible</button>
            <button id="album-3d-viewer-exit-button">Exit</button>
		</div>
        <div id="album-3d-viewer-renderer" class="album-3d-viewer__renderer"></div>
    `;
    document.body.appendChild(modelViewerContainer);

    const rendererContainer = document.getElementById('album-3d-viewer-renderer');

    camera = new THREE.PerspectiveCamera( 45, window.innerWidth / window.innerHeight, 0.25, 100 );
    camera.position.set(cameraPositions.closed.position.x, cameraPositions.closed.position.y, cameraPositions.closed.position.z);

    scene = new THREE.Scene();

    new RGBELoader()
        .setPath('/assets/models/hdri/')
        .load('venice_sunset_1k.hdr', function (texture) {

            texture.mapping = THREE.EquirectangularReflectionMapping;
            scene.environment = texture;

            render();

            const loader = new GLTFLoader().setPath('/assets/models/cd-case/');
            loader.load('jewel-case.glb', async function (gltf) {

                try {
                    model = gltf.scene;

                    await renderer.compileAsync(model, camera, scene);

                    scene.add(model);

                    const [bookletOutside, backInsertFront, backInsertBack, cdFront, cdFrontRoughness] = await Promise.all([
                        loadTexture(assets.textures.booklet_outside),
                        loadTexture(assets.textures.back_insert_front),
                        loadTexture(assets.textures.back_insert_back),
                        loadTexture(assets.textures.cd_front),
                        loadTexture(assets.textures.cd_front_roughness),
                    ]);

                    const booklet = getModelPart('booklet');
                    booklet.material.map = bookletOutside;
                    booklet.material.needsUpdate = true;

                    const backInsert = getModelPart('back_insert');
                    backInsert.children[0].material.map = backInsertFront;
                    backInsert.children[0].material.needsUpdate = true;
                    backInsert.children[1].material.map = backInsertBack;
                    backInsert.children[1].material.needsUpdate = true;

                    const cd = getModelPart('cd');
                    cd.children[0].material.map = cdFront;
                    cd.children[0].material.metalnessMap = cdFrontRoughness;
                    cd.children[0].material.roughnessMap = cdFrontRoughness;
                    cd.children[0].material.needsUpdate = true;

                    assets.texturesToFree = [bookletOutside, backInsertFront, backInsertBack, cdFront, cdFrontRoughness];

                    render();

                    document.getElementById('album-3d-viewer-loading').remove();
                    document.getElementById('album-3d-viewer-controls').removeAttribute('hidden');
                } catch (error) {
                    handleLoadError();
                }
            }, undefined, handleLoadError);

        }, undefined, handleLoadError);

    renderer = new THREE.WebGLRenderer({ antialias: true, alpha: true });
    renderer.setPixelRatio(window.devicePixelRatio);
    renderer.setSize(window.innerWidth, window.innerHeight);
    renderer.toneMapping = THREE.ACESFilmicToneMapping;
    renderer.toneMappingExposure = 1;
    renderer.outputEncoding = THREE.sRGBEncoding;
    renderer.setClearColor(0x000000, 0);
    rendererContainer.appendChild(renderer.domElement);

    controls = new OrbitControls(camera, renderer.domElement);
    controls.addEventListener('change', render);
    controls.minDistance = 2;
    controls.maxDistance = 90;
    controls.target.set(cameraPositions.closed.target.x, cameraPositions.closed.target.y, cameraPositions.closed.target.z);
    controls.update();

    window.addEventListener('resize', onWindowResize);

    const openCloseButton = document.getElementById('album-3d-viewer-open-close-button');
    openCloseButton.addEventListener('click', () => {
        if (isCdCaseOpen) {
            isCdCaseOpen = false;
            openCloseButton.textContent = 'CD Case: Closed';
            startTransition('closed');
        } else {
            isCdCaseOpen = true;
            openCloseButton.textContent = 'CD Case: Open';
            startTransition('open');
        }
    });

    const cdButton = document.getElementById('album-3d-viewer-cd-button');
    cdButton.addEventListener('click', () => {
        const cdModel = getModelPart('cd');
        cdModel.visible = !cdModel.visible;
        if (cdModel.visible) {
            cdButton.textContent = 'CD: Visible';
        } else {
            cdButton.textContent = 'CD: Hidden';
        }
        render();
    });

    const exitButton = document.getElementById('album-3d-viewer-exit-button');
    exitButton.addEventListener('click', () => {
        const album3dViewer = document.getElementById('album-3d-viewer');
        if (!album3dViewer) return;
        album3dViewer.setAttribute('hidden', 'hidden');
    });
}

function onWindowResize() {

    camera.aspect = window.innerWidth / window.innerHeight;
    camera.updateProjectionMatrix();

    renderer.setSize(window.innerWidth, window.innerHeight);

    render();

}

function lerp(start, end, t) {
    t = t < 0.5 ? 4 * t * t * t : 1 - Math.pow(-2 * t + 2, 3) / 2;
    return start + t * (end - start);
}

function getModelPart(partName) {
    for (const child of model.children) {
        if (child.name === partName) return child;
    }
}

function startTransition(transitionName) {
    previousTransitionName = currentTransitionName;
    currentTransitionName = transitionName;
    cameraTransitionStartTimestamp = window.performance.now();
    cameraTransitionStartPosition = { ...camera.position };
    cameraTransitionStartTarget = { x: controls.target.x, y: controls.target.y, z: controls.target.z };
    modelTransitionStart = {};
    if (modelTransitions[currentTransitionName]) {
        for (let modelName in modelTransitions[currentTransitionName]) {
            const modelPart = getModelPart(modelName);
            if (!modelPart) continue;
            let rotation;
            if (modelTransitions[previousTransitionName]?.[modelName]?.rotation) {
                rotation = modelTransitions[previousTransitionName][modelName].rotation;
            } else {
                var rotationMatrix = new THREE.Matrix4();
                rotationMatrix.extractRotation(modelPart.matrix.clone());
                rotation = new THREE.Euler();
                rotation.setFromRotationMatrix(rotationMatrix);
            }
            modelTransitionStart[modelName] = {
                position: modelPart.position.clone(),
                rotation,
            };
        }
    }
    transitionCamera();
}

function transitionCamera() {
    let cameraTransitionTime = (window.performance.now() - cameraTransitionStartTimestamp) / 1000;
    cameraTransitionTime = Math.min(cameraTransitionTime, cameraTransitionTimeMax);
    const delta = cameraTransitionTime / cameraTransitionTimeMax;
    if (cameraTransitionTime < cameraTransitionTimeMax) {
        window.requestAnimationFrame(() => {
            transitionCamera();
        });
    }
    if (cameraPositions[currentTransitionName]) {
        const oldCameraPosition = cameraPositions[previousTransitionName]?.position ?? cameraPositions[currentTransitionName].position;
        const oldCameraTarget = cameraPositions[previousTransitionName]?.target ?? cameraPositions[currentTransitionName].target;
        const cameraPosition = cameraPositions[currentTransitionName].position;
        const cameraTarget = cameraPositions[currentTransitionName].target;
        camera.position.set(
            lerp(cameraTransitionStartPosition.x, cameraPosition.x, delta),
            lerp(cameraTransitionStartPosition.y, cameraPosition.y, delta),
            lerp(cameraTransitionStartPosition.z, cameraPosition.z, delta),
        );
        camera.needsUpdate = true;
        controls.target.set(
            lerp(cameraTransitionStartTarget.x, cameraTarget.x, delta),
            lerp(cameraTransitionStartTarget.y, cameraTarget.y, delta),
            lerp(cameraTransitionStartTarget.z, cameraTarget.z, delta),
        );
        controls.update();
    }
    if (modelTransitions[currentTransitionName]) {
        for (let modelName in modelTransitions[currentTransitionName]) {
            const modelPart = getModelPart(modelName);
            if (!modelPart) continue;
            const modelStart = modelTransitionStart[modelName];
            const modelTransition = modelTransitions[currentTransitionName][modelName];
            if (modelTransition.rotation && modelTransition.pivot) {
                const startPosition = modelStart.position;
                const relativePosition = modelTransition.pivot;
                modelPart.matrixAutoUpdate = false;
                modelPart.matrix = (
                    new THREE.Matrix4().multiply(
                        new THREE.Matrix4().makeTranslation(-relativePosition.x, -relativePosition.y, -relativePosition.z)
                    ).multiply(
                        new THREE.Matrix4().makeRotationX(
                            lerp(modelStart.rotation.x, modelTransition.rotation.x, delta)
                        )
                    ).multiply(
                        new THREE.Matrix4().makeRotationY(
                            lerp(modelStart.rotation.y, modelTransition.rotation.y, delta)
                        )
                    ).multiply(
                        new THREE.Matrix4().makeRotationZ(
                            lerp(modelStart.rotation.z, modelTransition.rotation.z, delta)
                        )
                    ).multiply(
                        new THREE.Matrix4().makeTranslation(relativePosition.x, relativePosition.y, relativePosition.z),
                    ).multiply(
                        new THREE.Matrix4().makeTranslation(startPosition.x, startPosition.y, startPosition.z)
                    )
                );
            }
        }
    }
    render();
}

function render() {
    renderer.render( scene, camera );
}

async function init() {
    const pathBeforeRequest = window.location.pathname;
    bandAlbumPath = window.location.pathname.split('/').slice(2, 4).join('/');

    const assetsRequest = await fetch(`/album-3d/${bandAlbumPath}/assets.json`);
    if (!assetsRequest.ok || pathBeforeRequest !== window.location.pathname) return;
    assets = await assetsRequest.json();

    const albumCoverPreview = document.getElementById('album-cover-preview');
    albumCoverPreview.classList.add('album-3d-opener');
    albumCoverPreview.setAttribute('role', 'button');
    albumCoverPreview.setAttribute('tabindex', '0');
    albumCoverPreview.innerHTML = albumCoverPreview.innerHTML
        + '<span class="album-3d-opener__icon bi bi-zoom-in" aria-hidden="true"></span>';
    albumCoverPreview.addEventListener('click', loadModelViewer);

    document.addEventListener(
        'htmx:beforeSwap',
        () => {
            document.getElementById('album-3d-viewer')?.remove();
            if (assets?.texturesToFree) {
                for (const texture of assets.texturesToFree) {
                    texture.dispose();
                }
            }
            assets.texturesToFree = [];
        },
        { once: true },
    );
}

export { init };
