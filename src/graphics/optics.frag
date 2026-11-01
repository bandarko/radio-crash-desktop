#version 330 core
in vec2 uv;
out vec4 color;
uniform sampler2D source_image;
uniform sampler2D bloom_image;
uniform int stage;
uniform vec2 resolution;
uniform float brightness;
uniform float signal_light;
uniform float volume;

vec3 linearize(vec3 c) { return pow(max(c, vec3(0.0)), vec3(2.2)); }
vec3 encode(vec3 c) { return pow(clamp(c, 0.0, 1.0), vec3(1.0/2.2)); }
float box(vec2 p, vec4 r) {
    return step(r.x,p.x)*step(r.y,p.y)*step(p.x,r.x+r.z)*step(p.y,r.y+r.w);
}
float bell(float x, float center, float width) {
    float d=(x-center)/width; return exp(-d*d);
}
void main() {
    vec4 raw=texture(source_image,uv);
    if(stage==1) {
        // Extract real colored phosphor from the rendered PCM meters and text.
        float hi=max(raw.r,max(raw.g,raw.b));
        float lo=min(raw.r,min(raw.g,raw.b));
        float emission=smoothstep(0.13*brightness,0.48*brightness,hi)
            *smoothstep(0.13,0.52,(hi-lo)/max(hi,0.001));
        color=vec4(linearize(raw.rgb)*emission,1.0); return;
    }
    if(stage==2 || stage==3) {
        vec2 d=(stage==2 ? vec2(1,0) : vec2(0,1))/resolution;
        vec3 c=texture(source_image,uv).rgb*0.227027;
        c+=(texture(source_image,uv+d*1.384615).rgb+texture(source_image,uv-d*1.384615).rgb)*0.316216;
        c+=(texture(source_image,uv+d*3.230769).rgb+texture(source_image,uv-d*3.230769).rgb)*0.070270;
        color=vec4(c,1); return;
    }
    if(stage==4) {
        vec3 light=texture(bloom_image,uv).rgb;
        vec2 pixel=1.0/resolution;
        vec3 ghost=linearize(texture(source_image,clamp(uv+pixel*vec2(1.2,-1.6),0.0,1.0)).rgb);
        // Linear-light optical scattering, not an audio-driven invented animation.
        vec3 c=linearize(raw.rgb)+light*0.085+ghost*0.008;
        vec2 edge=min(uv,1.0-uv)*resolution;
        float seal=smoothstep(0.0,3.5,min(edge.x,edge.y));
        c*=mix(0.70,1.0,seal);
        float top=exp(-edge.y*0.13)*step(uv.y,0.5);
        float highlight=bell(uv.x+uv.y*0.22,0.19,0.26);
        c+=vec3(0.0012,0.0018,0.0020)*highlight*0.8;
        c+=vec3(0.001,0.0019,0.0022)*top;
        color=vec4(encode(c),raw.a); return;
    }
    vec2 p=vec2(uv.x,1.0-uv.y)*vec2(1200,440);
    // Keep the approved branding, panel lettering, HD badge and glass untouched.
    float face=box(p,vec4(26,29,1148,354));
    float protected_area=max(box(p,vec4(53,88,1094,164)),box(p,vec4(77,43,1060,32)));
    protected_area=max(protected_area,box(p,vec4(450,300,180,58)));
    protected_area=max(protected_area,box(p,vec4(77,292,80,71)));
    protected_area=max(protected_area,box(p,vec4(215,292,143,75)));
    protected_area=max(protected_area,box(p,vec4(692,294,93,72)));
    protected_area=max(protected_area,box(p,vec4(1053,296,75,68)));
    protected_area=max(protected_area,step(352.0,p.y));
    float knob=1.0-smoothstep(50.0,53.0,length(p-vec2(911,319)));
    float metal=face*(1.0-protected_area)*(1.0-knob);
    // Suppress frequencies above the device-pixel Nyquist limit when resized.
    // Derivatives are evaluated before the per-fragment material mask branch.
    float grain_phase=p.y*3.2+sin(p.y*0.9)*1.7+sin(p.x*0.012)*0.7;
    float grain=sin(grain_phase)*(1.0-smoothstep(1.4,3.14,fwidth(grain_phase)));
    vec2 k=p-vec2(911,319);
    float ring_phase=length(k)*4.0;
    float rings=sin(ring_phase)*0.012*(1.0-smoothstep(1.4,3.14,fwidth(ring_phase)));
    if(metal<0.001 && knob<0.001) { color=raw; return; }
    vec3 c=linearize(raw.rgb);
    // Directional grain is static in physical panel coordinates.
    float sheen=bell(p.x+p.y*0.28,120.0,190.0)*0.027
        +bell(p.x-p.y*0.12,1080.0,230.0)*0.012;
    c*=1.0+metal*(grain*0.025+sheen);
    c+=metal*vec3(0.0021,0.0020,0.0018)*sheen*10.0;
    // Faint phosphor spill reaches only the aluminium lip next to the glass.
    float spill=bell(p.y,253.0,8.0)*box(p,vec4(69,245,1062,28));
    c+=vec3(0.0,0.00075,0.0010)*spill*signal_light*brightness*metal;
    float angle=atan(k.y,k.x);
    float radial=pow(abs(cos(angle+0.75)),18.0)*0.014;
    c*=1.0+knob*(radial+rings);
    // A restrained machined edge, with the real knob position affecting the glint.
    float rim=bell(length(k),47.0,1.3);
    c+=knob*rim*vec3(0.010,0.010,0.009)*pow(max(0.0,cos(angle+1.0+volume*0.16)),8.0);
    color=vec4(encode(c),raw.a);
}
