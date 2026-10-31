import { cpSync, mkdirSync, existsSync } from 'node:fs';
import { fileURLToPath } from 'node:url';
import { join } from 'node:path';
const root=fileURLToPath(new URL('../',import.meta.url));
for(const name of ['cmaps','standard_fonts','wasm']){
 const source=join(root,'node_modules','pdfjs-dist',name),target=join(root,'web','public','pdfjs',name);
 if(!existsSync(source))throw new Error('PDF.js assets are missing. Run npm install first.');
 mkdirSync(target,{recursive:true});cpSync(source,target,{recursive:true});
}
