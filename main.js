import { createPlayer } from './entities/player.js';

export function initGame(THREE) {
  const scene = new THREE.Scene();
  const player = createPlayer(THREE);
  scene.add(player);
  return { scene, player };
}
