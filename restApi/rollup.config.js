import nodeResolve  from '@rollup/plugin-node-resolve';
import commonjs from '@rollup/plugin-commonjs';
import json from '@rollup/plugin-json';

export default {
    input: 'server.js',
    output: {
        inlineDynamicImports: true,
        file: '../bundle.mjs',
        format: 'esm',
    },
    plugins: [
        nodeResolve(),
        commonjs(),
        json()
    ]
}