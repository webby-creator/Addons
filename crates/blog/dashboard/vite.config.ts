import { defineConfig } from 'vite';
import { svelte } from '@sveltejs/vite-plugin-svelte';
import cssInjectedByJsPlugin from "vite-plugin-css-injected-by-js";

// https://vitejs.dev/config/
export default defineConfig({
    mode: 'development',
    plugins: [svelte(), cssInjectedByJsPlugin()],
});

// build: {
//     lib: {
//       entry: 'src/mycomponent.js',
//       name: 'mycomponent.js',
//       fileName: () => 'mycomponent.js',
//       formats: ['iife'],
//     },
//     cssCodeSplit: false,
//     rollupOptions: {
//       plugins: [
//         {
//           apply: 'build',
//           enforce: 'post',
//           name: 'pack-css',
//           generateBundle(opts, bundle) {
//             const {
//               [css_filename]: { source: rawCss },
//               [bundle_filename]: component,
//             } = bundle

//             const IIFEcss = `
//             (function() {
//               try {
//                   var elementStyle = document.createElement('style');
//                   elementStyle.innerText = ${JSON.stringify(rawCss)}
//                   document.head.appendChild(elementStyle)
//               } catch(error) {
//                 console.error(error, 'unable to concat style inside the bundled file')
//               }
//             })()`

//             component.code += IIFEcss

//             // remove from final bundle
//             delete bundle[css_filename]
//           },
//         },
//       ],
//     },
//   },