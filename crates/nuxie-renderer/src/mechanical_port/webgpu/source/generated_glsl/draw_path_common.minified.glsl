#define l7 -2.
#define fd -1.5
#define gd .25
#define F8 1e3
#define hd (F8*F8)
#ifdef VERTEX
X3 xc(f3,gg,JC);
#ifdef ENABLE_FEATHER
k6(f3,j7,XC);
#endif
Y3 E4 M4(Zc,Eg,OB);Q5(Tb,lf,CD);R5(Ub,mf,PB);M4(ad,Fg,HD);F4
#endif
#if defined(ENABLE_FEATHER)||defined(FEATHER_ATLAS_BLIT)
f4(j7,ga)
#endif
#ifdef FRAGMENT
I3 c3(f3,bd,DD);
#if defined(ENABLE_FEATHER)||defined(FEATHER_ATLAS_BLIT)
k6(f3,j7,XC);
#endif
#ifdef FEATHER_ATLAS_BLIT
p5(f3,cd,ED);
#endif
c3(g5,Z3,GC);
#if defined(RENDER_MODE_DEPTH_STENCIL)&&defined(ENABLE_ADVANCED_BLEND)&&!defined(FIXED_FUNCTION_COLOR_OUTPUT)
q5(XD);
#endif
J3 f4(bd,P9)
#ifdef FEATHER_ATLAS_BLIT
f4(cd,T9)
#endif
h5 a4(Y5)i5
#endif
#ifdef FRAGMENT
e bool X5(f N){return N.y>=.0;}e bool X5(E N){return N.y>=.0;}
#endif
#if defined(FRAGMENT)&&defined(ENABLE_FEATHER)
e bool bc(f N){return N.x<fd;}e bool cc(f N){return N.y<fd;}
#endif
#ifdef VERTEX
f id(float Da,c G8,float F1){c l6=(1.-G8*abs(F1))*.5;float g4,r5;if(abs(Da-Z6)<1./F8){g4=.0;r5=.0;}else{float Ea=tan(Da);g4=sign(Z6-Da)/max(abs(Ea),1./hd);r5=g4>=.0?l6.y-(1.-l6.x)*Ea:l6.y+l6.x*Ea;}f N;N.x=max(l6.x,.0)+gd;N.y=-l6.y+l7;N.z=g4;N.w=r5;return N;}
#endif
#ifdef ENABLE_FEATHER
e d g8(f N L3){d g4=N.z;d r5=max(N.w,.0);d m6=g4>=.0?m5(r5):.0;if(abs(g4)<F8){d x=abs(N.x)-gd;d y=-N.y+l7;d Z2=(y-r5)*0.5984134206;i t=r5+Z2*E0(0.20888568955,0.62665706865,1.04442844776,1.46219982687);i u=t*-g4+(y*g4+x);i Gg=E0(m5(u[0]),m5(u[1]),m5(u[2]),m5(u[3]));i jd=t*5.09593080173+-2.54796540086;i Hg=exp2(-jd*jd);m6+=dot(Gg,Hg)*Z2;}return m6*sign(N.x);}e d B4(f N L3){float m6=1.;float Ig=(1.-l7)+N.x;m6-=m5(Ig);float Jg=1.-N.y;m6-=m5(Jg);return m6;}
#endif
#if defined(VERTEX)&&defined(DRAW_PATH)
e Z v5(int kd){return Z(kd&((1<<Oc)-1),kd>>Oc);}e float ld(e0 V0,c Kg){c m2=O0(V0,Kg);return(abs(m2.x)+abs(m2.y))*(1./dot(m2,m2));}e bool x9(f m7,f Fa,int v,a1(uint)g3,a1(c)Lg
#ifndef RENDER_MODE_DEPTH_STENCIL
,a1(f)R1
#else
,a1(L)n7
#endif
n6){int H8=int(m7.x);float F1=m7.y;float Ga=m7.z;int md=floatBitsToInt(m7.w)>>2;int o7=floatBitsToInt(m7.w)&3;int Ha=min(H8,md-1);int N4=v*md+Ha;G4 w5=r1(JC,v5(N4));uint j0=l5(w5.w);uint I8=max(j0&Vc,1u);Y Ia=L0(HD,I8-1u);c nd=uintBitsToFloat(Ia.xy);g3=Ia.z&0xffffu;uint od=Ia.w;e0 V0=K1(uintBitsToFloat(L0(OB,g3*4u)));Y O4=L0(OB,g3*4u+1u);c I2=uintBitsToFloat(O4.xy);float M2=uintBitsToFloat(O4.z);float N2=uintBitsToFloat(O4.w);uint pd=j0&K3;if(pd!=0u){H8=int(Fa.x);F1=Fa.y;Ga=Fa.z;}if(H8!=Ha){int qd=N4+H8-Ha;G4 rd=r1(JC,v5(qd));if((l5(rd.w)&(K3|0xffffu))!=(j0&(K3|0xffffu))){bool Mg=M2==.0||nd.x!=.0;if(Mg){N4=int(od);w5=r1(JC,v5(N4));}}else{N4=qd;w5=rd;}j0=(l5(w5.w)&~K3)|pd;}float f1;
#ifdef ENABLE_FEATHER
float p7;float w1;if((j0&e4)==B8&&o7==E8){uint sd=l5(w5.z);float h4=float(sd&0xffffu);float n2=float(sd>>16);Z J8=Z(-h4-1.,n2-h4+1.);if((j0&K3)!=0u)J8=-J8;G4 td=r1(JC,v5(N4+J8.x));G4 Ja=r1(JC,v5(N4+J8.y));if((l5(Ja.w)&(K3|0xffffu))!=(l5(td.w)&(K3|0xffffu))){Ja=r1(JC,v5(int(od)));}p7=c6(td.z);float ud=c6(Ja.z);w1=ud-p7;if(abs(w1)>H3)w1-=v8*sign(w1);float Ka=n2+1.-float(Pc);float vd=clamp(round(abs(w1)/H3*Ka),1.,Ka-1.);float q7=Ka-vd;if(h4<=q7){w1=-(H3*sign(w1)-w1);n2=q7;if(h4==q7)F1=-F1;}else if(h4==q7+1.){h4=.0;n2=.0;F1=.0;}else{h4-=q7+2.;n2=vd;}if(h4==n2){f1=ud;}else{f1=p7+w1*(h4/n2);}}else
#endif
{f1=c6(w5.z);}c a3=c(sin(f1),-cos(f1));c wd=c6(w5.xy);c K8=c(0,0);if(N2!=.0){N2=max(N2,(ua/3.)/length(O0(V0,a3)));}if(M2!=.0){F1*=sign(determinant(V0));if((j0&D8)!=0u)F1=min(F1,.0);if((j0&Uc)!=0u)F1=max(F1,.0);float P4=N2!=.0?N2:ld(V0,a3)*w4;d xd=1.;if(P4>M2&&N2==.0){xd=Z4(M2)/Z4(P4);M2=P4;}c x5=a3*(M2+P4);
#ifndef RENDER_MODE_DEPTH_STENCIL
float x=F1*(M2+P4);R1.xy=(1./(P4*2.))*(c(x,-x)+M2)+.5;R1.zw=P6(.0);
#endif
uint La=j0&e4;if(La>A8){int r7=2;if((j0&va)==0u)r7=-r7;if((j0&K3)!=0u)r7=-r7;Z Ng=v5(N4+r7);G4 Og=r1(JC,Ng);float Pg=c6(Og.z);float v7=abs(Pg-f1);if(v7>H3)v7=v8-v7;bool L8=(j0&va)!=0u;bool Qg=(j0&D8)!=0u;float yd=v7*(L8==Qg?-.5:.5)+f1;c M8=c(sin(yd),-cos(yd));float Ma=ld(V0,M8);float w7=cos(v7*.5);float Na;if((La==bg)||(La==cg&&w7>=.25)){float Rg=(j0&C8)!=0u?1.:.25;Na=M2*(1./max(w7,Rg));}else{Na=M2*w7+Ma*.5;}float Oa=Na+Ma*w4;if((j0&Tc)!=0u){float zd=M2+P4;float Sg=P4*.125;if(zd<=Oa*w7+Sg){float Tg=zd*(1./w7);x5=M8*Tg;}else{c Pa=M8*Oa;c Ug=c(dot(x5,x5),dot(Pa,Pa));x5=O0(Ug,inverse(e0(x5,Pa)));}}c Vg=abs(F1)*x5;float Ad=(Oa-dot(Vg,M8))/(Ma*(w4*2.));
#ifndef RENDER_MODE_DEPTH_STENCIL
if((j0&D8)!=0u)R1.y=Ad;else R1.x=Ad;
#endif
}
#ifndef RENDER_MODE_DEPTH_STENCIL
R1.xy*=xd;R1.y=max(R1.y,1e-4);if(N2!=.0){R1.x=l7-R1.x;}
#endif
K8=O0(V0,F1*x5);if(o7!=E8)return false;}else{
#ifndef RENDER_MODE_DEPTH_STENCIL
R1=f(Ga,-1.,.0,.0);
#ifdef ENABLE_FEATHER
if(N2!=.0){R1.y=l7;R1.z=hd;R1.w=Ga;if((j0&e4)==B8&&o7==E8){if(w1<.0){p7+=w1;w1=-w1;}float i4=f1-p7;i4=mod(i4+Z6,v8)-Z6;i4=clamp(i4,.0,w1);if(i4>w1*.5){i4=w1-i4;}c G8=c(sin(i4),cos(i4));
#if 0
float S1=1.+.33*log2(Z6/(H3-min(w1,H3-H3/16.)));f Wg=id(w1,G8,.5*(S1/3.));float Xg=g8(Wg e1);float Yg=Ac(Xg);float Zg=(.5-Yg)*(ua*2.);float ah=S1/max(Zg,S1);F1*=ah;
#endif
R1=id(w1,G8,F1);}K8=O0(V0,(F1*N2)*a3);}else
#endif
{K8=sign(O0(F1*a3,inverse(V0)))*w4;}if(bool(j0&K3)!=bool(j0&dg)){R1*=f(-1.,+1.,+1.,+1.);}
#endif
if(o7==Xc)wd=nd;if((j0&Sc)!=0u&&o7!=Wc){return false;}}Lg=O0(V0,wd)+K8+I2;
#ifdef RENDER_MODE_DEPTH_STENCIL
Y Q4=L0(OB,g3*4u+2u);n7=a2(Q4.x);
#else
R1.xy=mix(R1.xy,c(1.,-1.),Lf(j.bh!=0u));
#endif
return true;}
#endif
#if defined(VERTEX)&&defined(DRAW_INTERIOR_TRIANGLES)
e c Nb(R o6,a1(uint)g3
#ifdef RENDER_MODE_DEPTH_STENCIL
,a1(L)n7
#else
,a1(d)ch
#endif
n6){g3=floatBitsToUint(o6.z)&0xffffu;
#ifdef RENDER_MODE_DEPTH_STENCIL
Y Q4=L0(OB,g3*4u+2u);n7=a2(Q4.x);
#else
ch=ha(floatBitsToInt(o6.z)>>16);
#endif
c p6=o6.xy;e0 V0=K1(uintBitsToFloat(L0(OB,g3*4u)));Y O4=L0(OB,g3*4u+1u);c I2=uintBitsToFloat(O4.xy);p6=O0(V0,p6)+I2;return p6;}
#endif
#if defined(VERTEX)&&defined(FEATHER_ATLAS_BLIT)
e c Mb(R o6,a1(uint)g3,
#ifdef RENDER_MODE_DEPTH_STENCIL
a1(L)n7,
#endif
a1(c)dh n6){g3=floatBitsToUint(o6.z)&0xffffu;Y Q4=L0(OB,g3*4u+2u);
#ifdef RENDER_MODE_DEPTH_STENCIL
n7=a2(Q4.x);
#endif
c p6=o6.xy;R x7=uintBitsToFloat(Q4.yzw);dh=(p6*x7.x+x7.yz)*j.eh;return p6;}
#endif
e d N8(d e2,d G1,d h3){return(G1-e2)/max(1.-e2*h3,v9);}
#if defined(RENDER_MODE_CLOCKWISE_ATOMIC)||defined(PLS_IMPL_STORAGE_BUFFER)
e uint O8(c1 j4,uint fh){uint Qa=(j4.y>>j6)*(fh<<j6)+((j4.x>>j6)<<(j6<<1));Qa+=((j4.x&0x1cu)<<j6)+((j4.y&0x1cu)<<2);Qa+=((j4.y&0x3u)<<2)+(j4.x&0x3u);return Qa;}
#endif
#ifdef RENDER_MODE_CLOCKWISE_ATOMIC
#ifdef FIXED_FUNCTION_COLOR_OUTPUT
#define j5 v2
#define c4(y5) E1=y5;p3
#else
#define j5 O1
#define c4(y5) A0(l0,y5);d2;
#endif
e d Ra(uint gh){return ha(int((gh&Aa)-o5))*ya;}e uint y7(d o){return uint(o*mg+.5);}
#endif
