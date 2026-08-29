import createWorker from './dracoWorker.js?worker&inline';

let worker = null;

// The dedicated worker is created lazily on first use so that importing this
// bundle for the pure in-context decode functions (re-exported below) does
// not spawn anything.
function getWorker() {
    if (!worker) {
        worker = createWorker();
        worker.onmessage = (e) => {
            const { id, success, decoded, error, config } = e.data;
            const cb = callbacks.get(id);
            if (!cb) return;

            if (success) {
                if (config) {
                    cb.resolve({ decoded, config });
                } else {
                    cb.resolve(decoded);
                }
            } else {
                cb.reject(error);
            }

            callbacks.delete(id);
        };
    }
    return worker;
}

let requestId = 0;
const callbacks = new Map();

export function decodeDracoMeshInWorker(view, bufferLength) {
    return new Promise((resolve, reject) => {
        const id = requestId++;
        callbacks.set(id, { resolve, reject });

        getWorker().postMessage({ id, view, bufferLength, withConfig: false }, [view.buffer]);
    });
}

export function decodeDracoMeshInWorkerWithConfig(view) {
    return new Promise((resolve, reject) => {
        const id = requestId++;
        callbacks.set(id, { resolve, reject });

        getWorker().postMessage({ id, view, withConfig: true }, [view.buffer]);
    });
}

// Pure in-context decoding lives in the separate `core` bundle entry
// (dracoCore.js → core.es.js) so hosts running inside their own worker do
// not pull the inline-worker copy of the decoder into their scope.
