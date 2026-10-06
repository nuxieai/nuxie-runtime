#define w7 -2.
#define Dd -1.5
#define Ed .25
#define Q8 1e3
#define Fd (Q8*Q8)
#ifdef VERTEX
k4 W4(l3,Rg,TB);
#ifdef ENABLE_FEATHER
q6(l3,r7,YC);
#endif
l4 Q4 X4(yd,ph,LB);Z5(pc,Hf,WC);a6(qc,If,JB);X4(zd,qh,ZC);R4
#endif
#if defined(ENABLE_FEATHER)||defined(FEATHER_ATLAS_BLIT)
p4(r7,xa)
#endif
#ifdef FRAGMENT
O3 i3(l3,Ad,ED);
#if defined(ENABLE_FEATHER)||defined(FEATHER_ATLAS_BLIT)
q6(l3,r7,YC);
#endif
#ifdef FEATHER_ATLAS_BLIT
D5(l3,Bd,FD);
#endif
i3(w5,m4,CC);
#if defined(RENDER_MODE_DEPTH_STENCIL)&&defined(ENABLE_ADVANCED_BLEND)&&!defined(FIXED_FUNCTION_COLOR_OUTPUT)
E5(XD);
#endif
P3 p4(Ad,ia)
#ifdef FEATHER_ATLAS_BLIT
p4(Bd,na)
#endif
x5 n4(r5) y5
#endif
#ifdef FRAGMENT
f bool f6(e U){return U.y>=.0;}f bool f6(C U){return U.y>=.0;}
#endif
#if defined(FRAGMENT)&&defined(ENABLE_FEATHER)
f bool zc(e U){return U.x<Dd;}f bool Ac(e U){return U.y<Dd;}
#endif
#ifdef VERTEX
e Gd(float Va,c R8,float M1){c r6=(1.-R8*abs(M1))*.5;float q4,F5;if(abs(Va-i7)<1./Q8){q4=.0;F5=.0;}else{float Wa=tan(Va);q4=sign(i7-Va)/max(abs(Wa),1./Fd);F5=q4>=.0?r6.y-(1.-r6.x)*Wa:r6.y+r6.x*Wa;}e U;U.x=max(r6.x,.0)+Ed;U.y=-r6.y+w7;U.z=q4;U.w=F5;return U;}
#endif
#ifdef ENABLE_FEATHER
f d p8(e U S3){d q4=U.z;d F5=max(U.w,.0);d v6=q4>=.0?A5(F5):.0;if(abs(q4)<Q8){d x=abs(U.x)-Ed;d y=-U.y+w7;d h3=(y-F5)*0.5984134206;i t=F5+h3*I0(0.20888568955,0.62665706865,1.04442844776,1.46219982687);i u=t*-q4+(y*q4+x);i rh=I0(A5(u[0]),A5(u[1]),A5(u[2]),A5(u[3]));i Hd=t*5.09593080173+-2.54796540086;i sh=exp2(-Hd*Hd);v6+=dot(rh,sh)*h3;}return v6*sign(U.x);}f d N4(e U S3){float v6=1.;float th=(1.-w7)+U.x;v6-=A5(th);float uh=1.-U.y;v6-=A5(uh);return v6;}
#endif
#ifdef VERTEX
f e0 r4(int Id){return e0(Id&((1<<kd)-1),Id>>kd);}f float Xa(uint z){return float(z)*(F8/(65536.*65536.));}f float vh(uint z){return float(z&0xffffu)*(1./65535.);}
#endif
#if defined(VERTEX)&&defined(DRAW_PATH)
f float Jd(Y S0,c wh){c r2=M0(S0,wh);return(abs(r2.x)+abs(r2.y))*(1./dot(r2,r2));}f bool L9(e x7,e Ya,int r,i1(uint) m3,i1(c) xh
#ifndef RENDER_MODE_DEPTH_STENCIL
,i1(e) X1
#else
,i1(Q) y7
#endif
w6){int T3=int(x7.x);float M1=x7.y;float Za=x7.z;int Kd=floatBitsToInt(x7.w)>>2;int z7=floatBitsToInt(x7.w)&3;int G5=min(T3,Kd-1);int U3=r*Kd+G5;M C2=p1(TB,r4(U3));uint i0=C2.w;uint x6=max(i0&Na,1u);M H5=p0(ZC,x6-1u);c S8=uintBitsToFloat(H5.xy);m3=H5.z&0xffffu;uint T8=H5.w;Y S0=n1(uintBitsToFloat(p0(LB,m3*4u)));M V3=p0(LB,m3*4u+1u);c m2=uintBitsToFloat(V3.xy);float R2=uintBitsToFloat(V3.z);float S2=uintBitsToFloat(V3.w);uint A7=i0&Q2;if(A7!=0u){T3=int(Ya.x);M1=Ya.y;Za=Ya.z;}if(T3!=G5){int U8=U3+T3-G5;M B7=p1(TB,r4(U8));if((B7.w&(Q2|0xffffu))!=(i0&(Q2|0xffffu))){bool yh=R2==.0||S8.x!=.0;if(yh){U3=int(T8);C2=p1(TB,r4(U3));}}else{U3=U8;C2=B7;}i0=(C2.w&~Q2)|A7;}bool ab=false;float y1;
#ifdef ENABLE_FEATHER
float C7;float E1;if((i0&R3)==M8&&z7==P8){uint Ld=C2.z;float v4=float(Ld&0xffffu);float v2=float(Ld>>16);e0 V8=e0(-v4-1.,v2-v4+1.);if((i0&Q2)!=0u) V8=-V8;M Md=p1(TB,r4(U3+V8.x));M bb=p1(TB,r4(U3+V8.y));if((bb.w&(Q2|0xffffu))!=(Md.w&(Q2|0xffffu))){bb=p1(TB,r4(int(T8)));}C7=Xa(Md.z);float Nd=Xa(bb.z);E1=Nd-C7;if(abs(E1)>j4) E1-=F8*sign(E1);float cb=v2+1.-float(nd);float Od=clamp(round(abs(E1)/j4*cb),1.,cb-1.);float D7=cb-Od;if(v4<=D7){E1=-(j4*sign(E1)-E1);v2=D7;if(v4==D7) M1=-M1;}else if(v4==D7+1.){v4=.0;v2=.0;M1=.0;}else{v4-=D7+2.;v2=Od;}if(v4==v2){y1=Nd;}else{y1=C7+E1*(v4/v2);}}else
#endif
{y1=Xa(C2.z);}c O2=c(sin(y1),-cos(y1));c W8=uintBitsToFloat(C2.xy);c X8=c(0,0);if(S2!=.0){S2=max(S2,(Ma/3.)/length(M0(S0,O2)));}if(R2!=.0){M1*=sign(determinant(S0));if((i0&O8)!=0u) M1=min(M1,.0);if((i0&td)!=0u) M1=max(M1,.0);float Y4=S2!=.0?S2:Jd(S0,O2)*I4;d Pd=1.;if(Y4>R2&&S2==.0){Pd=e4(R2)/e4(Y4);R2=Y4;}c I5=O2*(R2+Y4);
#ifndef RENDER_MODE_DEPTH_STENCIL
float x=M1*(R2+Y4);X1.xy=(1./(Y4*2.))*(c(x,-x)+R2)+.5;X1.zw=Y6(.0);
#endif
uint db=i0&R3;if(db>L8){bool Y8=(i0&rd)!=0u;bool zh=(i0&O8)!=0u;float w4=vh(C2.z);float Z8=sqrt(max(1.-w4*w4,.0));if(Y8==zh) Z8=-Z8;Y Ah=Y(w4,Z8,-Z8,w4);c a9=M0(Ah,O2);float eb=Jd(S0,a9);float fb;if((db==Hg)||(db==Ig&&w4>=.25)){float Bh=(i0&N8)!=0u?1.:.25;fb=R2*(1./max(w4,Bh));}else{fb=R2*w4+eb*.5;}float gb=fb+eb*I4;if((i0&sd)!=0u){float Qd=R2+Y4;float Ch=Y4*.125;if(Qd<=gb*w4+Ch){float Dh=Qd*(1./w4);I5=a9*Dh;}else{c hb=a9*gb;c Eh=c(dot(I5,I5),dot(hb,hb));I5=M0(Eh,inverse(Y(I5,hb)));}}c Fh=abs(M1)*I5;float Rd=(gb-dot(Fh,a9))/(eb*(I4*2.));
#ifndef RENDER_MODE_DEPTH_STENCIL
if((i0&O8)!=0u) X1.y=Rd;else X1.x=Rd;
#endif
}
#ifndef RENDER_MODE_DEPTH_STENCIL
X1.xy*=Pd;X1.y=max(X1.y,1e-4);if(S2!=.0){X1.x=w7-X1.x;}
#endif
X8=M0(S0,M1*I5);if(z7!=P8) ab=true;}else{
#ifndef RENDER_MODE_DEPTH_STENCIL
X1=e(Za,-1.,.0,.0);
#ifdef ENABLE_FEATHER
if(S2!=.0){X1.y=w7;X1.z=Fd;X1.w=Za;if((i0&R3)==M8&&z7==P8){if(E1<.0){C7+=E1;E1=-E1;}float x4=y1-C7;x4=mod(x4+i7,F8)-i7;x4=clamp(x4,.0,E1);if(x4>E1*.5){x4=E1-x4;}c R8=c(sin(x4),cos(x4));
#if 0
float Y1=1.+.33*log2(i7/(j4-min(E1,j4-j4/16.)));e Gh=Gd(E1,R8,.5*(Y1/3.));float Hh=p8(Gh k1);float Ih=Xc(Hh);float Jh=(.5-Ih)*(Ma*2.);float Kh=Y1/max(Jh,Y1);M1*=Kh;
#endif
X1=Gd(E1,R8,M1);}X8=M0(S0,(M1*S2)*O2);}else
#endif
{X8=sign(M0(M1*O2,inverse(S0)))*I4;}if(bool(i0&Q2)!=bool(i0&Jg)){X1*=e(-1.,+1.,+1.,+1.);}
#endif
if(z7==vd) W8=S8;if((i0&qd)!=0u&&z7!=ud){ab=true;}}xh=M0(S0,W8)+X8+m2;
#ifdef RENDER_MODE_DEPTH_STENCIL
M W3=p0(LB,m3*4u+2u);y7=Q1(W3.x);
#else
X1.xy=mix(X1.xy,c(1.,-1.),ig(j.Lh!=0u));
#endif
return!ab;}
#endif
#if defined(VERTEX)&&defined(DRAW_INTERIOR_TRIANGLES)
f c mc(O y6,i1(uint) m3
#ifdef RENDER_MODE_DEPTH_STENCIL
,i1(Q) y7
#else
,i1(d) Mh
#endif
w6){m3=floatBitsToUint(y6.z)&0xffffu;
#ifdef RENDER_MODE_DEPTH_STENCIL
M W3=p0(LB,m3*4u+2u);y7=Q1(W3.x);
#else
Mh=za(floatBitsToInt(y6.z)>>16);
#endif
c z6=y6.xy;Y S0=n1(uintBitsToFloat(p0(LB,m3*4u)));M V3=p0(LB,m3*4u+1u);c m2=uintBitsToFloat(V3.xy);z6=M0(S0,z6)+m2;return z6;}
#endif
#if defined(VERTEX)&&defined(FEATHER_ATLAS_BLIT)
f c lc(O y6,i1(uint) m3,
#ifdef RENDER_MODE_DEPTH_STENCIL
i1(Q) y7,
#endif
i1(c) Nh w6){m3=floatBitsToUint(y6.z)&0xffffu;M W3=p0(LB,m3*4u+2u);
#ifdef RENDER_MODE_DEPTH_STENCIL
y7=Q1(W3.x);
#endif
c z6=y6.xy;O E7=uintBitsToFloat(W3.yzw);Nh=(z6*E7.x+E7.yz)*j.Oh;return z6;}
#endif
f d c9(d i2,d N1,d n3){return(N1-i2)/max(1.-i2*n3,K9);}
#if defined(RENDER_MODE_CLOCKWISE_ATOMIC)||defined(PLS_IMPL_STORAGE_BUFFER)
f uint d9(O0 o3,uint Ph){uint ib=(o3.y>>p6)*(Ph<<p6)+((o3.x>>p6)<<(p6<<1));ib+=((o3.x&0x1cu)<<p6)+((o3.y&0x1cu)<<2);ib+=((o3.y&0x3u)<<2)+(o3.x&0x3u);return ib;}
#endif
#ifdef RENDER_MODE_CLOCKWISE_ATOMIC
#ifdef FIXED_FUNCTION_COLOR_OUTPUT
#define z5 z2
#define o4(J5) L1=J5;A3
#else
#define z5 U1
#define o4(J5) y0(n0,J5);h2;
#endif
f d jb(uint Qh){return za(int((Qh&Sa)-C5))*Qa;}f uint F7(d n){return uint(n*Yg+.5);}
#endif
