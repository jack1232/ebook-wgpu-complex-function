const pi:f32 = 3.14159265359;
const e:f32 = 2.71828182845;

fn cAdd(a:vec2f, s:f32) -> vec2f{
    return vec2(a.x+s, a.y);
}
fn cMul(a:vec2f, b:vec2f) ->vec2f{
    return vec2(a.x*b.x-a.y*b.y, a.x*b.y + a.y*b.x);
}
fn cDiv(a:vec2f, b:vec2f) ->vec2f{
    let d = dot(b,b);
    return vec2(dot(a,b)/d, (a.y*b.x-a.x*b.y)/d);
}
fn cSqrt(z:vec2f) -> vec2f{
    let m = length(z);
    let s = sqrt(0.5*vec2(m+z.x, m-z.x));
    return s*vec2(1.0, sign(z.y));
}
fn cConj(z:vec2f) -> vec2f{
    return vec2(z.x, -z.y);
}
fn cPow (z:vec2f, n:f32) -> vec2f{
    let r = length(z);
    let a = atan2(z.y, z.x);
    return pow(r, n) * vec2(cos(a*n), sin(a*n)); 
}
fn cInv(z:vec2f) -> vec2f{
    return vec2(z.x/dot(z,z), -z.y/dot(z,z));
}
fn cArg(z: vec2f) -> f32 {
    return atan2(z.y, z.x);
}
fn cLog(z:vec2f) -> vec2f{
    return vec2(log(sqrt(dot(z,z))), atan2(z.y, z.x));
}
fn cSin(z:vec2f) ->vec2f{
    let a = pow(e, z.y);
    let b = pow(e, -z.y);
    return vec2(sin(z.x)*(a+b)*0.5, cos(z.x)*(a-b)*0.5);
}
fn cCos(z:vec2f) ->vec2f{
    let a = pow(e, z.y);
    let b = pow(e, -z.y);
    return vec2(cos(z.x)*(a+b)*0.5, -sin(z.x)*(a-b)*0.5);
}
fn cTan(z:vec2f) ->vec2f{
    let a = pow(e, z.y);
    let b = pow(e, -z.y);
    let cx = cos(z.x);
    let ab = (a - b)*0.5;
    return vec2(sin(z.x)*cx, ab*(a+b)*0.5)/(cx*cx+ab*ab);
}
fn cExp2(z:vec2f) -> vec2f{
    return vec2(z.x*z.x - z.y*z.y, 2.*z.x*z.y);
}

fn cExp(z:vec2f) -> vec2f{
    return vec2(exp(z.x)*cos(z.y), exp(z.x)*sin(z.y));
}

fn cAsinh(z:vec2f) -> vec2f{
    let a = z + cSqrt(cMul(z,z) + vec2<f32>(1.0,0.0));
    return cLog(a);
}

// hsv to rgb color conversion
fn hsv2Rgb(z:vec2f) -> vec4f{
    let len = length(z);
    let h = cArg(z)/2.0/pi;
    var fx = 2.0*(fract(z.x) - 0.5);
    var fy = 2.0*(fract(z.y) - 0.5);
    fx = fx*fx;
    fy = fy*fy;
    var g = 1.0 -(1.0 - fx)*(1.0 - fy);
    g = pow(abs(g), 10.0);
    var c = 2.0*(fract(log2(len)) - 0.5);
    c = 0.7*pow(abs(c), 10.0);  
    var v = 1.0 - 0.5*g;
    let f = abs((h*6.0 + vec3(0.0,4.0,2.0))%6.0 - 3.0) - 1.0;
    var rgb = clamp(f, vec3(0.0), vec3(1.0));
    rgb = rgb*rgb*(3.0 - 2.0*rgb);
    rgb = (1.0-c)*v*mix(vec3(1.0), rgb, 1.0);  
    return vec4(rgb + c*vec3(1.0), 1.0);
}

fn colormap2Rgb(z:vec2f, colormap:array<vec4f,11>) -> vec4f {
    var c = colormap;
    let len = length(z);
    var h = atan2(z.y, z.x);
    if(h < 0.0) { h = h + 2.0*pi; }
    if(h >= 2.0*pi) { h = h - 2.0*pi; }
    var s = 0.0;
    var v = vec3f(0.0);

    for(var i:i32 = 0; i < 11; i = i+1){
        if(h >= 0.2*pi*f32(i) && h < 0.2*pi*(f32(i) + 1.0)){
            s = (h - f32(i)*0.2*pi)/(0.2*pi);
            v = s*c[i+1].rgb + (1.0-s)*c[i].rgb;
        }
    }
    let b = fract(log2(len));
    return vec4(v[0]*b, v[1]*b, v[2]*b, 1.0);
}