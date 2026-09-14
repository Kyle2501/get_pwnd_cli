export function createPS1Material(THREE, colorHex, emissiveHex = 0x000000) {
  return new THREE.ShaderMaterial({
    uniforms: {
      uColor: { value: new THREE.Color(colorHex) },
      uEmissive: { value: new THREE.Color(emissiveHex) },
      uLightDir: { value: new THREE.Vector3(0.5, 1.0, 0.3).normalize() }
    },
    vertexShader: `
      varying vec3 vNormal;
      void main() {
        vNormal = normalMatrix * normal;
        vec4 clipPos = projectionMatrix * modelViewMatrix * vec4(position, 1.0);
        vec4 snapped = clipPos;
        snapped.xyz = clipPos.xyz / clipPos.w;
        snapped.xy = floor(snapped.xy * 120.0) / 120.0;
        snapped.xyz *= clipPos.w;
        gl_Position = snapped;
      }
    `,
    fragmentShader: `
      uniform vec3 uColor;
      uniform vec3 uEmissive;
      uniform vec3 uLightDir;
      varying vec3 vNormal;
      void main() {
        float nDotL = max(dot(normalize(vNormal), uLightDir), 0.0);
        float lightIntensity = nDotL > 0.4 ? 1.0 : 0.35;
        gl_FragColor = vec4((uColor * lightIntensity) + uEmissive, 1.0);
      }
    `
  });
}
