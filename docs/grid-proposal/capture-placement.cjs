// Presentation-only browser instrumentation. Never changes the client or sends hosted commands.
const {chromium}=require('playwright');
const path=require('path');
(async()=>{
 const b=await chromium.launch({executablePath:'/usr/bin/chromium',args:['--no-sandbox','--enable-unsafe-swiftshader']});
 const p=await b.newPage({viewport:{width:1440,height:900}});
 await p.addInitScript(()=>{
  window.placementStudy=false;
  const g=WebGL2RenderingContext.prototype,source=g.shaderSource,use=g.useProgram,draw=g.drawArraysInstanced;let program=null;
  g.shaderSource=function(s,code){
   if(code.includes('Hud_block_0Vertex')){
    code=code.replace('void main() {','uniform float placement_study;\nvoid main() {');
    code=code.replace('vec2 pixel = (quad.rect.xy + (q * quad.rect.zw));',`vec2 pixel = (quad.rect.xy + (q * quad.rect.zw));
    vec2 screen = _group_0_binding_0_vs.screen;
    vec2 delta = vec2(0.0);
    vec2 center = quad.rect.xy + quad.rect.zw * 0.5;
    vec2 globe = screen - vec2(154.0);
    if(center.y < 110.0) {
     if(center.x < 150.0) delta = vec2(12.0,10.0);
     else if(center.x > screen.x*0.5) delta = vec2(-8.0,14.0);
    } else if(center.x > globe.x-50.0 && center.y > globe.y-80.0) {
     bool speed = false;
     for(int i=0;i<3;i++) {
      float angle = 3.14159265*(1.16+float(i)*0.17);
      vec2 old_center = globe+vec2(68.0)+vec2(cos(angle),sin(angle))*90.0;
      if(distance(center,old_center)<21.0 && quad.rect.z<40.0 && quad.rect.w<40.0) {
       vec2 new_center = vec2(globe.x+18.0+float(i)*44.0,globe.y-30.0);
       delta = new_center-old_center;speed=true;
      }
     }
     if(!speed && quad.rect.x >= globe.x-1.0 && quad.rect.y >= globe.y-1.0)delta=vec2(-6.0);
    } else if(center.y > screen.y-300.0 && center.x > screen.x*0.25 && center.x < screen.x*0.75) {
     delta = vec2(0.0,-6.0);
    }
    pixel += delta*placement_study;`);
   }return source.call(this,s,code);
  };
  g.useProgram=function(p){program=p;return use.call(this,p)};
  g.drawArraysInstanced=function(...a){if(program){const u=this.getUniformLocation(program,'placement_study');if(u!==null)this.uniform1f(u,window.placementStudy?1:0)}return draw.apply(this,a)};
 });
 const errors=[];p.on('pageerror',e=>errors.push(e.message));
 await p.goto('http://127.0.0.1:8001/web/index.html?local');
 await p.waitForFunction(()=>!document.querySelector('#loading'),{timeout:120000});
 const out=__dirname;
 async function capture(name){await p.mouse.move(1000,400);for(const proposed of [false,true]){await p.evaluate(v=>window.placementStudy=v,proposed);await p.waitForTimeout(600);await p.screenshot({path:path.join(out,`${name}-${proposed?'proposed':'current'}.png`)})}await p.evaluate(()=>window.placementStudy=false)}
 await p.mouse.click(720,340);await capture('placement-town');
 await p.mouse.click(590,405);await p.mouse.click(720,850);await capture('placement-categories');
 // Category row: four starter controls, including Close, centred at the bottom.
 await p.mouse.click(627,825);await capture('placement-submenu');
 console.log(JSON.stringify({errors}));await b.close();
})();
