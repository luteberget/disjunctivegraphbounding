import * as THREE from 'three';
import { OrbitControls } from 'three/addons/controls/OrbitControls.js';

const scene = new THREE.Scene();
scene.background = new THREE.Color( 0xbbbbbb );
const material1 = new THREE.LineBasicMaterial( { color: 0xff0000, linewidth: 3 } );
const material2 = new THREE.LineBasicMaterial( { color: 0x000000, linewidth: 1 } );

fetch("i1.json").then(r => r.json()).then(add_infrastructure);
function add_infrastructure(infrastructure) {

  const points = [];
  for(const node of infrastructure.nodes) {
    points.push(node.location.x, node.location.y, 0.0);
  }

  const index = [];
  for(const res of infrastructure.resources) {

    index.push(res.node_a);
    for(const pt of res.line_segments) {
      index.push(points.length/3);
      points.push(pt.x,pt.y,0.0);
    }
    index.push(res.node_b);
    // 65535 is the restart index for Uint16
    index.push(65535);
  }

  console.log("point", points);
  console.log("index", index);

  const infrastructureGeometry = new THREE.BufferGeometry();
  const positionAttribute = new THREE.Float32BufferAttribute( points, 3 );
  const indexAttribute = new THREE.Uint16BufferAttribute(index, 1 ); 
  infrastructureGeometry.setAttribute( 'position', positionAttribute );
  infrastructureGeometry.setIndex( indexAttribute );
  
  const line = new THREE.Line( infrastructureGeometry, material1 );
  scene.add( line );



  fetch("tt1.json").then(r => r.json()).then(add_timetable);
  function add_timetable(timetable) {

    console.log("TT");
    const time_scale = 1.0 / 3600.0;
    const points = [];
    const index = [];
    for(const train of timetable.trains) {
      for(const [op_idx,op] of train.operations.entries()) {
        const res = infrastructure.resources[op.resource];
        const node_a = infrastructure.nodes[res.node_a];
        const node_b = infrastructure.nodes[res.node_b];
        const pt = op.forward ? node_a.location : node_b.location;
        index.push(points.length/3);
        points.push(pt.x, pt.y, op.time *time_scale);
        if (op_idx + 1 == train.operations.length) {
          const pt = (!op.forward) ? node_a.location : node_b.location;
          index.push(points.length/3);
          points.push(pt.x, pt.y, (op.time + op.min_duration) *time_scale);
        }
      }
      index.push(65535);
    }
    console.log(points);
    console.log(index);

    const geometry = new THREE.BufferGeometry();
    const positionAttribute = new THREE.Float32BufferAttribute( points, 3 );
    const indexAttribute = new THREE.Uint16BufferAttribute(index, 1 ); 
    geometry.setAttribute( 'position', positionAttribute );
    geometry.setIndex( indexAttribute );
    
    const line = new THREE.Line( geometry, material2 );
    scene.add( line );

    for(let i = 0; i < 5; i += 1) {
      const l = new THREE.Line( infrastructureGeometry, material1 );
      l.translateZ(i);
      scene.add( l );
    }
  }
}



const camera = new THREE.PerspectiveCamera( 75, window.innerWidth / window.innerHeight, 0.1, 10000 );

const renderer = new THREE.WebGLRenderer();
renderer.setSize( window.innerWidth, window.innerHeight );
document.body.appendChild( renderer.domElement );

camera.position.z = 500;
camera.position.x = 5;
const controls = new OrbitControls( camera, renderer.domElement );
controls.update();

function animate() {
	renderer.render( scene, camera );
    controls.update();
}
renderer.setAnimationLoop( animate );