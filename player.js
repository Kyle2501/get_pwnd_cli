import { createPS1Material } from '../shaders/ps1Material.js';

export function createPlayer(THREE) {
  const group = new THREE.Group();
  
  // Torso
  const torso = new THREE.Mesh(
    new THREE.CylinderGeometry(0.35, 0.45, 1.0, 6),
    createPS1Material(THREE, 0xf4d32e)
  );
  torso.position.y = 1.0;
  group.add(torso);

  // Headphones
  const phoneMat = createPS1Material(THREE, 0xff77c6, 0x440028);
  const leftPad = new THREE.Mesh(new THREE.CylinderGeometry(0.15, 0.15, 0.1, 6), phoneMat);
  leftPad.rotation.z = Math.PI / 2;
  leftPad.position.set(-0.38, 1.7, 0);
  
  const rightPad = leftPad.clone();
  rightPad.position.set(0.38, 1.7, 0);
  group.add(leftPad, rightPad);

  return group;
}
