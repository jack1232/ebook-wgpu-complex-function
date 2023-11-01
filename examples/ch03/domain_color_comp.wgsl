#import ../../src/complex_func.wgsl as cf;

fn cFunc(z:vec2f, a:f32, selectId:u32) -> vec2f {
    var fz = z;

    if (selectId == 0u) {
        let f1 = z - vec2(a, 0.0);
        let f2 = cf::cMul(z,z) + z + vec2(a, 0.0);
        fz = cf::cDiv(f1, f2); 
    } else if (selectId == 1u) {
        fz = cf::cSqrt(cf::cDiv(cf::cLog(vec2(-z.y - 3.0*a, z.x)), cf::cLog(vec2(-z.y + a, z.x))));
    } else if (selectId == 2u){
        fz = a*cf::cSin(a*z);
    } else if(selectId == 3u){
        fz = (a+0.5)*cf::cTan(cf::cTan((a+0.5)*z));
    } else if(selectId == 4u){
        fz = a*cf::cTan(cf::cSin((a+0.5)*z));
    } else if (selectId == 5u){
        fz = cf::cSqrt(vec2(a + z.x, z.y)) + cf::cSqrt(vec2(a - z.x, -z.y));
    } else if (selectId == 6u){
        fz = cf::cDiv(cf::cTan(cf::cExp2((0.5+a)*z)), z);
    } else if (selectId == 7u){
        fz = cf::cDiv(cf::cSin(cf::cCos(cf::cSin((a+0.5)*z))), cf::cMul(z,z) - a);
    } else if (selectId == 8u){
        fz = (a+0.5)*cf::cInv(cf::cAdd(cf::cPow((a+0.5)*z,5.0), 1.0));
    } else if (selectId == 9u){
        fz = cf::cDiv(cf::cSin((a+0.5)*z), cf::cMul(cf::cCos(cf::cExp2((a+0.5)*z)), cf::cMul(z,z)- vec2((a+0.5)*(a+0.5),0.0)));
    } else if (selectId == 10u) {
        fz = cf::cInv(z + vec2(a, 0.0)) + cf::cInv(z - vec2(a, 0.0));
    } else if(selectId == 11u){
        fz = cf::cInv(z);
    } else if(selectId == 12u){
        fz = z;
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
    let funcId =  ips.funcSelect;
    let colorId = ips.colorSelect;    

    var z = vec2(scale*(f32(id.x) - 0.5*w)/w, -scale*(h/w)*(f32(id.y) - 0.5*h)/h);
    var fz = cFunc(z, a, funcId);
    
    var color:vec4f;
    if (colorId == 0u) { // default
        color = cf::hsv2Rgb(fz); 
    } else { // colormaps
        color = cf::colormap2Rgb(fz, colormap);
    }

    textureStore(tex, vec2(id.xy), color);
}