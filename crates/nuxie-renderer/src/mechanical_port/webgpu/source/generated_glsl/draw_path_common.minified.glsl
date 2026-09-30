#define y7 -2.
#define Dd -1.5
#define Ed .25
#define R8 1e3
#define Fd (R8*R8)
#ifdef VERTEX
k4 W4(l3,Pg,TB);
#ifdef ENABLE_FEATHER
r6(l3,w7,ZC);
#endif
l4 Q4 X4(yd,nh,LB);a6(pc,Ff,XC);c6(qc,Gf,JB);X4(zd,oh,AD);R4
#endif
#if defined(ENABLE_FEATHER)||defined(FEATHER_ATLAS_BLIT)
p4(w7,xa)
#endif
#ifdef FRAGMENT
O3 i3(l3,Ad,FD);
#if defined(ENABLE_FEATHER)||defined(FEATHER_ATLAS_BLIT)
r6(l3,w7,ZC);
#endif
#ifdef FEATHER_ATLAS_BLIT
E5(l3,Bd,GD);
#endif
i3(x5,m4,DC);
#if defined(RENDER_MODE_DEPTH_STENCIL)&&defined(ENABLE_ADVANCED_BLEND)&&!defined(FIXED_FUNCTION_COLOR_OUTPUT)
F5(YD);
#endif
P3 p4(Ad,ia)
#ifdef FEATHER_ATLAS_BLIT
p4(Bd,na)
#endif
y5 n4(v5) z5
#endif
#ifdef FRAGMENT
e bool g6(f U){return U.y>=.0;}e bool g6(C U){return U.y>=.0;}
#endif
#if defined(FRAGMENT)&&defined(ENABLE_FEATHER)
e bool zc(f U){return U.x<Dd;}e bool Ac(f U){return U.y<Dd;}
#endif
#ifdef VERTEX
f Gd(float Va,c S8,float L1){c v6=(1.-S8*abs(L1))*.5;float q4,G5;if(abs(Va-j7)<1./R8){q4=.0;G5=.0;}else{float Wa=tan(Va);q4=sign(j7-Va)/max(abs(Wa),1./Fd);G5=q4>=.0?v6.y-(1.-v6.x)*Wa:v6.y+v6.x*Wa;}f U;U.x=max(v6.x,.0)+Ed;U.y=-v6.y+y7;U.z=q4;U.w=G5;return U;}
#endif
#ifdef ENABLE_FEATHER
e d r8(f U S3){d q4=U.z;d G5=max(U.w,.0);d w6=q4>=.0?B5(G5):.0;if(abs(q4)<R8){d x=abs(U.x)-Ed;d y=-U.y+y7;d h3=(y-G5)*0.5984134206;i t=G5+h3*G0(0.20888568955,0.62665706865,1.04442844776,1.46219982687);i u=t*-q4+(y*q4+x);i ph=G0(B5(u[0]),B5(u[1]),B5(u[2]),B5(u[3]));i Hd=t*5.09593080173+-2.54796540086;i qh=exp2(-Hd*Hd);w6+=dot(ph,qh)*h3;}return w6*sign(U.x);}e d N4(f U S3){float w6=1.;float rh=(1.-y7)+U.x;w6-=B5(rh);float sh=1.-U.y;w6-=B5(sh);return w6;}
#endif
#ifdef VERTEX
e e0 r4(int Id){return e0(Id&((1<<kd)-1),Id>>kd);}e float Xa(uint z){return float(z)*(H8/(65536.*65536.));}e float th(uint z){return float(z&0xffffu)*(1./65535.);}
#endif
#if defined(VERTEX)&&defined(DRAW_PATH)
e float Jd(Y T0,c uh){c v2=M0(T0,uh);return(abs(v2.x)+abs(v2.y))*(1./dot(v2,v2));}e bool L9(f z7,f Ya,int r,i1(uint) m3,i1(c) vh
#ifndef RENDER_MODE_DEPTH_STENCIL
,i1(f) W1
#else
,i1(R) A7
#endif
x6){int T3=int(z7.x);float L1=z7.y;float Za=z7.z;int Kd=floatBitsToInt(z7.w)>>2;int B7=floatBitsToInt(z7.w)&3;int H5=min(T3,Kd-1);int U3=r*Kd+H5;N D2=p1(TB,r4(U3));uint i0=D2.w;uint y6=max(i0&Na,1u);N I5=p0(AD,y6-1u);c T8=uintBitsToFloat(I5.xy);m3=I5.z&0xffffu;uint U8=I5.w;Y T0=n1(uintBitsToFloat(p0(LB,m3*4u)));N V3=p0(LB,m3*4u+1u);c m2=uintBitsToFloat(V3.xy);float S2=uintBitsToFloat(V3.z);float T2=uintBitsToFloat(V3.w);uint C7=i0&R2;if(C7!=0u){T3=int(Ya.x);L1=Ya.y;Za=Ya.z;}if(T3!=H5){int V8=U3+T3-H5;N D7=p1(TB,r4(V8));if((D7.w&(R2|0xffffu))!=(i0&(R2|0xffffu))){bool wh=S2==.0||T8.x!=.0;if(wh){U3=int(U8);D2=p1(TB,r4(U3));}}else{U3=V8;D2=D7;}i0=(D2.w&~R2)|C7;}bool ab=false;float x1;
#ifdef ENABLE_FEATHER
float E7;float D1;if((i0&R3)==N8&&B7==Q8){uint Ld=D2.z;float v4=float(Ld&0xffffu);float w2=float(Ld>>16);e0 W8=e0(-v4-1.,w2-v4+1.);if((i0&R2)!=0u) W8=-W8;N Md=p1(TB,r4(U3+W8.x));N bb=p1(TB,r4(U3+W8.y));if((bb.w&(R2|0xffffu))!=(Md.w&(R2|0xffffu))){bb=p1(TB,r4(int(U8)));}E7=Xa(Md.z);float Nd=Xa(bb.z);D1=Nd-E7;if(abs(D1)>j4) D1-=H8*sign(D1);float cb=w2+1.-float(nd);float Od=clamp(round(abs(D1)/j4*cb),1.,cb-1.);float F7=cb-Od;if(v4<=F7){D1=-(j4*sign(D1)-D1);w2=F7;if(v4==F7) L1=-L1;}else if(v4==F7+1.){v4=.0;w2=.0;L1=.0;}else{v4-=F7+2.;w2=Od;}if(v4==w2){x1=Nd;}else{x1=E7+D1*(v4/w2);}}else
#endif
{x1=Xa(D2.z);}c P2=c(sin(x1),-cos(x1));c X8=uintBitsToFloat(D2.xy);c Y8=c(0,0);if(T2!=.0){T2=max(T2,(Ma/3.)/length(M0(T0,P2)));}if(S2!=.0){L1*=sign(determinant(T0));if((i0&P8)!=0u) L1=min(L1,.0);if((i0&td)!=0u) L1=max(L1,.0);float Y4=T2!=.0?T2:Jd(T0,P2)*I4;d Pd=1.;if(Y4>S2&&T2==.0){Pd=e4(S2)/e4(Y4);S2=Y4;}c J5=P2*(S2+Y4);
#ifndef RENDER_MODE_DEPTH_STENCIL
float x=L1*(S2+Y4);W1.xy=(1./(Y4*2.))*(c(x,-x)+S2)+.5;W1.zw=Z6(.0);
#endif
uint db=i0&R3;if(db>v7){bool Z8=(i0&rd)!=0u;bool xh=(i0&P8)!=0u;float w4=th(D2.z);float a9=sqrt(max(1.-w4*w4,.0));if(Z8==xh) a9=-a9;Y yh=Y(w4,a9,-a9,w4);c c9=M0(yh,P2);float eb=Jd(T0,c9);float fb;if((db==Fg)||(db==Gg&&w4>=.25)){float zh=(i0&O8)!=0u?1.:.25;fb=S2*(1./max(w4,zh));}else{fb=S2*w4+eb*.5;}float gb=fb+eb*I4;if((i0&sd)!=0u){float Qd=S2+Y4;float Ah=Y4*.125;if(Qd<=gb*w4+Ah){float Bh=Qd*(1./w4);J5=c9*Bh;}else{c hb=c9*gb;c Ch=c(dot(J5,J5),dot(hb,hb));J5=M0(Ch,inverse(Y(J5,hb)));}}c Dh=abs(L1)*J5;float Rd=(gb-dot(Dh,c9))/(eb*(I4*2.));
#ifndef RENDER_MODE_DEPTH_STENCIL
if((i0&P8)!=0u) W1.y=Rd;else W1.x=Rd;
#endif
}
#ifndef RENDER_MODE_DEPTH_STENCIL
W1.xy*=Pd;W1.y=max(W1.y,1e-4);if(T2!=.0){W1.x=y7-W1.x;}
#endif
Y8=M0(T0,L1*J5);if(B7!=Q8) ab=true;}else{
#ifndef RENDER_MODE_DEPTH_STENCIL
W1=f(Za,-1.,.0,.0);
#ifdef ENABLE_FEATHER
if(T2!=.0){W1.y=y7;W1.z=Fd;W1.w=Za;if((i0&R3)==N8&&B7==Q8){if(D1<.0){E7+=D1;D1=-D1;}float x4=x1-E7;x4=mod(x4+j7,H8)-j7;x4=clamp(x4,.0,D1);if(x4>D1*.5){x4=D1-x4;}c S8=c(sin(x4),cos(x4));
#if 0
float X1=1.+.33*log2(j7/(j4-min(D1,j4-j4/16.)));f Eh=Gd(D1,S8,.5*(X1/3.));float Fh=r8(Eh k1);float Gh=Xc(Fh);float Hh=(.5-Gh)*(Ma*2.);float Ih=X1/max(Hh,X1);L1*=Ih;
#endif
W1=Gd(D1,S8,L1);}Y8=M0(T0,(L1*T2)*P2);}else
#endif
{Y8=sign(M0(L1*P2,inverse(T0)))*I4;}if(bool(i0&R2)!=bool(i0&Hg)){W1*=f(-1.,+1.,+1.,+1.);}
#endif
if(B7==vd) X8=T8;if((i0&qd)!=0u&&B7!=ud){ab=true;}}vh=M0(T0,X8)+Y8+m2;
#ifdef RENDER_MODE_DEPTH_STENCIL
N W3=p0(LB,m3*4u+2u);A7=P1(W3.x);
#else
W1.xy=mix(W1.xy,c(1.,-1.),gg(j.Jh!=0u));
#endif
return!ab;}
#endif
#if defined(VERTEX)&&defined(DRAW_INTERIOR_TRIANGLES)
e c mc(P z6,i1(uint) m3
#ifdef RENDER_MODE_DEPTH_STENCIL
,i1(R) A7
#else
,i1(d) Kh
#endif
x6){m3=floatBitsToUint(z6.z)&0xffffu;
#ifdef RENDER_MODE_DEPTH_STENCIL
N W3=p0(LB,m3*4u+2u);A7=P1(W3.x);
#else
Kh=za(floatBitsToInt(z6.z)>>16);
#endif
c A6=z6.xy;Y T0=n1(uintBitsToFloat(p0(LB,m3*4u)));N V3=p0(LB,m3*4u+1u);c m2=uintBitsToFloat(V3.xy);A6=M0(T0,A6)+m2;return A6;}
#endif
#if defined(VERTEX)&&defined(FEATHER_ATLAS_BLIT)
e c lc(P z6,i1(uint) m3,
#ifdef RENDER_MODE_DEPTH_STENCIL
i1(R) A7,
#endif
i1(c) Lh x6){m3=floatBitsToUint(z6.z)&0xffffu;N W3=p0(LB,m3*4u+2u);
#ifdef RENDER_MODE_DEPTH_STENCIL
A7=P1(W3.x);
#endif
c A6=z6.xy;P G7=uintBitsToFloat(W3.yzw);Lh=(A6*G7.x+G7.yz)*j.Mh;return A6;}
#endif
e d d9(d i2,d M1,d n3){return(M1-i2)/max(1.-i2*n3,K9);}
#if defined(RENDER_MODE_CLOCKWISE_ATOMIC)||defined(PLS_IMPL_STORAGE_BUFFER)
e uint e9(O0 o3,uint Nh){uint ib=(o3.y>>q6)*(Nh<<q6)+((o3.x>>q6)<<(q6<<1));ib+=((o3.x&0x1cu)<<q6)+((o3.y&0x1cu)<<2);ib+=((o3.y&0x3u)<<2)+(o3.x&0x3u);return ib;}
#endif
#ifdef RENDER_MODE_CLOCKWISE_ATOMIC
#ifdef FIXED_FUNCTION_COLOR_OUTPUT
#define A5 A2
#define o4(K5) K1=K5;A3
#else
#define A5 T1
#define o4(K5) y0(n0,K5);h2;
#endif
e d jb(uint Oh){return za(int((Oh&Sa)-D5))*Qa;}e uint H7(d o){return uint(o*Wg+.5);}
#endif
