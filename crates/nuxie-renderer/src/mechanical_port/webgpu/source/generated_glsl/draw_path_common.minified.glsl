#define X7 -2.
#define Be -1.5
#define Ce .25
#define L9 1e3
#define De (L9*L9)
#ifdef VERTEX
A4 r5(c3,Sh,WB);
#ifdef ENABLE_FEATHER
M6(c3,M7,ZC);
#endif
B4 h5 o4(Na,nd,JB);n6(od,Cg,KB);o4(pe,xi,CD);i5
#endif
#if defined(ENABLE_FEATHER)||defined(FEATHER_ATLAS_BLIT)
f4(M7,ib)
#endif
#ifdef FRAGMENT
Y3 Y2(c3,qe,YC);
#if defined(ENABLE_FEATHER)||defined(FEATHER_ATLAS_BLIT)
M6(c3,M7,ZC);
#endif
#ifdef FEATHER_ATLAS_BLIT
S5(c3,re,ID);
#endif
Y2(I5,C4,SB);
#if defined(RENDER_MODE_DEPTH_STENCIL)&&defined(ENABLE_ADVANCED_BLEND)&&!defined(FIXED_FUNCTION_COLOR_OUTPUT)&&!defined(RENDER_TO_UAV)
J4(LD);
#endif
Z3
#ifdef RENDER_TO_UAV
H1 K4(B2,LC);I1
#endif
N6(qe,V8)
#ifdef FEATHER_ATLAS_BLIT
f4(re,Xa)
#endif
J5 D4(x4) K5
#endif
#ifdef FRAGMENT
f bool r6(e T){return T.y>=.0;}f bool r6(F T){return T.y>=.0;}
#endif
#ifdef FRAGMENT
f d Ae(i Ub,uint G3){d Ee=dot(Ub.xyz,d1(.30,.59,.11));if(G3==Ph) return Ub.w;if(G3==Qh) return 1.-Ub.w;if(G3==Rh) return Ee;return 1.-Ee;}
#endif
#if defined(FRAGMENT)&&defined(ENABLE_FEATHER)
f bool sd(e T){return T.x<Be;}f bool td(e T){return T.y<Be;}
#endif
#ifdef VERTEX
e Fe(float Vb,c M9,float K0){c O6=(1.-M9*abs(K0))*.5;float L4,T5;if(abs(Vb-B7)<1./L9){L4=.0;T5=.0;}else{float Wb=tan(Vb);L4=sign(B7-Vb)/max(abs(Wb),1./De);T5=L4>=.0?O6.y-(1.-O6.x)*Wb:O6.y+O6.x*Wb;}e T;T.x=max(O6.x,.0)+Ce;T.y=-O6.y+X7;T.z=L4;T.w=T5;return T;}
#endif
#ifdef ENABLE_FEATHER
f d Y8(e T g4){d L4=T.z;d T5=max(T.w,.0);d P6=L4>=.0?M5(T5):.0;if(abs(L4)<L9){d x=abs(T.x)-Ce;d y=-T.y+X7;d x3=(y-T5)*0.5984134206;i t=T5+x3*z0(0.20888568955,0.62665706865,1.04442844776,1.46219982687);i u=t*-L4+(y*L4+x);i yi=z0(M5(u[0]),M5(u[1]),M5(u[2]),M5(u[3]));i Ge=t*5.09593080173+-2.54796540086;i zi=exp2(-Ge*Ge);P6+=dot(yi,zi)*x3;}return P6*sign(T.x);}f d e5(e T g4){float P6=1.;float Ai=(1.-X7)+T.x;P6-=M5(Ai);float Bi=1.-T.y;P6-=M5(Bi);return P6;}
#endif
#ifdef VERTEX
f f0 F4(int He){return f0(He&((1<<ce)-1),He>>ce);}f float I9(uint z){return float(z)*(A7/(65536.*65536.));}f float xe(uint z){return float(z&0xffffu)*(1./65535.);}
#endif
#if defined(VERTEX)&&defined(DRAW_PATH)
f float Ie(X Q0,c Ci){c F2=F0(Q0,Ci);return(abs(F2.x)+abs(F2.y))*(1./dot(F2,F2));}f bool va(e Y7,e Xb,int r,n1(uint) A3,n1(c) Di
#ifndef RENDER_MODE_DEPTH_STENCIL
,n1(e) h2
#else
,n1(O) Z7
#endif
Q6){int N9=int(Y7.x);float K0=Y7.y;float Yb=Y7.z;int Je=floatBitsToInt(Y7.w)>>2;int R6=floatBitsToInt(Y7.w)&3;int Q5=min(N9,Je-1);int d4=r*Je+Q5;R e2=q1(WB,F4(d4));uint c0=e2.w;uint H6=max(c0&Cb,1u);R R5=o0(CD,H6-1u);c Q7=uintBitsToFloat(R5.xy);A3=R5.z&0xffffu;uint G9=R5.w;X Q0=v1(uintBitsToFloat(o0(JB,A3*4u)));R w2=o0(JB,A3*4u+1u);c C2=uintBitsToFloat(w2.xy);float f2=.0;float P2=.0;float Ke=0.0;if((w2.z&Oh)!=0u){R S6=o0(JB,A3*4u+2u);f2=uintBitsToFloat(S6.x);P2=uintBitsToFloat(S6.y);Ke=uintBitsToFloat(S6.z);}uint J6=c0&a3;if(J6!=0u){N9=int(Xb.x);K0=Xb.y;Yb=Xb.z;}if(N9!=Q5){int H9=d4+N9-Q5;R S7=q1(WB,F4(H9));if((S7.w&(a3|0xffffu))!=(c0&(a3|0xffffu))){bool T7=f2==.0||Q7.x!=.0;if(T7){d4=int(G9);e2=q1(WB,F4(d4));}}else{d4=H9;e2=S7;}c0=(e2.w&~a3)|J6;}bool Zb=false;float h1;
#ifdef ENABLE_FEATHER
float a8;float N1;if((c0&y3)==w9&&R6==y9){uint Le=e2.z;float M4=float(Le&0xffffu);float G2=float(Le>>16);f0 O9=f0(-M4-1.,G2-M4+1.);if((c0&a3)!=0u) O9=-O9;R Me=q1(WB,F4(d4+O9.x));R ac=q1(WB,F4(d4+O9.y));if((ac.w&(a3|0xffffu))!=(Me.w&(a3|0xffffu))){ac=q1(WB,F4(int(G9)));}a8=I9(Me.z);float Ne=I9(ac.z);N1=Ne-a8;if(abs(N1)>z4) N1-=A7*sign(N1);float bc=G2+1.-float(ge);float Oe=clamp(round(abs(N1)/z4*bc),1.,bc-1.);float c8=bc-Oe;if(M4<=c8){N1=-(z4*sign(N1)-N1);G2=c8;if(M4==c8) K0=-K0;}else if(M4==c8+1.){M4=.0;G2=.0;K0=.0;}else{M4-=c8+2.;G2=Oe;}if(M4==G2){h1=Ne;}else{h1=a8+N1*(M4/G2);}}else
#endif
{h1=I9(e2.z);}c M1=c(sin(h1),-cos(h1));c K6=uintBitsToFloat(e2.xy);c d8=c(0,0);if(P2!=.0){P2=max(P2,(xb/3.)/length(F0(Q0,M1)));}if(f2!=.0){K0*=sign(determinant(Q0));if((c0&C6)!=0u) K0=min(K0,.0);if((c0&Bb)!=0u) K0=max(K0,.0);float v5=P2!=.0?P2:Ie(Q0,M1)*Y4;d Pe=1.;if(v5>f2&&P2==.0){Pe=Z4(f2)/Z4(v5);f2=v5;}c P3=M1*(f2+v5);
#ifndef RENDER_MODE_DEPTH_STENCIL
float x=K0*(f2+v5);h2.xy=(1./(v5*2.))*(c(x,-x)+f2)+.5;h2.zw=o7(.0);
#endif
uint o5=c0&y3;if(o5>K7){bool L6=(c0&zb)!=0u;bool Nb=(c0&C6)!=0u;float g2=xe(e2.z);float G4=sqrt(max(1.-g2*g2,.0));if(L6==Nb) G4=-G4;X Ob=X(g2,G4,-G4,g2);c p5=F0(Ob,M1);float cc=Ie(Q0,p5);float dc;if((o5==yb)||(o5==Hh&&g2>=.25)){float Ei=(c0&x9)!=0u?1.:.25;dc=f2*(1./max(g2,Ei));}else{dc=f2*g2+cc*.5;}float ec=dc+cc*Y4;if((c0&Ab)!=0u){float Qe=f2+v5;float Fi=v5*.125;if(Qe<=ec*g2+Fi){float Gi=Qe*(1./g2);P3=p5*Gi;}else{c fc=p5*ec;c Hi=c(dot(P3,P3),dot(fc,fc));P3=F0(Hi,inverse(X(P3,fc)));}}c Ii=abs(K0)*P3;float Re=(ec-dot(Ii,p5))/(cc*(Y4*2.));
#ifndef RENDER_MODE_DEPTH_STENCIL
if((c0&C6)!=0u) h2.y=Re;else h2.x=Re;
#endif
}
#ifndef RENDER_MODE_DEPTH_STENCIL
h2.xy*=Pe;h2.y=max(h2.y,1e-4);if(P2!=.0){h2.x=X7-h2.x;}
#endif
d8=F0(Q0,K0*P3);if(R6!=y9) Zb=true;}else{
#ifndef RENDER_MODE_DEPTH_STENCIL
h2=e(Yb,-1.,.0,.0);
#ifdef ENABLE_FEATHER
if(P2!=.0){h2.y=X7;h2.z=De;h2.w=Yb;if((c0&y3)==w9&&R6==y9){if(N1<.0){a8+=N1;N1=-N1;}float N4=h1-a8;N4=mod(N4+B7,A7)-B7;N4=clamp(N4,.0,N1);if(N4>N1*.5){N4=N1-N4;}c M9=c(sin(N4),cos(N4));
#if 0
float i2=1.+.33*log2(B7/(z4-min(N1,z4-z4/16.)));e Ji=Fe(N1,M9,.5*(i2/3.));float Ki=Y8(Ji p1);float Li=Pd(Ki);float Mi=(.5-Li)*(xb*2.);float Ni=i2/max(Mi,i2);K0*=Ni;
#endif
h2=Fe(N1,M9,K0);}d8=F0(Q0,(K0*P2)*M1);}else
#endif
{d8=sign(F0(K0*M1,inverse(Q0)))*Y4;if(R6!=Db){d8+=M1*Ke;}}if(bool(c0&a3)!=bool(c0&Kh)){h2*=e(-1.,+1.,+1.,+1.);}
#endif
if(R6==Db) K6=Q7;if((c0&je)!=0u&&R6!=ke){Zb=true;}}Di=F0(Q0,K6)+d8+C2;
#ifdef RENDER_MODE_DEPTH_STENCIL
Z7=Z1(w2.z>>16u);
#else
h2.xy=mix(h2.xy,c(1.,-1.),eh(j.Oi!=0u));
#endif
return!Zb;}
#endif
#if defined(VERTEX)&&defined(DRAW_INTERIOR_TRIANGLES)
f c kd(M T6,n1(uint) A3
#ifdef RENDER_MODE_DEPTH_STENCIL
,n1(O) Z7
#else
,n1(d) Pi
#endif
Q6){A3=floatBitsToUint(T6.z)&0xffffu;R w2=o0(JB,A3*4u+1u);
#ifdef RENDER_MODE_DEPTH_STENCIL
Z7=Z1(w2.z>>16u);
#else
Pi=kb(floatBitsToInt(T6.z)>>16);
#endif
c U6=T6.xy;X Q0=v1(uintBitsToFloat(o0(JB,A3*4u)));c C2=uintBitsToFloat(w2.xy);U6=F0(Q0,U6)+C2;return U6;}
#endif
#if defined(VERTEX)&&defined(FEATHER_ATLAS_BLIT)
f c jd(M T6,n1(uint) A3,
#ifdef RENDER_MODE_DEPTH_STENCIL
n1(O) Z7,
#endif
n1(c) Qi Q6){A3=floatBitsToUint(T6.z)&0xffffu;
#ifdef RENDER_MODE_DEPTH_STENCIL
Z7=Z1(o0(JB,A3*4u+1u).z>>16u);
#endif
c U6=T6.xy;M e8=uintBitsToFloat(o0(JB,A3*4u+3u).xyz);Qi=(U6*e8.x+e8.yz)*j.Ri;return U6;}
#endif
f d P9(d x2,d U1,d B3){return(U1-x2)/max(1.-x2*B3,ua);}
#if defined(RENDER_MODE_CLOCKWISE_ATOMIC)||defined(PLS_IMPL_STORAGE_BUFFER)
f uint Q9(R0 z3,uint Si){uint gc=(z3.y>>E6)*(Si<<E6)+((z3.x>>E6)<<(E6<<1));gc+=((z3.x&0x1cu)<<E6)+((z3.y&0x1cu)<<2);gc+=((z3.y&0x3u)<<2)+(z3.x&0x3u);return gc;}
#endif
#ifdef RENDER_MODE_CLOCKWISE_ATOMIC
#ifdef FIXED_FUNCTION_COLOR_OUTPUT
#define L5 K2
#define E4(U5) S1=U5;J3
#else
#define L5 D1
#define E4(U5) x0(p0,U5);K1;
#endif
f d hc(uint Ti){return kb(int((Ti&Ib)-O5))*Gb;}f uint f8(d n){return uint(n*Yh+.5);}
#endif
