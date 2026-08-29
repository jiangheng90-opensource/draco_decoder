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

// Copy core.es.js (pure in-context decode entry, no worker)
const srcCoreFile = path.join(jsDir, 'dist', 'core.es.js');
const destCoreFile = path.join(rootDir, 'javascript', 'core.es.js');
console.log('Copying core.es.js...');
fs.copyFileSync(srcCoreFile, destCoreFile);
console.log('Copied core.es.js to javascript/');

// Copy draco_decoder.wasm
console.log('Copying draco_decoder.wasm...');
fs.mkdirSync(destWasmDir, { recursive: true });
fs.copyFileSync(srcWasmFile, destWasmFile);
console.log('Copied draco_decoder.wasm to javascript/draco3d/');

// The Rust side (src/wasm.rs) embeds these bundles verbatim in a JS template
// literal. It escapes backslashes and backticks itself, but NOT "${", which
// would start a template interpolation in the eval'd code. Fail the build if
// the minifier emitted one (adjust terser options instead of shipping it).
for (const file of [destJsFile, destCoreFile]) {
    const bundle = fs.readFileSync(file, 'utf8');
    const bad = ['${', '`'].filter((s) => bundle.includes(s));
    if (bad.length > 0) {
        throw new Error(
            file + ' contains raw ' + bad.join(' and ') +
            ' sequence(s); adjust the vite/terser config so the bundle is safe to embed'
        );
    }
}
console.log('Bundles are embed-safe.');

// Build Rust
console.log('Building Rust...');
execSync('cargo build', { cwd: rootDir, stdio: 'inherit' });
