// Mobile presentation study. Vertex transforms only; original hit regions remain unchanged.
const {chromium}=require('playwright');
const path=require('path');
(async()=>{
 const browser=await chromium.launch({executablePath:'/usr/bin/chromium',args:['--no-sandbox','--enable-unsafe-swiftshader']});
 const page=await browser.newPage({viewport:{width:390,height:844},deviceScaleFactor:2,isMobile:true,hasTouch:true});
 const errors=[];page.on('pageerror',e=>errors.push(e.message));
 await page.addInitScript(()=>{
  window.mobileStudy=false;window.commandCount=0;window.commandLabels=false;window.hudShapes=[];
  const gl=WebGL2RenderingContext.prototype,source=gl.shaderSource,sub=gl.bufferSubData,use=gl.useProgram,draw=gl.drawArraysInstanced;
  let current=null;
  gl.bufferSubData=function(target,offset,data,srcOffset=0,length){
   if(data&&data.buffer){const bytes=data.BYTES_PER_ELEMENT;const n=(length===undefined?data.length-srcOffset:length)*bytes;
    if(n%64===0 && n>512){const f=new Float32Array(data.buffer,data.byteOffset+srcOffset*bytes,n/4);let hud=false;
     for(let i=0;i<f.length;i+=16)if(f[i+12]===3&&f[i+2]===272&&f[i+3]===272)hud=true;
     if(hud){window.hudShapes=[];for(let i=0;i<f.length;i+=16)if(f[i+12]===1)window.hudShapes.push(Array.from(f.slice(i,i+4)).map(v=>v/devicePixelRatio));}
    }
   }return sub.apply(this,arguments);
  };
  gl.shaderSource=function(shader,code){
   if(code.includes('Hud_block_0Vertex')){
    code=code.replace('void main() {','uniform float mobile_study; uniform vec4 mobile_info; uniform vec4 mobile_controls;\nvoid main() {');
    code=code.replace('vec2 pixel = (quad.rect.xy + (q * quad.rect.zw));',`vec2 pixel = (quad.rect.xy + (q * quad.rect.zw));
    vec2 placement_size = quad.rect.zw;
    float dpr = mobile_controls.w;
    if(mobile_study>0.5){
     vec2 screen = _group_0_binding_0_vs.screen/dpr;
     vec4 rect = quad.rect/dpr;
     vec2 center = rect.xy+rect.zw*0.5;
     vec2 px = pixel/dpr;
     vec2 globe = screen-vec2(154.0);
     float count=mobile_controls.x;
     float stepY=58.0+mobile_controls.y*26.0;
     float oldColumns=max(1.0,floor((globe.x-40.0)/58.0));
     float oldRows=ceil(max(1.0,count)/oldColumns);
     vec4 oldBar=vec4(12.0,screen.y-18.0-(oldRows*stepY+2.0),min(count,oldColumns)*58.0+18.0,oldRows*stepY+2.0);
     vec2 newBarSize=vec2(count*58.0+18.0,stepY+2.0);
     vec2 newBar=vec2((screen.x-newBarSize.x)*0.5,screen.y-16.0-newBarSize.y);
     vec2 newInfo=vec2((screen.x-mobile_info.z)*0.5,newBar.y-12.0-mobile_info.w);
     vec2 newGlobe=vec2(screen.x-130.0,count>0.0?newInfo.y-120.0:screen.y-112.0);
     bool handled=false;
     if(center.y<110.0){
      if(center.x<150.0)px+=vec2(4.0,2.0);
      else px+=vec2(0.0,6.0);
      handled=true;
     }
     if(!handled && center.x>globe.x-50.0 && center.y>globe.y-80.0){
      for(int i=0;i<3;i++){
       float angle=3.14159265*(1.16+float(i)*0.17);
       vec2 oldCenter=globe+vec2(68.0)+vec2(cos(angle),sin(angle))*90.0;
       if(distance(center,oldCenter)<21.0&&rect.z<40.0&&rect.w<40.0){
        vec2 newCenter=newGlobe+vec2(4.0+float(i)*44.0,-24.0);
        px+=newCenter-oldCenter;handled=true;
       }
      }
      if(!handled&&rect.x>=globe.x-1.0&&rect.y>=globe.y-1.0){px=(px-globe)*(96.0/136.0)+newGlobe;handled=true;}
     }
     if(!handled&&count>0.0){
      if(center.x>=mobile_info.x&&center.x<=mobile_info.x+mobile_info.z&&center.y>=mobile_info.y&&center.y<=mobile_info.y+mobile_info.w){px+=newInfo-mobile_info.xy;}
      else if(center.x>=oldBar.x&&center.x<=oldBar.x+oldBar.z&&center.y>=oldBar.y&&center.y<=oldBar.y+oldBar.w){
       if(abs(rect.x-oldBar.x)<0.1&&abs(rect.y-oldBar.y)<0.1&&quad.params.x==1.0){px=newBar+q*newBarSize;placement_size=newBarSize*dpr;}
       else{
        float col=clamp(floor((center.x-oldBar.x-9.0)/58.0),0.0,oldColumns-1.0);
        float row=clamp(floor((center.y-oldBar.y-1.0)/stepY),0.0,oldRows-1.0);
        float index=row*oldColumns+col;
        px+=newBar-oldBar.xy+vec2((index-col)*58.0,-row*stepY);
       }
      }
     }
     pixel=px*dpr;
    }`);
    code=code.replace('out_.size = quad.rect.zw;', 'out_.size = placement_size;');
   }return source.call(this,shader,code);
  };
  gl.useProgram=function(p){current=p;return use.call(this,p)};
  gl.drawArraysInstanced=function(...args){
   if(current){const uniform=this.getUniformLocation(current,'mobile_study');if(uniform!==null){
    const info=window.hudShapes.find(r=>r[0]===12&&r[1]>innerHeight/2&&r[2]>=200&&r[3]<=90)||[0,0,0,0];
    this.uniform1f(uniform,window.mobileStudy?1:0);
    this.uniform4fv(this.getUniformLocation(current,'mobile_info'),info);
    this.uniform4f(this.getUniformLocation(current,'mobile_controls'),window.commandCount,window.commandLabels?1:0,0,devicePixelRatio);
   }}return draw.apply(this,args);
  };
 });
 await page.goto('http://127.0.0.1:8001/web/index.html?local');await page.waitForFunction(()=>!document.querySelector('#loading'),{timeout:120000});
 async function capture(state,count,labels){await page.evaluate(([c,l])=>{window.commandCount=c;window.commandLabels=l},[count,labels]);
  for(const proposed of [false,true]){await page.evaluate(v=>window.mobileStudy=v,proposed);await page.waitForTimeout(500);await page.screenshot({path:path.join(__dirname,`mobile-${state}-${proposed?'proposed':'current'}.png`)});}
  console.log(state,await page.evaluate(()=>window.hudShapes));await page.evaluate(()=>window.mobileStudy=false);
 }
 await capture('overview',0,false);
 await page.touchscreen.tap(195,330);await capture('town',4,false);
 await page.touchscreen.tap(72,375);await page.touchscreen.tap(50,796);await capture('categories',4,true);
 await page.touchscreen.tap(50,686);await capture('submenu',5,true);
 console.log(JSON.stringify({errors}));await browser.close();
})();
