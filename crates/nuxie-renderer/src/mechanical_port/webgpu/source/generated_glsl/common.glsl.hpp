#pragma once

#include "common.glsl.exports.h"

namespace rive {
namespace gpu {
namespace glsl {
const char common[] = R"===(#define D3 3.14159265359
#define o8 6.28318530718
#define V6 1.57079632679
#ifndef CB
#define p4 float(.5)
#else
#define p4 float(.0)
#endif
#define M3(l) n8(l,m.jf,m.kf)
#ifdef IF
#define kc(U,f,a) f5(U,f,a)
#define D4 g
#define Y9(q) q
#define X5(q) q
#define Z9(q) uintBitsToFloat(q)
#define g5(q) floatBitsToUint(q)
#else
#define kc(U,f,a) E4(U,f,a)
#define D4 G
#define Y9(q) floatBitsToUint(q)
#define X5(q) uintBitsToFloat(q)
#define Z9(q) q
#define g5(q) q
#endif
#define lf(a,l,p8) q1(a,Y(l)+Y(-1,0))p8,q1(a,Y(l)+Y(0,0))p8,q1(a,Y(l)+Y(0,-1))p8,q1(a,Y(l)+Y(-1,-1))p8
#define h5(q) W6(YC,aa,q,lc,float(lc),.0).x
#define nc(q) W6(YC,aa,q,mc,float(mc),.0).x
#ifdef oc
e c U4(float x){return x;}e c Y5(uint x){return float(x);}e c mf(N x){return float(x);}e c ba(int x){return float(x);}e i Z4(g xyzw){return xyzw;}e E Q7(d xy){return xy;}e i fc(G xyzw){return vec4(xyzw);}e N Z5(c x){return uint(x);}e N W1(uint x){return x;}
#else
e c U4(float x){return(c)x;}e c Y5(uint x){return(c)x;}e c mf(N x){return(c)x;}e c ba(int x){return(c)x;}e i Z4(g xyzw){return(i)xyzw;}e E Q7(d xy){return(E)xy;}e i fc(G xyzw){return(i)xyzw;}e N Z5(c x){return(N)x;}e N W1(uint x){return(N)x;}
#endif
e c G0(c x){return x;}e E B2(E xy){return xy;}e E B2(c x,c y){E T;T.x=x,T.y=y;return T;}e E B2(c x){E T;T.x=x,T.y=x;return T;}e d L6(float x){return d(x,x);}e A Q0(c x,c y,c z){A T;T.x=x,T.y=y,T.z=z;return T;}e A Q0(c x){A T;T.x=x,T.y=x,T.z=x;return T;}e i C0(c x,c y,c z,c w){i T;T.x=x,T.y=y,T.z=z,T.w=w;return T;}e i C0(A xyz,c w){i T;T.xyz=xyz;T.w=w;return T;}e i C0(c x){i T;T.x=x,T.y=x,T.z=x,T.w=x;return T;}e i C0(i x){return x;}e F4 nf(bool b){return F4(b,b);}e X6 Zh(A o,A b,A G1){X6 T;T[0]=o;T[1]=b;T[2]=G1;return T;}e Y6 ai(A o,A b){Y6 T;T[0]=o;T[1]=b;return T;}e i5 bi(i o,i b,i G1,i of){i5 T;T[0]=o;T[1]=b;T[2]=G1;T[3]=of;return T;}e f0 g2(g x){return f0(x.xy,x.zw);}e uint Sb(N x){return x;}e d a6(d o,d b,float t){return(b-o)*t+o;}e c q8(uint pc,uint c6){return pc==0u?.0:unpackHalf2x16((pc+pf)*c6).x;}e float qc(d i2){i2=normalize(i2);float e1=acos(clamp(i2.x,-1.,1.));return i2.y>=.0?e1:-e1;}e i ci(i j){return C0(j.xyz*j.w,j.w);}e A E6(i ca){return ca.xyz*(ca.w!=.0?1./ca.w:.0);}e c h3(E Z6){return min(Z6.x,Z6.y);}e c h3(A rc){return min(h3(rc.xy),rc.z);}e c h3(i sc){E Z6=min(sc.xy,sc.zw);c qf=min(Z6.x,Z6.y);return qf;}e c L5(E a7){return max(a7.x,a7.y);}e c L5(A tc){return max(L5(tc.xy),tc.z);}e c L5(i uc){E a7=max(uc.xy,uc.zw);c rf=max(a7.x,a7.y);return rf;}e float E9(d x){return abs(x.x)+abs(x.y);}e c da(c x,c ea,c fa){
#if defined(JF)||defined(ED)
#ifdef ED
if(ED)
#endif
{if(x<fa)if(x>ea)return x;else return ea;else return fa;}
#endif
return clamp(x,ea,fa);}e c vc(d L0,c C2,c o3){c sf=fract(0.06711056*L0.x+0.00583715*L0.y);c tf=fract(52.9829189*sf);return(tf*C2)+o3;}
#if 0
e c di(d L0,float C2,float o3){int x=int(L0.x);int y=int(L0.y);int wc=(x^y);int b=(y>>1)&1;b|=(wc&2);b|=(y&1)<<2;b|=(wc&1)<<3;float uf=float(b);c vf=U4(uf)/16.0;return(vf*C2)+o3;}e c ei(d L0,float C2,float o3){L0.y*=0.5;L0.x=fract(L0.x*0.5+L0.y);L0.y=fract(L0.y);float P3=(L0.y*0.5+L0.x);return(P3*C2)+o3;}
#endif
#ifdef MB
e c ga(d L0,c C2,c o3){return MB?vc(L0,C2,o3):.0;}e A F2(A j,c c7,d L0,c C2,c o3){return(MB&&c7!=.0)?(vc(L0,C2,o3)+j):j;}e A F2(A j,c c7,c xc){return(MB&&c7!=.0)?(xc+j):j;}
#else
e c ga(d L0,float C2,float o3){return 0.;}e A F2(A j,c c7,d L0,c C2,c o3){return j;}e A F2(A j,c c7,c xc){return j;}
#endif
#ifdef DB
e g n8(d yc,float wf,float zc){return g(yc.x*wf-1.,yc.y*zc-sign(zc),0.,1.);}
#ifndef CB
e g S7(f0 Z3,d G4,d ha){d ia=abs(Z3[0])+abs(Z3[1]);if(ia.x!=.0&&ia.y!=.0){d K=1./ia;d j5=R0(Z3,ha)+G4;const float xf=.5;return g(j5,-j5)*K.xyxy+K.xyxy+xf;}else{return G4.xyxy;}}
#else
e float ja(uint ka){return 1.-float(ka)*(2./32768.);}
#ifdef BB
e void Ac(f0 Z3,d G4,d ha d7){
#ifndef OE
if(any(notEqual(g(Z3),g(.0,.0,.0,.0)))){d j5=R0(Z3,ha)+G4.xy;gl_ClipDistance[0]=j5.x+1.;gl_ClipDistance[1]=j5.y+1.;gl_ClipDistance[2]=1.-j5.x;gl_ClipDistance[3]=1.-j5.y;}else{gl_ClipDistance[0]=gl_ClipDistance[1]=gl_ClipDistance[2]=gl_ClipDistance[3]=G4.x-.5;}
#endif
}
#endif
#endif
#endif
#ifdef GB
#ifdef CC
e c m3(c j){return(j<=0.04045)?j/12.92:pow(abs((j+0.055)/1.055),2.4);}e A m3(A j){return Q0(m3(j.x),m3(j.y),m3(j.z));}e i m3(i j){return C0(m3(j.xyz),j.w);}
#endif
#endif
#if defined(GB)&&defined(CB)&&!defined(Q)
e i Bc(i5 e7,int r8){if(r8==0xf){return(e7[0]+e7[1]+e7[2]+e7[3])*.25;}else{i yf=g(notEqual(r8&d6(1,2,4,8),d6(0,0,0,0)));i T=R0(e7,yf);int v8=(r8&5)+((r8>>1)&5);v8=(v8&3)+(v8>>2);T*=1./float(v8);return T;}}
#endif
)===";
} // namespace glsl
} // namespace gpu
} // namespace rive