#define T7 -2.
#define le -1.5
#define me .25
#define E9 1e3
#define ne (E9*E9)
#ifdef VERTEX
q4 i5(q3,zh,UB);
#ifdef ENABLE_FEATHER
I6(q3,I7,YC);
#endif
r4 Y4 j5(ae,di,KB);j6(ad,jg,VC);k6(bd,kg,JB);j5(be,ei,BD);Z4
#endif
#if defined(ENABLE_FEATHER)||defined(FEATHER_ATLAS_BLIT)
a4(I7,ab)
#endif
#ifdef FRAGMENT
V3 p3(q3,ce,XC);
#if defined(ENABLE_FEATHER)||defined(FEATHER_ATLAS_BLIT)
I6(q3,I7,YC);
#endif
#ifdef FEATHER_ATLAS_BLIT
M5(q3,de,HD);
#endif
p3(A5,v4,TB);
#if defined(RENDER_MODE_DEPTH_STENCIL)&&defined(ENABLE_ADVANCED_BLEND)&&!defined(FIXED_FUNCTION_COLOR_OUTPUT)
N5(KD);
#endif
W3 J6(ce,N8)
#ifdef FEATHER_ATLAS_BLIT
a4(de,Pa)
#endif
B5 w4(U4) C5
#endif
#ifdef FRAGMENT
e bool o6(f T){return T.y>=.0;}e bool o6(D T){return T.y>=.0;}
#endif
#if defined(FRAGMENT)&&defined(ENABLE_FEATHER)
e bool fd(f T){return T.x<le;}e bool gd(f T){return T.y<le;}
#endif
#ifdef VERTEX
f oe(float Ib,c F9,float I0){c K6=(1.-F9*abs(I0))*.5;float A4,O5;if(abs(Ib-x7)<1./E9){A4=.0;O5=.0;}else{float Jb=tan(Ib);A4=sign(x7-Ib)/max(abs(Jb),1./ne);O5=A4>=.0?K6.y-(1.-K6.x)*Jb:K6.y+K6.x*Jb;}f T;T.x=max(K6.x,.0)+me;T.y=-K6.y+T7;T.z=A4;T.w=O5;return T;}
#endif
#ifdef ENABLE_FEATHER
e d Q8(f T c4){d A4=T.z;d O5=max(T.w,.0);d L6=A4>=.0?F5(O5):.0;if(abs(A4)<E9){d x=abs(T.x)-me;d y=-T.y+T7;d o3=(y-O5)*0.5984134206;i t=O5+o3*H0(0.20888568955,0.62665706865,1.04442844776,1.46219982687);i u=t*-A4+(y*A4+x);i fi=H0(F5(u[0]),F5(u[1]),F5(u[2]),F5(u[3]));i pe=t*5.09593080173+-2.54796540086;i gi=exp2(-pe*pe);L6+=dot(fi,gi)*o3;}return L6*sign(T.x);}e d T4(f T c4){float L6=1.;float hi=(1.-T7)+T.x;L6-=F5(hi);float ii=1.-T.y;L6-=F5(ii);return L6;}
#endif
#ifdef VERTEX
e g0 y4(int qe){return g0(qe&((1<<Od)-1),qe>>Od);}e float B9(uint z){return float(z)*(e9/(65536.*65536.));}e float ie(uint z){return float(z&0xffffu)*(1./65535.);}
#endif
#if defined(VERTEX)&&defined(DRAW_PATH)
e float re(X N0,c ji){c A2=B0(N0,ji);return(abs(A2.x)+abs(A2.y))*(1./dot(A2,A2));}e bool pa(f U7,f Kb,int r,k1(uint) v3,k1(c) ki
#ifndef RENDER_MODE_DEPTH_STENCIL
,k1(f) d2
#else
,k1(P) V7
#endif
M6){int G9=int(U7.x);float I0=U7.y;float Lb=U7.z;int se=floatBitsToInt(U7.w)>>2;int W7=floatBitsToInt(U7.w)&3;int J5=min(G9,se-1);int Y3=r*se+J5;O a2=q1(UB,y4(Y3));uint a0=a2.w;uint D6=max(a0&tb,1u);O K5=p0(BD,D6-1u);c N7=uintBitsToFloat(K5.xy);v3=K5.z&0xffffu;uint z9=K5.w;X N0=o1(uintBitsToFloat(p0(KB,v3*4u)));O L3=p0(KB,v3*4u+1u);c x2=uintBitsToFloat(L3.xy);float B2=uintBitsToFloat(L3.z);float Y2=uintBitsToFloat(L3.w);uint F6=a0&X2;if(F6!=0u){G9=int(Kb.x);I0=Kb.y;Lb=Kb.z;}if(G9!=J5){int A9=Y3+G9-J5;O P7=q1(UB,y4(A9));if((P7.w&(X2|0xffffu))!=(a0&(X2|0xffffu))){bool Q7=B2==.0||N7.x!=.0;if(Q7){Y3=int(z9);a2=q1(UB,y4(Y3));}}else{Y3=A9;a2=P7;}a0=(a2.w&~X2)|F6;}bool Mb=false;float f1;
#ifdef ENABLE_FEATHER
float X7;float F1;if((a0&J3)==m9&&W7==o9){uint te=a2.z;float B4=float(te&0xffffu);float C2=float(te>>16);g0 H9=g0(-B4-1.,C2-B4+1.);if((a0&X2)!=0u) H9=-H9;O ue=q1(UB,y4(Y3+H9.x));O Nb=q1(UB,y4(Y3+H9.y));if((Nb.w&(X2|0xffffu))!=(ue.w&(X2|0xffffu))){Nb=q1(UB,y4(int(z9)));}X7=B9(ue.z);float ve=B9(Nb.z);F1=ve-X7;if(abs(F1)>p4) F1-=e9*sign(F1);float Ob=C2+1.-float(Sd);float we=clamp(round(abs(F1)/p4*Ob),1.,Ob-1.);float Y7=Ob-we;if(B4<=Y7){F1=-(p4*sign(F1)-F1);C2=Y7;if(B4==Y7) I0=-I0;}else if(B4==Y7+1.){B4=.0;C2=.0;I0=.0;}else{B4-=Y7+2.;C2=we;}if(B4==C2){f1=ve;}else{f1=X7+F1*(B4/C2);}}else
#endif
{f1=B9(a2.z);}c N1=c(sin(f1),-cos(f1));c G6=uintBitsToFloat(a2.xy);c I9=c(0,0);if(Y2!=.0){Y2=max(Y2,(ob/3.)/length(B0(N0,N1)));}if(B2!=.0){I0*=sign(determinant(N0));if((a0&y6)!=0u) I0=min(I0,.0);if((a0&sb)!=0u) I0=max(I0,.0);float k5=Y2!=.0?Y2:re(N0,N1)*O4;d xe=1.;if(k5>B2&&Y2==.0){xe=v5(B2)/v5(k5);B2=k5;}c M3=N1*(B2+k5);
#ifndef RENDER_MODE_DEPTH_STENCIL
float x=I0*(B2+k5);d2.xy=(1./(k5*2.))*(c(x,-x)+B2)+.5;d2.zw=l7(.0);
#endif
uint f5=a0&J3;if(f5>H7){bool H6=(a0&qb)!=0u;bool Db=(a0&y6)!=0u;float c2=ie(a2.z);float z4=sqrt(max(1.-c2*c2,.0));if(H6==Db) z4=-z4;X Eb=X(c2,z4,-z4,c2);c g5=B0(Eb,N1);float Pb=re(N0,g5);float Qb;if((f5==pb)||(f5==oh&&c2>=.25)){float li=(a0&n9)!=0u?1.:.25;Qb=B2*(1./max(c2,li));}else{Qb=B2*c2+Pb*.5;}float Rb=Qb+Pb*O4;if((a0&rb)!=0u){float ye=B2+k5;float mi=k5*.125;if(ye<=Rb*c2+mi){float ni=ye*(1./c2);M3=g5*ni;}else{c Sb=g5*Rb;c oi=c(dot(M3,M3),dot(Sb,Sb));M3=B0(oi,inverse(X(M3,Sb)));}}c pi=abs(I0)*M3;float ze=(Rb-dot(pi,g5))/(Pb*(O4*2.));
#ifndef RENDER_MODE_DEPTH_STENCIL
if((a0&y6)!=0u) d2.y=ze;else d2.x=ze;
#endif
}
#ifndef RENDER_MODE_DEPTH_STENCIL
d2.xy*=xe;d2.y=max(d2.y,1e-4);if(Y2!=.0){d2.x=T7-d2.x;}
#endif
I9=B0(N0,I0*M3);if(W7!=o9) Mb=true;}else{
#ifndef RENDER_MODE_DEPTH_STENCIL
d2=f(Lb,-1.,.0,.0);
#ifdef ENABLE_FEATHER
if(Y2!=.0){d2.y=T7;d2.z=ne;d2.w=Lb;if((a0&J3)==m9&&W7==o9){if(F1<.0){X7+=F1;F1=-F1;}float C4=f1-X7;C4=mod(C4+x7,e9)-x7;C4=clamp(C4,.0,F1);if(C4>F1*.5){C4=F1-C4;}c F9=c(sin(C4),cos(C4));
#if 0
float e2=1.+.33*log2(x7/(p4-min(F1,p4-p4/16.)));f qi=oe(F1,F9,.5*(e2/3.));float ri=Q8(qi m1);float si=Bd(ri);float ti=(.5-si)*(ob*2.);float ui=e2/max(ti,e2);I0*=ui;
#endif
d2=oe(F1,F9,I0);}I9=B0(N0,(I0*Y2)*N1);}else
#endif
{I9=sign(B0(I0*N1,inverse(N0)))*O4;}if(bool(a0&X2)!=bool(a0&qh)){d2*=f(-1.,+1.,+1.,+1.);}
#endif
if(W7==Xd) G6=N7;if((a0&Vd)!=0u&&W7!=Wd){Mb=true;}}ki=B0(N0,G6)+I9+x2;
#ifdef RENDER_MODE_DEPTH_STENCIL
O Z3=p0(KB,v3*4u+2u);V7=S1(Z3.x);
#else
d2.xy=mix(d2.xy,c(1.,-1.),Lg(j.vi!=0u));
#endif
return!Mb;}
#endif
#if defined(VERTEX)&&defined(DRAW_INTERIOR_TRIANGLES)
e c Xc(M N6,k1(uint) v3
#ifdef RENDER_MODE_DEPTH_STENCIL
,k1(P) V7
#else
,k1(d) wi
#endif
M6){v3=floatBitsToUint(N6.z)&0xffffu;
#ifdef RENDER_MODE_DEPTH_STENCIL
O Z3=p0(KB,v3*4u+2u);V7=S1(Z3.x);
#else
wi=cb(floatBitsToInt(N6.z)>>16);
#endif
c O6=N6.xy;X N0=o1(uintBitsToFloat(p0(KB,v3*4u)));O L3=p0(KB,v3*4u+1u);c x2=uintBitsToFloat(L3.xy);O6=B0(N0,O6)+x2;return O6;}
#endif
#if defined(VERTEX)&&defined(FEATHER_ATLAS_BLIT)
e c Wc(M N6,k1(uint) v3,
#ifdef RENDER_MODE_DEPTH_STENCIL
k1(P) V7,
#endif
k1(c) xi M6){v3=floatBitsToUint(N6.z)&0xffffu;O Z3=p0(KB,v3*4u+2u);
#ifdef RENDER_MODE_DEPTH_STENCIL
V7=S1(Z3.x);
#endif
c O6=N6.xy;M Z7=uintBitsToFloat(Z3.yzw);xi=(O6*Z7.x+Z7.yz)*j.yi;return O6;}
#endif
e d J9(d p2,d O1,d w3){return(O1-p2)/max(1.-p2*w3,oa);}
#if defined(RENDER_MODE_CLOCKWISE_ATOMIC)||defined(PLS_IMPL_STORAGE_BUFFER)
e uint K9(R0 r3,uint zi){uint Tb=(r3.y>>B6)*(zi<<B6)+((r3.x>>B6)<<(B6<<1));Tb+=((r3.x&0x1cu)<<B6)+((r3.y&0x1cu)<<2);Tb+=((r3.y&0x3u)<<2)+(r3.x&0x3u);return Tb;}
#endif
#ifdef RENDER_MODE_CLOCKWISE_ATOMIC
#ifdef FIXED_FUNCTION_COLOR_OUTPUT
#define E5 G2
#define x4(P5) L1=P5;E3
#else
#define E5 X1
#define x4(P5) y0(n0,P5);o2;
#endif
e d Ub(uint Ai){return cb(int((Ai&yb)-H5))*wb;}e uint a8(d n){return uint(n*Gh+.5);}
#endif
