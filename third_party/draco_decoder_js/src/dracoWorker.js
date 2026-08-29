import { parseDracoMesh, parseDracoMeshWithConfig } from './dracoCore.js';

self.onmessage = async (e) => {
    const { id, view, bufferLength, withConfig } = e.data;

    try {
        let result;
        if (withConfig) {
            result = await parseDracoMeshWithConfig(view);
            self.postMessage({
                id,
                success: true,
                decoded: result.decoded,
                config: result.config
            }, [result.decoded.buffer]);
        } else {
            const decoded = await parseDracoMesh(view, bufferLength);
            self.postMessage({ id, success: true, decoded }, [decoded.buffer]);
        }
    } catch (err) {
        self.postMessage({ id, success: false, error: err.message });
    }
};
