#define W7 -2.
#define ne -1.5
#define oe .25
#define H9 1e3
#define pe (H9*H9)
#ifdef VERTEX
q4 i5(r3,Ch,UB);
#ifdef ENABLE_FEATHER
K6(r3,L7,AD);
#endif
r4 Y4 j5(ce,gi,KB);j6(bd,lg,XC);k6(cd,mg,JB);j5(de,hi,DD);Z4
#endif
#if defined(ENABLE_FEATHER)||defined(FEATHER_ATLAS_BLIT)
a4(L7,db)
#endif
#ifdef FRAGMENT
V3 q3(r3,ee,ZC);
#if defined(ENABLE_FEATHER)||defined(FEATHER_ATLAS_BLIT)
K6(r3,L7,AD);
#endif
#ifdef FEATHER_ATLAS_BLIT
N5(r3,fe,JD);
#endif
q3(B5,v4,TB);
#if defined(RENDER_MODE_DEPTH_STENCIL)&&defined(ENABLE_ADVANCED_BLEND)&&!defined(FIXED_FUNCTION_COLOR_OUTPUT)&&!defined(RENDER_TO_UAV)
O5(MD);
#endif
W3
#ifdef RENDER_TO_UAV
E1 L6(x2,RC);F1
#endif
M6(ee,Q8)
#ifdef FEATHER_ATLAS_BLIT
a4(fe,Sa)
#endif
C5 w4(U4) D5
#endif
#ifdef FRAGMENT
e bool o6(f T){return T.y>=.0;}e bool o6(D T){return T.y>=.0;}
#endif
#if defined(FRAGMENT)&&defined(ENABLE_FEATHER)
e bool gd(f T){return T.x<ne;}e bool hd(f T){return T.y<ne;}
#endif
#ifdef VERTEX
f qe(float Mb,c I9,float I0){c N6=(1.-I9*abs(I0))*.5;float A4,P5;if(abs(Mb-A7)<1./H9){A4=.0;P5=.0;}else{float Nb=tan(Mb);A4=sign(A7-Mb)/max(abs(Nb),1./pe);P5=A4>=.0?N6.y-(1.-N6.x)*Nb:N6.y+N6.x*Nb;}f T;T.x=max(N6.x,.0)+oe;T.y=-N6.y+W7;T.z=A4;T.w=P5;return T;}
#endif
#ifdef ENABLE_FEATHER
e d T8(f T c4){d A4=T.z;d P5=max(T.w,.0);d O6=A4>=.0?G5(P5):.0;if(abs(A4)<H9){d x=abs(T.x)-oe;d y=-T.y+W7;d p3=(y-P5)*0.5984134206;i t=P5+p3*H0(0.20888568955,0.62665706865,1.04442844776,1.46219982687);i u=t*-A4+(y*A4+x);i ii=H0(G5(u[0]),G5(u[1]),G5(u[2]),G5(u[3]));i re=t*5.09593080173+-2.54796540086;i ji=exp2(-re*re);O6+=dot(ii,ji)*p3;}return O6*sign(T.x);}e d T4(f T c4){float O6=1.;float ki=(1.-W7)+T.x;O6-=G5(ki);float li=1.-T.y;O6-=G5(li);return O6;}
#endif
#ifdef VERTEX
e g0 y4(int se){return g0(se&((1<<Qd)-1),se>>Qd);}e float E9(uint z){return float(z)*(h9/(65536.*65536.));}e float ke(uint z){return float(z&0xffffu)*(1./65535.);}
#endif
#if defined(VERTEX)&&defined(DRAW_PATH)
e float te(X N0,c mi){c B2=C0(N0,mi);return(abs(B2.x)+abs(B2.y))*(1./dot(B2,B2));}e bool ra(f X7,f Ob,int r,l1(uint) w3,l1(c) ni
#ifndef RENDER_MODE_DEPTH_STENCIL
,l1(f) d2
#else
,l1(P) Y7
#endif
P6){int J9=int(X7.x);float I0=X7.y;float Pb=X7.z;int ue=floatBitsToInt(X7.w)>>2;int Z7=floatBitsToInt(X7.w)&3;int K5=min(J9,ue-1);int Y3=r*ue+K5;O a2=r1(UB,y4(Y3));uint a0=a2.w;uint D6=max(a0&xb,1u);O L5=p0(DD,D6-1u);c Q7=uintBitsToFloat(L5.xy);w3=L5.z&0xffffu;uint C9=L5.w;X N0=p1(uintBitsToFloat(p0(KB,w3*4u)));O L3=p0(KB,w3*4u+1u);c y2=uintBitsToFloat(L3.xy);float C2=uintBitsToFloat(L3.z);float Y2=uintBitsToFloat(L3.w);uint F6=a0&X2;if(F6!=0u){J9=int(Ob.x);I0=Ob.y;Pb=Ob.z;}if(J9!=K5){int D9=Y3+J9-K5;O S7=r1(UB,y4(D9));if((S7.w&(X2|0xffffu))!=(a0&(X2|0xffffu))){bool T7=C2==.0||Q7.x!=.0;if(T7){Y3=int(C9);a2=r1(UB,y4(Y3));}}else{Y3=D9;a2=S7;}a0=(a2.w&~X2)|F6;}bool Qb=false;float g1;
#ifdef ENABLE_FEATHER
float a8;float I1;if((a0&J3)==p9&&Z7==r9){uint ve=a2.z;float B4=float(ve&0xffffu);float D2=float(ve>>16);g0 K9=g0(-B4-1.,D2-B4+1.);if((a0&X2)!=0u) K9=-K9;O we=r1(UB,y4(Y3+K9.x));O Rb=r1(UB,y4(Y3+K9.y));if((Rb.w&(X2|0xffffu))!=(we.w&(X2|0xffffu))){Rb=r1(UB,y4(int(C9)));}a8=E9(we.z);float xe=E9(Rb.z);I1=xe-a8;if(abs(I1)>p4) I1-=h9*sign(I1);float Sb=D2+1.-float(Ud);float ye=clamp(round(abs(I1)/p4*Sb),1.,Sb-1.);float c8=Sb-ye;if(B4<=c8){I1=-(p4*sign(I1)-I1);D2=c8;if(B4==c8) I0=-I0;}else if(B4==c8+1.){B4=.0;D2=.0;I0=.0;}else{B4-=c8+2.;D2=ye;}if(B4==D2){g1=xe;}else{g1=a8+I1*(B4/D2);}}else
#endif
{g1=E9(a2.z);}c P1=c(sin(g1),-cos(g1));c G6=uintBitsToFloat(a2.xy);c L9=c(0,0);if(Y2!=.0){Y2=max(Y2,(sb/3.)/length(C0(N0,P1)));}if(C2!=.0){I0*=sign(determinant(N0));if((a0&y6)!=0u) I0=min(I0,.0);if((a0&wb)!=0u) I0=max(I0,.0);float k5=Y2!=.0?Y2:te(N0,P1)*O4;d ze=1.;if(k5>C2&&Y2==.0){ze=w5(C2)/w5(k5);C2=k5;}c M3=P1*(C2+k5);
#ifndef RENDER_MODE_DEPTH_STENCIL
float x=I0*(C2+k5);d2.xy=(1./(k5*2.))*(c(x,-x)+C2)+.5;d2.zw=o7(.0);
#endif
uint f5=a0&J3;if(f5>K7){bool H6=(a0&ub)!=0u;bool Hb=(a0&y6)!=0u;float c2=ke(a2.z);float z4=sqrt(max(1.-c2*c2,.0));if(H6==Hb) z4=-z4;X Ib=X(c2,z4,-z4,c2);c g5=C0(Ib,P1);float Tb=te(N0,g5);float Ub;if((f5==tb)||(f5==qh&&c2>=.25)){float oi=(a0&q9)!=0u?1.:.25;Ub=C2*(1./max(c2,oi));}else{Ub=C2*c2+Tb*.5;}float Vb=Ub+Tb*O4;if((a0&vb)!=0u){float Ae=C2+k5;float pi=k5*.125;if(Ae<=Vb*c2+pi){float qi=Ae*(1./c2);M3=g5*qi;}else{c Wb=g5*Vb;c ri=c(dot(M3,M3),dot(Wb,Wb));M3=C0(ri,inverse(X(M3,Wb)));}}c si=abs(I0)*M3;float Be=(Vb-dot(si,g5))/(Tb*(O4*2.));
#ifndef RENDER_MODE_DEPTH_STENCIL
if((a0&y6)!=0u) d2.y=Be;else d2.x=Be;
#endif
}
#ifndef RENDER_MODE_DEPTH_STENCIL
d2.xy*=ze;d2.y=max(d2.y,1e-4);if(Y2!=.0){d2.x=W7-d2.x;}
#endif
L9=C0(N0,I0*M3);if(Z7!=r9) Qb=true;}else{
#ifndef RENDER_MODE_DEPTH_STENCIL
d2=f(Pb,-1.,.0,.0);
#ifdef ENABLE_FEATHER
if(Y2!=.0){d2.y=W7;d2.z=pe;d2.w=Pb;if((a0&J3)==p9&&Z7==r9){if(I1<.0){a8+=I1;I1=-I1;}float C4=g1-a8;C4=mod(C4+A7,h9)-A7;C4=clamp(C4,.0,I1);if(C4>I1*.5){C4=I1-C4;}c I9=c(sin(C4),cos(C4));
#if 0
float e2=1.+.33*log2(A7/(p4-min(I1,p4-p4/16.)));f ti=qe(I1,I9,.5*(e2/3.));float ui=T8(ti n1);float vi=Dd(ui);float wi=(.5-vi)*(sb*2.);float xi=e2/max(wi,e2);I0*=xi;
#endif
d2=qe(I1,I9,I0);}L9=C0(N0,(I0*Y2)*P1);}else
#endif
{L9=sign(C0(I0*P1,inverse(N0)))*O4;}if(bool(a0&X2)!=bool(a0&sh)){d2*=f(-1.,+1.,+1.,+1.);}
#endif
if(Z7==Zd) G6=Q7;if((a0&Xd)!=0u&&Z7!=Yd){Qb=true;}}ni=C0(N0,G6)+L9+y2;
#ifdef RENDER_MODE_DEPTH_STENCIL
O Z3=p0(KB,w3*4u+2u);Y7=U1(Z3.x);
#else
d2.xy=mix(d2.xy,c(1.,-1.),Ng(j.yi!=0u));
#endif
return!Qb;}
#endif
#if defined(VERTEX)&&defined(DRAW_INTERIOR_TRIANGLES)
e c Yc(M Q6,l1(uint) w3
#ifdef RENDER_MODE_DEPTH_STENCIL
,l1(P) Y7
#else
,l1(d) zi
#endif
P6){w3=floatBitsToUint(Q6.z)&0xffffu;
#ifdef RENDER_MODE_DEPTH_STENCIL
O Z3=p0(KB,w3*4u+2u);Y7=U1(Z3.x);
#else
zi=fb(floatBitsToInt(Q6.z)>>16);
#endif
c R6=Q6.xy;X N0=p1(uintBitsToFloat(p0(KB,w3*4u)));O L3=p0(KB,w3*4u+1u);c y2=uintBitsToFloat(L3.xy);R6=C0(N0,R6)+y2;return R6;}
#endif
#if defined(VERTEX)&&defined(FEATHER_ATLAS_BLIT)
e c Xc(M Q6,l1(uint) w3,
#ifdef RENDER_MODE_DEPTH_STENCIL
l1(P) Y7,
#endif
l1(c) Ai P6){w3=floatBitsToUint(Q6.z)&0xffffu;O Z3=p0(KB,w3*4u+2u);
#ifdef RENDER_MODE_DEPTH_STENCIL
Y7=U1(Z3.x);
#endif
c R6=Q6.xy;M d8=uintBitsToFloat(Z3.yzw);Ai=(R6*d8.x+d8.yz)*j.Bi;return R6;}
#endif
e d M9(d p2,d Q1,d x3){return(Q1-p2)/max(1.-p2*x3,qa);}
#if defined(RENDER_MODE_CLOCKWISE_ATOMIC)||defined(PLS_IMPL_STORAGE_BUFFER)
e uint N9(R0 v3,uint Ci){uint Xb=(v3.y>>B6)*(Ci<<B6)+((v3.x>>B6)<<(B6<<1));Xb+=((v3.x&0x1cu)<<B6)+((v3.y&0x1cu)<<2);Xb+=((v3.y&0x3u)<<2)+(v3.x&0x3u);return Xb;}
#endif
#ifdef RENDER_MODE_CLOCKWISE_ATOMIC
#ifdef FIXED_FUNCTION_COLOR_OUTPUT
#define F5 H2
#define x4(Q5) N1=Q5;E3
#else
#define F5 X1
#define x4(Q5) x0(n0,Q5);o2;
#endif
e d Yb(uint Di){return fb(int((Di&Cb)-I5))*Ab;}e uint e8(d n){return uint(n*Jh+.5);}
#endif
