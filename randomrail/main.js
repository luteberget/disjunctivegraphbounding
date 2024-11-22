import * as THREE from 'three';
import { OrbitControls } from 'three/addons/controls/OrbitControls.js';

fetch("l1.json")
  .then(response => response.json())
  .then(main);

function main(data) {
    
}

// const renderer = new THREE.WebGLRenderer();
// renderer.setSize( window.innerWidth, window.innerHeight );
// document.body.appendChild( renderer.domElement );

// const scene = new THREE.Scene();

// const camera = new THREE.PerspectiveCamera( 45, window.innerWidth / window.innerHeight, 1, 10000 );

// const controls = new OrbitControls( camera, renderer.domElement );

// //controls.update() must be called after any manual changes to the camera's transform
// camera.position.set( 0, 20, 100 );
// controls.update();

// function animate() {

// 	requestAnimationFrame( animate );

// 	// required if controls.enableDamping or controls.autoRotate are set to true
// 	controls.update();

// 	renderer.render( scene, camera );

// }

const scene = new THREE.Scene();
scene.background = new THREE.Color( 0xbbbbbb );

const camera = new THREE.PerspectiveCamera( 75, window.innerWidth / window.innerHeight, 0.1, 1000 );

const renderer = new THREE.WebGLRenderer();
renderer.setSize( window.innerWidth, window.innerHeight );
document.body.appendChild( renderer.domElement );

// const geometry = new THREE.BoxGeometry( 1, 1, 1 );
// const material = new THREE.MeshBasicMaterial( { color: 0x00ff00 } );

//create a blue LineBasicMaterial
const blue = new THREE.LineBasicMaterial( { color: 0xff0000, linewidth: 5 } );

// const cube = new THREE.Mesh( geometry, material );
// scene.add( cube );


const geometry = new THREE.BufferGeometry();
const positionAttribute = new THREE.Float32BufferAttribute( [
  - 1,   1, 0,
  - 1, - 1, 0, 
    0,   1, 0,
    0, - 1, 0,
    1,   1, 0,
    1, - 1, 0
], 3 );

// 65535 is the restart index for Uint16
const index = new THREE.Uint16BufferAttribute( [ 0, 1, 2, 65535, 3, 4, 5 ], 1 ); 

geometry.setAttribute( 'position', positionAttribute );
geometry.setIndex( index );

// matLine = new THREE.Line2NodeMaterial( {

//     color: 0xffffff,
//     linewidth: 5, // in world units with size attenuation, pixels otherwise
//     vertexColors: true,
//     dashed: false,
//     alphaToCoverage: true,

// } );

const line = new THREE.Line( geometry, blue );
scene.add( line );


camera.position.z = 5;
camera.position.x = 5;
const controls = new OrbitControls( camera, renderer.domElement );
controls.update();

function animate() {
	renderer.render( scene, camera );
    controls.update();

    // cube.rotation.x += 0.01;
    // cube.rotation.y += 0.01;
}
renderer.setAnimationLoop( animate );