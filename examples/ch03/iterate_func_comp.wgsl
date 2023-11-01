#import ../../src/complex_func.wgsl as cf;

// define iterated complex functions
fn cFunc(z:vec2f, a:f32, selectId:u32) -> vec2f {
    var fz = z;
   
    if (selectId == 0u) {
        fz = cf::cMul(vec2(a, a), cf::cLog(cf::cMul(z,z)));
    } else if (selectId == 1u){
        fz = cf::cDiv(cf::cLog(cf::cMul(z,z)-vec2(0.0, a)), cf::cExp(cf::cMul(z,z))-vec2(a, 0.0));
    } else if (selectId == 2u){
        fz = cf::cDiv(cf::cCos(z), cf::cSin(cf::cMul(z,z) - vec2(0.5*a, 0.0)));
    } else if (selectId == 3u){
        let f1 = cf::cInv(cf::cPow(z, 4.0) + vec2(0.0, 0.1*a));
        fz = cf::cAsinh(cf::cSin(f1));
    } else if (selectId == 4u){
        let f1 = cf::cInv(cf::cPow(z, 6.0) + vec2(0.0, 0.5*a));
        fz = cf::cLog(cf::cSin(f1));
    } else if (selectId == 5u){
        let f1 = cf::cMul(vec2<f32>(0.0,1.0), cf::cCos(z));
        let f2 = cf::cSin(cf::cMul(z,z) - vec2(a, 0.0));
        fz = cf::cDiv(f1, f2);
    } else if (selectId == 6u){
        let f1 = cf::cCos(cf::cMul(vec2<f32>(0.0,1.0), z));
        let f2 = cf::cSin(cf::cMul(z,z) - vec2(a, 0.0));
        fz = cf::cDiv(f1, f2);
    } else if (selectId == 7u){
        let f1 = cf::cTan(z);
        let f2 = cf::cSin(cf::cPow(z,8.0) - vec2(0.5*a, 0.0));
        fz = cf::cDiv(f1, f2);
    } else if (selectId == 8u){
        fz = cf::cInv(z) + cf::cDiv(cf::cMul(z,z), cf::cSin(cf::cPow(z,2.0) - vec2(a, 0.0)));
    } else if (selectId == 9u){
        fz = cf::cConj(z) + cf::cDiv(cf::cMul(z,z), cf::cSin(cf::cPow(z,2.0) - vec2(2.0*a, 0.0)));
    } else if (selectId == 10u){
        fz = cf::cSqrt(cf::cMul(vec2(0.0,1.0), z)) + cf::cDiv(cf::cMul(z,z), cf::cSin(cf::cPow(z,2.0) - vec2(2.0*a, 0.0)));
    } else {
        fz = cf::cMul(vec2(a), cf::cLog(cf::cMul(z,z)));
    }
    return fz;
}

@group(0) @binding(0) var<uniform> colormap: array<vec4f, 11>;

struct IntParams {
    funcSelect: u32,   
    colorSelect: u32,             
}
@group(0) @binding(1) var<uniform> ips: IntParams;

struct FloatParams {
    animateParam: f32,
    width: f32,
    height: f32, 
    scale: f32,          
}
@group(0) @binding(2) var<uniform> fps: FloatParams;

@group(1) @binding(0) var tex: texture_storage_2d<rgba8unorm, write>;

@compute @workgroup_size(8, 8, 1)
fn cs_main(@builtin(global_invocation_id) id: vec3u) {
    let a = fps.animateParam;    
    let w = fps.width;
    let h = fps.height;
    let scale = fps.scale;
    var funcId =  ips.funcSelect;
    let colorId = ips.colorSelect;    

    var z = vec2(scale*(f32(id.x) - 0.5*w)/w, -scale*(h/w)*(f32(id.y) - 0.5*h)/h);
    var iters = array<u32,11>(4u,3u,4u,2u,2u,5u,4u,10u,6u,9u,4u);
    if(funcId >= 10u) {
        funcId = 0u;
    }

    var i = 0u;
    loop {
        if(i >= iters[funcId]) { break; }      
        z = cFunc(z, a, funcId);
        i = i + 1u;
    }

    var color:vec4f;
    if (colorId == 0u) {    // default
        color = cf::hsv2Rgb(z); 
    } else {                // colormaps
        color = cf::colormap2Rgb(z, colormap);
    }

    textureStore(tex, vec2(id.xy), color);
}