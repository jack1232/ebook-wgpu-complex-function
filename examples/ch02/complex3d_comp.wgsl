#import ../../src/complex_func.wgsl as cf;

const pi:f32 = 3.14159265359;

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
    } 
   
    return fz;
}

struct DataRange {
    xRange: vec2f,
    yRange: vec2f,
    zRange: vec2f,
    cRange: vec2f,
}

fn getDataRange(funcSelection:u32) -> DataRange{
	var dr:DataRange;

	if (funcSelection == 0u) { 
		dr.xRange = vec2(-3.0, 2.0);
		dr.zRange = vec2(-2.0, 2.0);
        dr.cRange = vec2(-pi, pi);
        dr.yRange = vec2(0.0, 45.0);
	} else if (funcSelection == 1u) {
		dr.xRange = vec2(-6.0, 6.0);
		dr.zRange = vec2(-6.0, 6.0);
        dr.cRange = vec2(-pi/2.0, pi/2.0);
        dr.yRange = vec2(0.0, 7.0);       
	} else if (funcSelection == 2u) {
		dr.xRange = vec2(-6.0, 6.0);
		dr.zRange = vec2(-6.0, 6.0);
        dr.cRange = vec2(-pi, pi);
        dr.yRange = vec2(0.0, 203.0);
	} else if (funcSelection == 3u) {
		dr.xRange = vec2(-10.0, 10.0);
		dr.zRange = vec2(-1.0, 1.0);
        dr.cRange = vec2(-pi, pi);        
        dr.yRange = vec2(0.0, 30.0);
	} else if (funcSelection == 4u) {
		dr.xRange = vec2(-8.0, 8.0);
		dr.zRange = vec2(-2.0, 2.0);
        dr.cRange = vec2(-pi, pi);
        dr.yRange = vec2(0.0, 27.0);
	} else if (funcSelection == 5u) {
		dr.xRange = vec2(-2.0, 2.0);
		dr.zRange = vec2(-2.0, 2.0);
        dr.cRange = vec2(-pi/2.0, pi/2.0);
        dr.yRange = vec2(1.4, 2.9);
	} else if (funcSelection == 6u) {
		dr.xRange = vec2(-1.0, 2.0);
		dr.zRange = vec2(-1.0, 1.0);
        dr.cRange = vec2(-pi, pi);
        dr.yRange = vec2(0.0, 120.0);
	} else if (funcSelection == 7u) {
		dr.xRange = vec2(-2.0, 2.0);
		dr.zRange = vec2(-1.0, 1.0);
        dr.cRange = vec2(-pi, pi);
        dr.yRange = vec2(0.0, 18.5);
	} else if (funcSelection == 8u) {
		dr.xRange = vec2(-1.0, 1.0);
		dr.zRange = vec2(-1.0, 1.0);
        dr.cRange = vec2(-pi, pi);
        dr.yRange = vec2(0.0, 26.0);
	} else if (funcSelection == 9u) {
		dr.xRange = vec2(-4.0, 6.0);
		dr.zRange = vec2(-2.0, 2.0);
        dr.cRange = vec2(-pi, pi);
        dr.yRange = vec2(0.0, 8.0);
	} else if (funcSelection == 10u) {
		dr.xRange = vec2(-2.0, 2.0);
		dr.zRange = vec2(-2.0, 2.0);
        dr.cRange = vec2(-pi, pi);
        dr.yRange = vec2(0.0, 46.0);
	}
	return dr;
}

struct VertexData{
    position: vec4f,
    color: vec4f,
}

struct VertexDataArray{
    vertexDataArray: array<VertexData>,
}

struct ComplexParams {
    resolution: f32,
    funcSelection: f32,
    animationTime: f32,
    scale: f32,
    aspectRatio: f32,
}

@group(0) @binding(0) var<storage, read_write> vda : VertexDataArray;
@group(0) @binding(1) var<uniform> colormap: array<vec4f, 11>;
@group(0) @binding(2) var<uniform> cp: ComplexParams;

fn colorLerp(tmin:f32, tmax:f32, t:f32) -> vec4f{
    var t1 = t;
    if (t1 < tmin) {t1 = tmin;}
    if (t1 > tmax) {t1 = tmax;}
    var tn = (t1-tmin)/(tmax-tmin);

    var idx = u32(floor(10.0*tn));
    var color = vec4(0.0,0.0,0.0, 1.0);
   
    if(f32(idx) == 10.0*tn) {
        color = colormap[idx];
    } else {
        var tn1 = (tn - 0.1*f32(idx))*10.0;
        var a = colormap[idx];
        var b = colormap[idx+1u];
        color.x = a.x + (b.x - a.x)*tn1;
        color.y = a.y + (b.y - a.y)*tn1;
        color.z = a.z + (b.z - a.z)*tn1;
    }

    return color;
}

var<private> xmin:f32;
var<private> xmax:f32;
var<private> ymin:f32; 
var<private> ymax:f32;
var<private> zmin:f32; 
var<private> zmax:f32;
var<private> cmin:f32; 
var<private> cmax:f32;
var<private> aspect:f32;

fn getUv(i:u32, j:u32) -> vec2f {
    var dr = getDataRange(u32(cp.funcSelection));
	xmin = dr.xRange[0];
	xmax = dr.xRange[1];
	ymin = dr.yRange[0];
	ymax = dr.yRange[1];
	zmin = dr.zRange[0];
	zmax = dr.zRange[1];	
    cmin = dr.cRange[0];
    cmax = dr.cRange[1];

    var dx = (xmax - xmin)/(cp.resolution - 1.0);
    var dz = (zmax - zmin)/(cp.resolution - 1.0);
    var x = xmin + f32(i) * dx;
    var z = zmin + f32(j) * dz;
    return vec2(x, z);
}

fn normalizePoint(pos1: vec3f) -> vec3f {
    var pos = pos1;
    pos.x = (2.0 * (pos.x - xmin)/(xmax - xmin) - 1.0) * cp.scale;
    pos.y = (2.0 * (pos.y - ymin)/(ymax - ymin) - 1.0) * cp.scale;
    pos.z = (2.0 * (pos.z - zmin)/(zmax - zmin) - 1.0) * cp.scale;    
    pos.y = pos.y * cp.aspectRatio;
    return pos;
}

@compute @workgroup_size(8, 8, 1)
fn cs_main(@builtin(global_invocation_id) id : vec3u) {
    let i = id.x;
    let j = id.y;   
    let z = getUv(i, j);

    let fz = cFunc(z, cp.animationTime, u32(cp.funcSelection));        
    var pt:vec3f = vec3(z.x, length(fz), z.y);

    if(pt.y < ymin) {
        pt.y = ymin;
    }
    if(pt.y > ymax) {
        pt.y = ymax;
    }

    var ps = normalizePoint(pt);
    let color = colorLerp(cmin, cmax, cf::cArg(fz));
   
    var idx = i + j * u32(cp.resolution);
    vda.vertexDataArray[idx].position = vec4(ps, 1.0);
    vda.vertexDataArray[idx].color = color;
}