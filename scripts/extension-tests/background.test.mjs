import fs from 'node:fs';
import vm from 'node:vm';
import assert from 'node:assert/strict';
import test from 'node:test';
import { webcrypto } from 'node:crypto';
const read=n=>fs.readFileSync(new URL('../../lap-extension/'+n,import.meta.url),'utf8');
const settle=async(n=12)=>{for(let i=0;i<n;i++)await new Promise(r=>setImmediate(r));};
function harness(config={}) {
 const data={token:'test-fixture-token',apiUrl:'http://127.0.0.1:47821',folderPath:'D:/assets',recentFolders:[],tags:['Default'],...config};let listener;const calls=[];
 const chrome={runtime:{id:'fixture-extension',onInstalled:{addListener(){}},onMessage:{addListener(f){listener=f}},openOptionsPage:async()=>{},getPlatformInfo:async()=>({os:'win'})},storage:{local:{get:async q=>typeof q==='string'?{[q]:data[q]}:{...q,...data},set:async obj=>Object.assign(data,structuredClone(obj))}}};
 const context=vm.createContext({chrome,URL,Headers,AbortController,crypto:webcrypto,structuredClone,setTimeout,clearTimeout,setInterval,clearInterval,console,fetch:async(url,opts)=>{calls.push({url,opts});return {ok:true,status:200,json:async()=>url.endsWith('/folders')?{folders:[{name:'Assets',path:'D:/assets'}]}:{ok:true}};}});
 vm.runInContext(read('api.js').replaceAll('export ',''),context);
 return {context,chrome,data,calls,setFetch:f=>context.fetch=f,background(){vm.runInContext(read('background.js').replace(/^import[\s\S]*?from ['"]\.\/api\.js['"];?\s*/,''),context);},message(message){return new Promise(resolve=>listener(message,{id:'fixture-extension'},resolve));}};
}
test('Pairing request keeps authentication and allows only existing loopback service',async()=>{
  const h=harness();await vm.runInContext("request('/folders')",h.context);assert.equal(h.calls[0].opts.headers.get('X-Lap-Token'),'test-fixture-token');
  await assert.rejects(vm.runInContext("request('/folders',{config:{apiUrl:'https://example.com',token:'fixture'}})",h.context),/采集服务地址/);assert.equal(h.calls.length,1);
 });
test('Offline, unauthenticated, malformed and timeout responses do not appear successful',async()=>{
  const h=harness({token:''});await assert.rejects(vm.runInContext("request('/folders')",h.context),/首次使用/);assert.equal(h.calls.length,0);
  h.data.token='fixture';h.setFetch(async()=>({ok:false,status:401,json:async()=>({})}));await assert.rejects(vm.runInContext("request('/folders')",h.context),/失效/);
  h.setFetch(async()=>({ok:true,status:200,json:async()=>{throw Error('bad json')}}));await assert.rejects(vm.runInContext("request('/folders')",h.context),/无法识别/);
  h.setFetch((url,opts)=>new Promise((_,reject)=>opts.signal.addEventListener('abort',()=>reject(Error('abort')))));
  await assert.rejects(vm.runInContext("request('/capture',{method:'POST',body:'{}',timeout:5})",h.context),/结果待确认/);
 });
test('Empty explicit tags are respected and recent folders survive parallel captures',async()=>{
  const h=harness();h.background();await Promise.all(['D:/a','D:/b','D:/c'].map(folderPath=>h.message({type:'lap:capture',payload:{sourceUrl:'https://example.com/a.jpg',tags:[],folderPath}})));
  assert.equal(h.data.recentFolders.length,3);for(const call of h.calls)assert.deepEqual(JSON.parse(call.opts.body).tags,[]);
 });
test('Windows invalid folder names are rejected before request',async()=>{
  const h=harness();h.background();for(const name of ['bad/name','CON','NUL.txt','trailing.','  '])assert.equal((await h.message({type:'lap:create-folder',name})).ok,false);assert.equal(h.calls.length,0);
 });
test('Batch survives absent popup; locks concurrent starts; persists mixed outcomes',async()=>{
  const h=harness();let release;let started=false;
  h.setFetch(async(url,opts)=>{if(url.endsWith('/folders'))return {ok:true,status:200,json:async()=>({folders:[]})};const payload=JSON.parse(opts.body);
   if(!started){started=true;await new Promise(r=>release=r);}
   return {ok:!payload.sourceUrl.includes('fail'),status:payload.sourceUrl.includes('fail')?422:200,json:async()=>payload.sourceUrl.includes('fail')?{error:'unsupported'}:{ok:true,duplicate:payload.sourceUrl.includes('duplicate')}};
  });h.background();
  const first=await h.message({type:'lap:start-batch',payloads:['saved','duplicate','fail'].map(x=>({sourceUrl:`https://example.com/${x}.jpg`}))});assert.equal(first.ok,true);await settle();
  const second=await h.message({type:'lap:start-batch',payloads:[{sourceUrl:'https://example.com/extra.jpg'}]});assert.equal(second.ok,false);release();await settle(60);
  const result=await h.message({type:'lap:get-job'});assert.equal(result.job.status,'done');assert.equal(result.job.saved,1);assert.equal(result.job.duplicate,1);assert.equal(result.job.failed,1);assert.equal(result.job.completed,3);
 });
test('Restart never blindly replays an uncertain write',async()=>{
  const h=harness({captureJob:{id:'interrupted',status:'running',items:[{payload:{sourceUrl:'https://example.com/one'},status:'saving'},{payload:{sourceUrl:'https://example.com/two'},status:'pending'}]}});h.background();const result=await h.message({type:'lap:get-job'});assert.equal(result.job.status,'interrupted');assert.equal(result.job.unknown,1);assert.equal(result.job.failed,1);assert.equal(h.calls.length,0);
 });
