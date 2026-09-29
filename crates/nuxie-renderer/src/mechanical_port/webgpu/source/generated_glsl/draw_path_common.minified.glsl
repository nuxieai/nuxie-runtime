#define k7 -2.
#define Uc -1.5
#define Vc .25
#define D8 1e3
#define Wc (D8*D8)
#ifdef VERTEX
U3 nc(d3,Of,MC);
#ifdef ENABLE_FEATHER
j6(d3,i7,YC);
#endif
V3 C4 K4(Pc,mg,QB);O5(Lb,Re,BD);P5(Mb,Se,RB);K4(Qc,ng,FD);D4
#endif
#if defined(ENABLE_FEATHER)||defined(FEATHER_ATLAS_BLIT)
c4(i7,ba)
#endif
#ifdef FRAGMENT
E3 Z2(d3,Rc,ND);
#if defined(ENABLE_FEATHER)||defined(FEATHER_ATLAS_BLIT)
j6(d3,i7,YC);
#endif
#ifdef FEATHER_ATLAS_BLIT
n5(d3,Sc,CD);
#endif
Z2(e5,W3,JC);
#if defined(RENDER_MODE_DEPTH_STENCIL)&&defined(ENABLE_ADVANCED_BLEND)&&!defined(FIXED_FUNCTION_COLOR_OUTPUT)
k6(VD);
#endif
F3 c4(Rc,Rb)
#ifdef FEATHER_ATLAS_BLIT
c4(Sc,R9)
#endif
f5 X3(W5)g5
#endif
#ifdef FRAGMENT
e bool V5(g P){return P.y>=.0;}e bool V5(E P){return P.y>=.0;}
#endif
#if defined(FRAGMENT)&&defined(ENABLE_FEATHER)
e bool Sb(g P){return P.x<Uc;}e bool Tb(g P){return P.y<Uc;}
#endif
#ifdef VERTEX
g Xc(float xa,d E8,float E1){d l6=(1.-E8*abs(E1))*.5;float d4,o5;if(abs(xa-Y6)<1./D8){d4=.0;o5=.0;}else{float ya=tan(xa);d4=sign(Y6-xa)/max(abs(ya),1./Wc);o5=d4>=.0?l6.y-(1.-l6.x)*ya:l6.y+l6.x*ya;}g P;P.x=max(l6.x,.0)+Vc;P.y=-l6.y+k7;P.z=d4;P.w=o5;return P;}
#endif
#ifdef ENABLE_FEATHER
e c e8(g P I3){c d4=P.z;c o5=max(P.w,.0);c m6=d4>=.0?k5(o5):.0;if(abs(d4)<D8){c x=abs(P.x)-Vc;c y=-P.y+k7;c X2=(y-o5)*0.5984134206;i t=o5+X2*C0(0.20888568955,0.62665706865,1.04442844776,1.46219982687);i u=t*-d4+(y*d4+x);i og=C0(k5(u[0]),k5(u[1]),k5(u[2]),k5(u[3]));i Yc=t*5.09593080173+-2.54796540086;i pg=exp2(-Yc*Yc);m6+=dot(og,pg)*X2;}return m6*sign(P.x);}e c z4(g P I3){float m6=1.;float qg=(1.-k7)+P.x;m6-=k5(qg);float rg=1.-P.y;m6-=k5(rg);return m6;}
#endif
#if defined(VERTEX)&&defined(DRAW_PATH)
e Y p5(int Zc){return Y(Zc&((1<<Ec)-1),Zc>>Ec);}e float ad(f0 U0,d sg){d j2=R0(U0,sg);return(abs(j2.x)+abs(j2.y))*(1./dot(j2,j2));}e bool r9(g l7,g za,int v,Z0(uint)e3,Z0(d)tg
#ifndef RENDER_MODE_DEPTH_STENCIL
,Z0(g)P1
#else
,Z0(N)m7
#endif
n6){int F8=int(l7.x);float E1=l7.y;float Aa=l7.z;int bd=floatBitsToInt(l7.w)>>2;int n7=floatBitsToInt(l7.w)&3;int Ba=min(F8,bd-1);int L4=v*bd+Ba;E4 q5=q1(MC,p5(L4));uint i0=j5(q5.w);uint G8=max(i0&Lc,1u);H Ca=J0(FD,G8-1u);d cd=uintBitsToFloat(Ca.xy);e3=Ca.z&0xffffu;uint dd=Ca.w;f0 U0=h2(uintBitsToFloat(J0(QB,e3*4u)));H M4=J0(QB,e3*4u+1u);d k3=uintBitsToFloat(M4.xy);float J2=uintBitsToFloat(M4.z);float K2=uintBitsToFloat(M4.w);uint ed=i0&G3;if(ed!=0u){F8=int(za.x);E1=za.y;Aa=za.z;}if(F8!=Ba){int fd=L4+F8-Ba;E4 gd=q1(MC,p5(fd));if((j5(gd.w)&(G3|0xffffu))!=(i0&(G3|0xffffu))){bool ug=J2==.0||cd.x!=.0;if(ug){L4=int(dd);q5=q1(MC,p5(L4));}}else{L4=fd;q5=gd;}i0=(j5(q5.w)&~G3)|ed;}float e1;
#ifdef ENABLE_FEATHER
float o7;float v1;if((i0&a4)==z8&&n7==C8){uint hd=j5(q5.z);float e4=float(hd&0xffffu);float k2=float(hd>>16);Y H8=Y(-e4-1.,k2-e4+1.);if((i0&G3)!=0u)H8=-H8;E4 id=q1(MC,p5(L4+H8.x));E4 Da=q1(MC,p5(L4+H8.y));if((j5(Da.w)&(G3|0xffffu))!=(j5(id.w)&(G3|0xffffu))){Da=q1(MC,p5(int(dd)));}o7=Z5(id.z);float jd=Z5(Da.z);v1=jd-o7;if(abs(v1)>D3)v1-=q8*sign(v1);float Ea=k2+1.-float(Fc);float kd=clamp(round(abs(v1)/D3*Ea),1.,Ea-1.);float p7=Ea-kd;if(e4<=p7){v1=-(D3*sign(v1)-v1);k2=p7;if(e4==p7)E1=-E1;}else if(e4==p7+1.){e4=.0;k2=.0;E1=.0;}else{e4-=p7+2.;k2=kd;}if(e4==k2){e1=jd;}else{e1=o7+v1*(e4/k2);}}else
#endif
{e1=Z5(q5.z);}d Y2=d(sin(e1),-cos(e1));d ld=Z5(q5.xy);d I8=d(0,0);if(K2!=.0){K2=max(K2,(pa/3.)/length(R0(U0,Y2)));}if(J2!=.0){E1*=sign(determinant(U0));if((i0&B8)!=0u)E1=min(E1,.0);if((i0&Kc)!=0u)E1=max(E1,.0);float N4=K2!=.0?K2:ad(U0,Y2)*q4;c md=1.;if(N4>J2&&K2==.0){md=X4(J2)/X4(N4);J2=N4;}d r5=Y2*(J2+N4);
#ifndef RENDER_MODE_DEPTH_STENCIL
float x=E1*(J2+N4);P1.xy=(1./(N4*2.))*(d(x,-x)+J2)+.5;P1.zw=O6(.0);
#endif
uint Fa=i0&a4;if(Fa>y8){int q7=2;if((i0&qa)==0u)q7=-q7;if((i0&G3)!=0u)q7=-q7;Y vg=p5(L4+q7);E4 wg=q1(MC,vg);float xg=Z5(wg.z);float r7=abs(xg-e1);if(r7>D3)r7=q8-r7;bool J8=(i0&qa)!=0u;bool yg=(i0&B8)!=0u;float nd=r7*(J8==yg?-.5:.5)+e1;d K8=d(sin(nd),-cos(nd));float Ga=ad(U0,K8);float v7=cos(r7*.5);float Ha;if((Fa==If)||(Fa==Jf&&v7>=.25)){float zg=(i0&A8)!=0u?1.:.25;Ha=J2*(1./max(v7,zg));}else{Ha=J2*v7+Ga*.5;}float Ia=Ha+Ga*q4;if((i0&Jc)!=0u){float od=J2+N4;float Ag=N4*.125;if(od<=Ia*v7+Ag){float Bg=od*(1./v7);r5=K8*Bg;}else{d Ja=K8*Ia;d Cg=d(dot(r5,r5),dot(Ja,Ja));r5=R0(Cg,inverse(f0(r5,Ja)));}}d Dg=abs(E1)*r5;float pd=(Ia-dot(Dg,K8))/(Ga*(q4*2.));
#ifndef RENDER_MODE_DEPTH_STENCIL
if((i0&B8)!=0u)P1.y=pd;else P1.x=pd;
#endif
}
#ifndef RENDER_MODE_DEPTH_STENCIL
P1.xy*=md;P1.y=max(P1.y,1e-4);if(K2!=.0){P1.x=k7-P1.x;}
#endif
I8=R0(U0,E1*r5);if(n7!=C8)return false;}else{
#ifndef RENDER_MODE_DEPTH_STENCIL
P1=g(Aa,-1.,.0,.0);
#ifdef ENABLE_FEATHER
if(K2!=.0){P1.y=k7;P1.z=Wc;P1.w=Aa;if((i0&a4)==z8&&n7==C8){if(v1<.0){o7+=v1;v1=-v1;}float f4=e1-o7;f4=mod(f4+Y6,q8)-Y6;f4=clamp(f4,.0,v1);if(f4>v1*.5){f4=v1-f4;}d E8=d(sin(f4),cos(f4));
#if 0
float Q1=1.+.33*log2(Y6/(D3-min(v1,D3-D3/16.)));g Eg=Xc(v1,E8,.5*(Q1/3.));float Fg=e8(Eg d1);float Gg=qc(Fg);float Hg=(.5-Gg)*(pa*2.);float Ig=Q1/max(Hg,Q1);E1*=Ig;
#endif
P1=Xc(v1,E8,E1);}I8=R0(U0,(E1*K2)*Y2);}else
#endif
{I8=sign(R0(E1*Y2,inverse(U0)))*q4;}if(bool(i0&G3)!=bool(i0&Kf)){P1*=g(-1.,+1.,+1.,+1.);}
#endif
if(n7==Nc)ld=cd;if((i0&Ic)!=0u&&n7!=Mc){return false;}}tg=R0(U0,ld)+I8+k3;
#ifdef RENDER_MODE_DEPTH_STENCIL
H O4=J0(QB,e3*4u+2u);m7=X1(O4.x);
#else
P1.xy=mix(P1.xy,d(1.,-1.),sf(m.Jg!=0u));
#endif
return true;}
#endif
#if defined(VERTEX)&&defined(DRAW_INTERIOR_TRIANGLES)
e d Jb(R o6,Z0(uint)e3
#ifdef RENDER_MODE_DEPTH_STENCIL
,Z0(N)m7
#else
,Z0(c)Kg
#endif
n6){e3=floatBitsToUint(o6.z)&0xffffu;
#ifdef RENDER_MODE_DEPTH_STENCIL
H O4=J0(QB,e3*4u+2u);m7=X1(O4.x);
#else
Kg=ca(floatBitsToInt(o6.z)>>16);
#endif
d p6=o6.xy;f0 U0=h2(uintBitsToFloat(J0(QB,e3*4u)));H M4=J0(QB,e3*4u+1u);d k3=uintBitsToFloat(M4.xy);p6=R0(U0,p6)+k3;return p6;}
#endif
#if defined(VERTEX)&&defined(FEATHER_ATLAS_BLIT)
e d Ib(R o6,Z0(uint)e3,
#ifdef RENDER_MODE_DEPTH_STENCIL
Z0(N)m7,
#endif
Z0(d)Lg n6){e3=floatBitsToUint(o6.z)&0xffffu;H O4=J0(QB,e3*4u+2u);
#ifdef RENDER_MODE_DEPTH_STENCIL
m7=X1(O4.x);
#endif
d p6=o6.xy;R w7=uintBitsToFloat(O4.yzw);Lg=(p6*w7.x+w7.yz)*m.Mg;return p6;}
#endif
e c L8(c a2,c F1,c v2){return(F1-a2)/max(1.-a2*v2,p9);}
#if defined(RENDER_MODE_CLOCKWISE_ATOMIC)||defined(PLS_IMPL_STORAGE_BUFFER)
e uint M8(a1 q6,uint Ng){uint Ka=(q6.y>>i6)*(Ng<<i6)+((q6.x>>i6)<<(i6<<1));Ka+=((q6.x&0x1cu)<<i6)+((q6.y&0x1cu)<<2);Ka+=((q6.y&0x3u)<<2)+(q6.x&0x3u);return Ka;}
#endif
#ifdef RENDER_MODE_CLOCKWISE_ATOMIC
#ifdef FIXED_FUNCTION_COLOR_OUTPUT
#define h5 p2
#define Y3(v5) D1=v5;n3
#else
#define h5 M1
#define Y3(v5) y0(j0,v5);Z1;
#endif
e c La(uint Og){return ca(int((Og&ua)-m5))*sa;}e uint x7(c n){return uint(n*Uf+.5);}
#endif
