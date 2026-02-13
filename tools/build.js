const { execSync } = require('child_process');
const fs = require('fs');
const path = require('path');

const rootDir = path.join(__dirname, '..');
const jsDir = path.join(rootDir, 'third_party', 'draco_decoder_js');

// Source files
const srcJsFile = path.join(jsDir, 'dist', 'index.es.js');
const srcWasmFile = path.join(jsDir, 'dist', 'draco3d', 'draco_decoder.wasm');

// Destination files
const destJsFile = path.join(rootDir, 'javascript', 'index.es.js');
const destWasmDir = path.join(rootDir, 'javascript', 'draco3d');
const destWasmFile = path.join(destWasmDir, 'draco_decoder.wasm');

// Build draco_decoder_js
console.log('Building draco_decoder_js...');
execSync('npm run build', { cwd: jsDir, stdio: 'inherit' });

// Copy index.es.js
console.log('Copying index.es.js...');
fs.copyFileSync(srcJsFile, destJsFile);
console.log('Copied index.es.js to javascript/');

// Copy draco_decoder.wasm
console.log('Copying draco_decoder.wasm...');
fs.mkdirSync(destWasmDir, { recursive: true });
fs.copyFileSync(srcWasmFile, destWasmFile);
console.log('Copied draco_decoder.wasm to javascript/draco3d/');

// Build Rust
console.log('Building Rust...');
execSync('cargo build', { cwd: rootDir, stdio: 'inherit' });
