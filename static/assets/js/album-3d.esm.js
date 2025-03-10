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
        cd_case_cover: {
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
        cd_case_cover: {
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

let sceneHdri = null;
let cdCaseModels = {};

// Image assets by name
let assets = {};
// Unique identifier for the currently loaded model
let bandAlbumPath = '';

let isCdCaseOpen = false;

async function loadTexture(url) {
    if (!url) return;
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

    let isAssetLoadError = false;
    
    if (sceneHdri == null) {
        try {
            sceneHdri = await new Promise((resolve, reject) => {
                new RGBELoader()
                    .setPath('/assets/models/hdri/')
                    .load('venice_sunset_1k.hdr', function (texture) {
                        texture.mapping = THREE.EquirectangularReflectionMapping;
                        resolve(texture);
                    }, undefined, reject);
            });
        } catch (error) {
            console.error(error);
            isAssetLoadError = true;
        }
    }

    scene.environment = sceneHdri;

    render();

    if (!cdCaseModels[assets.config.cd_case_type]) {
        try {
            cdCaseModels[assets.config.cd_case_type] = await new Promise((resolve, reject) => {
                const loader = new GLTFLoader().setPath('/assets/models/cd-case/');
                loader.load(`${assets.config.cd_case_type}.glb`, async function (gltf) {
                    resolve(gltf.scene);
                }, undefined, reject);
            });
        } catch (error) {
            console.error(error);
            isAssetLoadError = true;
        }
    }

    try {
        if (isAssetLoadError) {
            throw new Error('Error loading scene or case models.');
        }

        renderer = new THREE.WebGLRenderer({ antialias: true, alpha: true });
        renderer.setPixelRatio(window.devicePixelRatio);
        renderer.setSize(window.innerWidth, window.innerHeight);
        renderer.toneMapping = THREE.ACESFilmicToneMapping;
        renderer.toneMappingExposure = 1;
        renderer.outputEncoding = THREE.sRGBEncoding;
        renderer.setClearColor(0x000000, 0);
        rendererContainer.appendChild(renderer.domElement);

        scene.environment = sceneHdri;
        model = cdCaseModels[assets.config.cd_case_type].clone(true);

        await renderer.compileAsync(model, camera, scene);

        scene.add(model);

        const [
            bookletOutside,
            bookletInside,
            backInsertFront,
            backInsertFrontAlpha,
            backInsertBack,
            backInsertBackAlpha,
            cdFront,
            cdFrontRoughness
        ] = await Promise.all([
            loadTexture(assets.textures.booklet_outside),
            loadTexture(assets.textures.booklet_inside),
            loadTexture(assets.textures.back_insert_front),
            loadTexture(assets.textures.back_insert_front_alpha),
            loadTexture(assets.textures.back_insert_back),
            loadTexture(assets.textures.back_insert_back_alpha),
            loadTexture(assets.textures.cd_front),
            loadTexture(assets.textures.cd_front_roughness),
        ]);

        if (assets.config.cd_case_type === 'slimline-jewel-case') {
            const booklet = getModelPart('booklet');
            booklet.children[0].material.map = bookletOutside;
            booklet.children[0].material.needsUpdate = true;
            booklet.children[1].material.map = bookletInside;
            booklet.children[1].material.needsUpdate = true;
        } else {
            const booklet = getModelPart('booklet');
            booklet.material.map = bookletOutside;
            booklet.material.needsUpdate = true;
        }

        if (backInsertFront && backInsertBack) {
            const backInsert = getModelPart('back_insert');
            backInsert.children[0].material.map = backInsertFront;
            if (backInsertFrontAlpha) {
                backInsert.children[0].material.alphaMap = backInsertFrontAlpha;
                backInsert.children[0].material.alphaTest = 0.5;
            } else {
                backInsert.children[0].material.alphaMap = undefined;
                backInsert.children[0].material.alphaTest = 0;
            }
            backInsert.children[0].material.needsUpdate = true;
            backInsert.children[1].material.map = backInsertBack;
            if (backInsertBackAlpha) {
                backInsert.children[1].material.alphaMap = backInsertBackAlpha;
                backInsert.children[1].material.alphaTest = 0.5;
            } else {
                backInsert.children[1].material.alphaMap = undefined;
                backInsert.children[1].material.alphaTest = 0.0;
            }
            backInsert.children[1].material.needsUpdate = true;
        }

        const cd = getModelPart('cd');
        cd.children[0].material.map = cdFront;
        cd.children[0].material.metalnessMap = cdFrontRoughness;
        cd.children[0].material.roughnessMap = cdFrontRoughness;
        cd.children[0].material.needsUpdate = true;
        if (assets.config.cd_case_type === 'slimline-jewel-case') {
            cd.children[1].material.iridescence = 1.0;
            cd.children[1].material.iridescenceMap = cd.children[1].material.roughnessMap;
            cd.children[1].material.iridescenceIOR = 1.7;
            cd.children[1].material.iridescenceThicknessRange = [100, 800];
            cd.children[1].material.sheen = 1.0;
            cd.children[1].material.sheenRoughness = 0.0;
            cd.children[1].material.thickness = 0.1;
            cd.children[1].material.needsUpdate = true;
        }

        assets.texturesToFree = [
            bookletOutside,
            backInsertFront,
            backInsertFrontAlpha,
            backInsertBack,
            backInsertBackAlpha,
            cdFront,
            cdFrontRoughness
        ];

        if (assets.config.cd_case_disc_holder_color) {
            const cdCaseDiscHolder = getModelPart('cd_case_disc_holder');
            cdCaseDiscHolder.material.dispose();
            cdCaseDiscHolder.material = new THREE.MeshStandardMaterial();
            cdCaseDiscHolder.material.color = new THREE.Color(assets.config.cd_case_disc_holder_color);
            cdCaseDiscHolder.material.needsUpdate = true;
        }

        if (assets.config.cd_case_back_color) {
            const cdCaseBack = getModelPart('cd_case_back');
            cdCaseBack.material.dispose();
            cdCaseBack.material = new THREE.MeshStandardMaterial();
            cdCaseBack.material.color = new THREE.Color(assets.config.cd_case_back_color);
            cdCaseBack.material.needsUpdate = true;
        }

        if (assets.config.cd_case_front_color) {
            const cdCaseFront = getModelPart('cd_case_front');
            cdCaseFront.material.dispose();
            cdCaseFront.material = new THREE.MeshStandardMaterial();
            cdCaseFront.material.color = new THREE.Color(assets.config.cd_case_front_color);
            cdCaseFront.material.needsUpdate = true;
        }


        document.getElementById('album-3d-viewer-loading').remove();
        document.getElementById('album-3d-viewer-controls').removeAttribute('hidden');
    } catch (error) {
        console.error(error);
        handleLoadError();
    }

    controls = new OrbitControls(camera, renderer.domElement);
    controls.addEventListener('change', render);
    controls.minDistance = 2;
    controls.maxDistance = 90;
    controls.target.set(cameraPositions.closed.target.x, cameraPositions.closed.target.y, cameraPositions.closed.target.z);
    controls.update();

    startTransition('closed');
    render();

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
    renderer?.render(scene, camera);
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
    if (!albumCoverPreview.querySelector('album-3d-opener__icon')) {
        albumCoverPreview.innerHTML = albumCoverPreview.innerHTML
            + '<span class="album-3d-opener__icon bi bi-zoom-in" aria-hidden="true"></span>';
    }
    albumCoverPreview.addEventListener('click', loadModelViewer);

    document.addEventListener(
        'htmx:beforeSwap',
        () => {
            document.getElementById('album-3d-viewer')?.remove();
            if (assets?.texturesToFree) {
                for (const texture of assets.texturesToFree) {
                    texture?.dispose();
                }
            }
            assets.texturesToFree = [];
        },
        { once: true },
    );
}

export { init };
