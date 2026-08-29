import wasm from 'vite-plugin-wasm';
import topLevelAwait from 'vite-plugin-top-level-await';
import { viteStaticCopy } from 'vite-plugin-static-copy';

export default {
    build: {
        lib: {
            entry: {
                // Worker-hosted API (spawns the dedicated decoder worker,
                // lazily on first call).
                index: 'src/index.js',
                // Pure in-context decode API — no worker. Embedded by hosts
                // that already run inside their own worker.
                core: 'src/dracoCore.js',
            },
            name: 'DracoDecoderJS',
            fileName: (format, entryName) => `${entryName}.${format}.js`,
            // UMD requires a single entry; ES-only now that there are two.
            formats: ['es'],
        },
        rollupOptions: {
            external: [],
            output: {
                assetFileNames: '[name][extname]', // 保持 wasm 文件名
            },
        },
        minify: 'terser',
        terserOptions: {
            compress: {
                drop_console: true,
                drop_debugger: true
            },
            mangle: true,
        },
    },
    plugins: [
        wasm(),
        topLevelAwait(),
        viteStaticCopy({
            targets: [
                {
                    src: 'node_modules/draco3d/*.wasm',
                    dest: 'draco3d'
                }
            ]
        }),
    ],
};
