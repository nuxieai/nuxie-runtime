#define x7 -2.
#define Cd -1.5
#define Dd .25
#define P8 1e3
#define Ed (P8*P8)
#ifdef VERTEX
j4 V4(l3,Fg,TB);
#ifdef ENABLE_FEATHER
p6(l3,v7,ZC);
#endif
k4 P4 W4(xd,dh,LB);X5(oc,Bf,XC);Y5(pc,Cf,JB);W4(yd,eh,AD);Q4
#endif
#if defined(ENABLE_FEATHER)||defined(FEATHER_ATLAS_BLIT)
o4(v7,wa)
#endif
#ifdef FRAGMENT
O3 i3(l3,zd,FD);
#if defined(ENABLE_FEATHER)||defined(FEATHER_ATLAS_BLIT)
p6(l3,v7,ZC);
#endif
#ifdef FEATHER_ATLAS_BLIT
B5(l3,Ad,GD);
#endif
i3(r5,l4,IC);
#if defined(RENDER_MODE_DEPTH_STENCIL)&&defined(ENABLE_ADVANCED_BLEND)&&!defined(FIXED_FUNCTION_COLOR_OUTPUT)
C5(YD);
#endif
P3 o4(zd,ha)
#ifdef FEATHER_ATLAS_BLIT
o4(Ad,ma)
#endif
v5 m4(f6) w5
#endif
#ifdef FRAGMENT
f bool e6(e U){return U.y>=.0;}f bool e6(C U){return U.y>=.0;}
#endif
#if defined(FRAGMENT)&&defined(ENABLE_FEATHER)
f bool yc(e U){return U.x<Cd;}f bool zc(e U){return U.y<Cd;}
#endif
#ifdef VERTEX
e Fd(float Ua,c Q8,float K1){c q6=(1.-Q8*abs(K1))*.5;float p4,D5;if(abs(Ua-i7)<1./P8){p4=.0;D5=.0;}else{float Va=tan(Ua);p4=sign(i7-Ua)/max(abs(Va),1./Ed);D5=p4>=.0?q6.y-(1.-q6.x)*Va:q6.y+q6.x*Va;}e U;U.x=max(q6.x,.0)+Dd;U.y=-q6.y+x7;U.z=p4;U.w=D5;return U;}
#endif
#ifdef ENABLE_FEATHER
f d p8(e U S3){d p4=U.z;d D5=max(U.w,.0);d r6=p4>=.0?y5(D5):.0;if(abs(p4)<P8){d x=abs(U.x)-Dd;d y=-U.y+x7;d h3=(y-D5)*0.5984134206;i t=D5+h3*G0(0.20888568955,0.62665706865,1.04442844776,1.46219982687);i u=t*-p4+(y*p4+x);i fh=G0(y5(u[0]),y5(u[1]),y5(u[2]),y5(u[3]));i Gd=t*5.09593080173+-2.54796540086;i gh=exp2(-Gd*Gd);r6+=dot(fh,gh)*h3;}return r6*sign(U.x);}f d M4(e U S3){float r6=1.;float hh=(1.-x7)+U.x;r6-=y5(hh);float ih=1.-U.y;r6-=y5(ih);return r6;}
#endif
#ifdef VERTEX
f e0 q4(int Hd){return e0(Hd&((1<<jd)-1),Hd>>jd);}f float Wa(uint z){return float(z)*(F8/(65536.*65536.));}f float jh(uint z){return float(z&0xffffu)*(1./65535.);}
#endif
#if defined(VERTEX)&&defined(DRAW_PATH)
f float Id(Y S0,c kh){c v2=K0(S0,kh);return(abs(v2.x)+abs(v2.y))*(1./dot(v2,v2));}f bool K9(e y7,e Xa,int r,i1(uint) m3,i1(c) lh
#ifndef RENDER_MODE_DEPTH_STENCIL
,i1(e) W1
#else
,i1(R) z7
#endif
v6){int T3=int(y7.x);float K1=y7.y;float Ya=y7.z;int Jd=floatBitsToInt(y7.w)>>2;int A7=floatBitsToInt(y7.w)&3;int E5=min(T3,Jd-1);int U3=r*Jd+E5;N D2=p1(TB,q4(U3));uint i0=D2.w;uint w6=max(i0&Ma,1u);N F5=p0(AD,w6-1u);c R8=uintBitsToFloat(F5.xy);m3=F5.z&0xffffu;uint S8=F5.w;Y S0=n1(uintBitsToFloat(p0(LB,m3*4u)));N V3=p0(LB,m3*4u+1u);c m2=uintBitsToFloat(V3.xy);float S2=uintBitsToFloat(V3.z);float T2=uintBitsToFloat(V3.w);uint B7=i0&R2;if(B7!=0u){T3=int(Xa.x);K1=Xa.y;Ya=Xa.z;}if(T3!=E5){int T8=U3+T3-E5;N C7=p1(TB,q4(T8));if((C7.w&(R2|0xffffu))!=(i0&(R2|0xffffu))){bool mh=S2==.0||R8.x!=.0;if(mh){U3=int(S8);D2=p1(TB,q4(U3));}}else{U3=T8;D2=C7;}i0=(D2.w&~R2)|B7;}bool Za=false;float w1;
#ifdef ENABLE_FEATHER
float D7;float C1;if((i0&R3)==L8&&A7==O8){uint Kd=D2.z;float r4=float(Kd&0xffffu);float w2=float(Kd>>16);e0 U8=e0(-r4-1.,w2-r4+1.);if((i0&R2)!=0u) U8=-U8;N Ld=p1(TB,q4(U3+U8.x));N ab=p1(TB,q4(U3+U8.y));if((ab.w&(R2|0xffffu))!=(Ld.w&(R2|0xffffu))){ab=p1(TB,q4(int(S8)));}D7=Wa(Ld.z);float Md=Wa(ab.z);C1=Md-D7;if(abs(C1)>i4) C1-=F8*sign(C1);float bb=w2+1.-float(md);float Nd=clamp(round(abs(C1)/i4*bb),1.,bb-1.);float E7=bb-Nd;if(r4<=E7){C1=-(i4*sign(C1)-C1);w2=E7;if(r4==E7) K1=-K1;}else if(r4==E7+1.){r4=.0;w2=.0;K1=.0;}else{r4-=E7+2.;w2=Nd;}if(r4==w2){w1=Md;}else{w1=D7+C1*(r4/w2);}}else
#endif
{w1=Wa(D2.z);}c P2=c(sin(w1),-cos(w1));c V8=uintBitsToFloat(D2.xy);c W8=c(0,0);if(T2!=.0){T2=max(T2,(La/3.)/length(K0(S0,P2)));}if(S2!=.0){K1*=sign(determinant(S0));if((i0&N8)!=0u) K1=min(K1,.0);if((i0&sd)!=0u) K1=max(K1,.0);float X4=T2!=.0?T2:Id(S0,P2)*H4;d Od=1.;if(X4>S2&&T2==.0){Od=d4(S2)/d4(X4);S2=X4;}c G5=P2*(S2+X4);
#ifndef RENDER_MODE_DEPTH_STENCIL
float x=K1*(S2+X4);W1.xy=(1./(X4*2.))*(c(x,-x)+S2)+.5;W1.zw=X6(.0);
#endif
uint cb=i0&R3;if(cb>r7){bool X8=(i0&qd)!=0u;bool nh=(i0&N8)!=0u;float v4=jh(D2.z);float Y8=sqrt(max(1.-v4*v4,.0));if(X8==nh) Y8=-Y8;Y oh=Y(v4,Y8,-Y8,v4);c Z8=K0(oh,P2);float db=Id(S0,Z8);float eb;if((cb==Bg)||(cb==Cg&&v4>=.25)){float ph=(i0&M8)!=0u?1.:.25;eb=S2*(1./max(v4,ph));}else{eb=S2*v4+db*.5;}float fb=eb+db*H4;if((i0&rd)!=0u){float Pd=S2+X4;float qh=X4*.125;if(Pd<=fb*v4+qh){float rh=Pd*(1./v4);G5=Z8*rh;}else{c gb=Z8*fb;c sh=c(dot(G5,G5),dot(gb,gb));G5=K0(sh,inverse(Y(G5,gb)));}}c th=abs(K1)*G5;float Qd=(fb-dot(th,Z8))/(db*(H4*2.));
#ifndef RENDER_MODE_DEPTH_STENCIL
if((i0&N8)!=0u) W1.y=Qd;else W1.x=Qd;
#endif
}
#ifndef RENDER_MODE_DEPTH_STENCIL
W1.xy*=Od;W1.y=max(W1.y,1e-4);if(T2!=.0){W1.x=x7-W1.x;}
#endif
W8=K0(S0,K1*G5);if(A7!=O8) Za=true;}else{
#ifndef RENDER_MODE_DEPTH_STENCIL
W1=e(Ya,-1.,.0,.0);
#ifdef ENABLE_FEATHER
if(T2!=.0){W1.y=x7;W1.z=Ed;W1.w=Ya;if((i0&R3)==L8&&A7==O8){if(C1<.0){D7+=C1;C1=-C1;}float w4=w1-D7;w4=mod(w4+i7,F8)-i7;w4=clamp(w4,.0,C1);if(w4>C1*.5){w4=C1-w4;}c Q8=c(sin(w4),cos(w4));
#if 0
float X1=1.+.33*log2(i7/(i4-min(C1,i4-i4/16.)));e uh=Fd(C1,Q8,.5*(X1/3.));float vh=p8(uh k1);float wh=Wc(vh);float xh=(.5-wh)*(La*2.);float yh=X1/max(xh,X1);K1*=yh;
#endif
W1=Fd(C1,Q8,K1);}W8=K0(S0,(K1*T2)*P2);}else
#endif
{W8=sign(K0(K1*P2,inverse(S0)))*H4;}if(bool(i0&R2)!=bool(i0&Dg)){W1*=e(-1.,+1.,+1.,+1.);}
#endif
if(A7==ud) V8=R8;if((i0&pd)!=0u&&A7!=td){Za=true;}}lh=K0(S0,V8)+W8+m2;
#ifdef RENDER_MODE_DEPTH_STENCIL
N W3=p0(LB,m3*4u+2u);z7=O1(W3.x);
#else
W1.xy=mix(W1.xy,c(1.,-1.),cg(j.zh!=0u));
#endif
return!Za;}
#endif
#if defined(VERTEX)&&defined(DRAW_INTERIOR_TRIANGLES)
f c lc(P x6,i1(uint) m3
#ifdef RENDER_MODE_DEPTH_STENCIL
,i1(R) z7
#else
,i1(d) Ah
#endif
v6){m3=floatBitsToUint(x6.z)&0xffffu;
#ifdef RENDER_MODE_DEPTH_STENCIL
N W3=p0(LB,m3*4u+2u);z7=O1(W3.x);
#else
Ah=ya(floatBitsToInt(x6.z)>>16);
#endif
c y6=x6.xy;Y S0=n1(uintBitsToFloat(p0(LB,m3*4u)));N V3=p0(LB,m3*4u+1u);c m2=uintBitsToFloat(V3.xy);y6=K0(S0,y6)+m2;return y6;}
#endif
#if defined(VERTEX)&&defined(FEATHER_ATLAS_BLIT)
f c kc(P x6,i1(uint) m3,
#ifdef RENDER_MODE_DEPTH_STENCIL
i1(R) z7,
#endif
i1(c) Bh v6){m3=floatBitsToUint(x6.z)&0xffffu;N W3=p0(LB,m3*4u+2u);
#ifdef RENDER_MODE_DEPTH_STENCIL
z7=O1(W3.x);
#endif
c y6=x6.xy;P F7=uintBitsToFloat(W3.yzw);Bh=(y6*F7.x+F7.yz)*j.Ch;return y6;}
#endif
f d a9(d h2,d L1,d n3){return(L1-h2)/max(1.-h2*n3,I9);}
#if defined(RENDER_MODE_CLOCKWISE_ATOMIC)||defined(PLS_IMPL_STORAGE_BUFFER)
f uint c9(O0 o3,uint Dh){uint hb=(o3.y>>o6)*(Dh<<o6)+((o3.x>>o6)<<(o6<<1));hb+=((o3.x&0x1cu)<<o6)+((o3.y&0x1cu)<<2);hb+=((o3.y&0x3u)<<2)+(o3.x&0x3u);return hb;}
#endif
#ifdef RENDER_MODE_CLOCKWISE_ATOMIC
#ifdef FIXED_FUNCTION_COLOR_OUTPUT
#define x5 A2
#define n4(H5) J1=H5;A3
#else
#define x5 T1
#define n4(H5) B0(o0,H5);g2;
#endif
f d ib(uint Eh){return ya(int((Eh&Ra)-A5))*Pa;}f uint G7(d o){return uint(o*Mg+.5);}
#endif
