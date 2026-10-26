// Execute only after the Rust artifacts and wasm-bindgen glue were built on pool.
// Usage: node test.mjs /absolute/packaged-web-directory /absolute/result.json
import {createServer} from 'node:http';
import {readFile,writeFile} from 'node:fs/promises';
import {resolve,extname,sep} from 'node:path';
import {pathToFileURL} from 'node:url';
const root=resolve(process.argv[2]??'.'),resultPath=resolve(process.argv[3]??'browser-result.json');
const playwright=await import(process.env.PLAYWRIGHT_PATH?pathToFileURL(process.env.PLAYWRIGHT_PATH).href:'playwright');
const mime={'.html':'text/html','.mjs':'text/javascript','.js':'text/javascript','.wasm':'application/wasm','.json':'application/json'};
const server=createServer(async(req,res)=>{
    try{
        const url=new URL(req.url,'http://localhost');
        const file=resolve(root,'.'+decodeURIComponent(url.pathname==='/'?'/qualification.html':url.pathname));
        if(file!==root&&!file.startsWith(root+sep)){res.writeHead(403);res.end();return;}
        if(!url.searchParams.has('unisolated')){
            res.setHeader('Cross-Origin-Opener-Policy','same-origin');res.setHeader('Cross-Origin-Embedder-Policy','require-corp');
        }
        res.setHeader('Content-Type',mime[extname(file)]??'application/octet-stream');res.end(await readFile(file));
    }catch(error){res.writeHead(404);res.end(String(error));}
});
await new Promise(resolve=>server.listen(0,'127.0.0.1',resolve));
const browser=await playwright.chromium.launch({executablePath:process.env.PLAYWRIGHT_EXECUTABLE_PATH,headless:true,args:['--autoplay-policy=no-user-gesture-required']});
const errors=[],messages=[];
try{
    const page=await browser.newPage();page.on('pageerror',error=>errors.push(String(error)));page.on('console',message=>messages.push({type:message.type(),text:message.text()}));
    await page.goto(`http://127.0.0.1:${server.address().port}/qualification.html`);
    await page.waitForFunction(()=>typeof window.runQualification==='function');
    const result=await page.evaluate(()=>window.runQualification());
    if(!result.pass||errors.length)throw Error(`qualification failed: ${JSON.stringify(errors)}`);
    await writeFile(resultPath,JSON.stringify({...result,browser:browser.version(),errors,messages},null,2)+'\n');
    console.log(JSON.stringify({pass:true,resultPath,stages:result.stages.map(v=>v.label),browser:browser.version()}));
}catch(error){await writeFile(resultPath,JSON.stringify({pass:false,error:String(error),errors,messages},null,2)+'\n');throw error;}
finally{await browser.close();await new Promise(resolve=>server.close(resolve));}
