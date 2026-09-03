#define i7 -2.
#define Tc -1.5
#define Uc .25
#define C8 1e3
#define Vc (C8*C8)
#ifdef VERTEX
U3 lc(d3,Kf,MC);
#ifdef ENABLE_FEATHER
i6(d3,g7,YC);
#endif
V3 B4 I4(Oc,ig,QB);N5(Jb,Ne,BD);O5(Kb,Oe,RB);I4(Pc,jg,FD);C4
#endif
#if defined(ENABLE_FEATHER)||defined(FEATHER_ATLAS_BLIT)
c4(g7,aa)
#endif
#ifdef FRAGMENT
E3 Z2(d3,Qc,MD);
#if defined(ENABLE_FEATHER)||defined(FEATHER_ATLAS_BLIT)
i6(d3,g7,YC);
#endif
#ifdef FEATHER_ATLAS_BLIT
m5(d3,Rc,CD);
#endif
Z2(c5,W3,JC);
#if defined(RENDER_MODE_MSAA)&&defined(ENABLE_ADVANCED_BLEND)&&!defined(FIXED_FUNCTION_COLOR_OUTPUT)
j7(UD);
#endif
F3 c4(Qc,Pb)
#ifdef FEATHER_ATLAS_BLIT
c4(Rc,Q9)
#endif
d5 X3(V5)e5
#endif
#ifdef FRAGMENT
e bool U5(g P){return P.y>=.0;}e bool U5(E P){return P.y>=.0;}
#endif
#if defined(FRAGMENT)&&defined(ENABLE_FEATHER)
e bool Qb(g P){return P.x<Tc;}e bool Rb(g P){return P.y<Tc;}
#endif
#ifdef VERTEX
g Wc(float va,d D8,float D1){d j6=(1.-D8*abs(D1))*.5;float d4,n5;if(abs(va-W6)<1./C8){d4=.0;n5=.0;}else{float wa=tan(va);d4=sign(W6-va)/max(abs(wa),1./Vc);n5=d4>=.0?j6.y-(1.-j6.x)*wa:j6.y+j6.x*wa;}g P;P.x=max(j6.x,.0)+Uc;P.y=-j6.y+i7;P.z=d4;P.w=n5;return P;}
#endif
#ifdef ENABLE_FEATHER
e c d8(g P I3){c d4=P.z;c n5=max(P.w,.0);c k6=d4>=.0?i5(n5):.0;if(abs(d4)<C8){c x=abs(P.x)-Uc;c y=-P.y+i7;c X2=(y-n5)*0.5984134206;i t=n5+X2*C0(0.20888568955,0.62665706865,1.04442844776,1.46219982687);i u=t*-d4+(y*d4+x);i kg=C0(i5(u[0]),i5(u[1]),i5(u[2]),i5(u[3]));i Xc=t*5.09593080173+-2.54796540086;i lg=exp2(-Xc*Xc);k6+=dot(kg,lg)*X2;}return k6*sign(P.x);}e c y4(g P I3){float k6=1.;float mg=(1.-i7)+P.x;k6-=i5(mg);float ng=1.-P.y;k6-=i5(ng);return k6;}
#endif
#if defined(VERTEX)&&defined(DRAW_PATH)
e Y o5(int Yc){return Y(Yc&((1<<Dc)-1),Yc>>Dc);}e float Zc(f0 U0,d og){d j2=R0(U0,og);return(abs(j2.x)+abs(j2.y))*(1./dot(j2,j2));}e bool q9(g k7,g xa,int v,Z0(uint)e3,Z0(d)pg
#ifndef RENDER_MODE_MSAA
,Z0(g)O1
#else
,Z0(N)l7
#endif
l6){int E8=int(k7.x);float D1=k7.y;float ya=k7.z;int ad=floatBitsToInt(k7.w)>>2;int m7=floatBitsToInt(k7.w)&3;int za=min(E8,ad-1);int J4=v*ad+za;D4 p5=q1(MC,o5(J4));uint i0=h5(p5.w);uint F8=max(i0&Kc,1u);G Aa=J0(FD,F8-1u);d bd=uintBitsToFloat(Aa.xy);e3=Aa.z&0xffffu;uint cd=Aa.w;f0 U0=h2(uintBitsToFloat(J0(QB,e3*4u)));G K4=J0(QB,e3*4u+1u);d k3=uintBitsToFloat(K4.xy);float J2=uintBitsToFloat(K4.z);float K2=uintBitsToFloat(K4.w);uint dd=i0&G3;if(dd!=0u){E8=int(xa.x);D1=xa.y;ya=xa.z;}if(E8!=za){int ed=J4+E8-za;D4 fd=q1(MC,o5(ed));if((h5(fd.w)&(G3|0xffffu))!=(i0&(G3|0xffffu))){bool qg=J2==.0||bd.x!=.0;if(qg){J4=int(cd);p5=q1(MC,o5(J4));}}else{J4=ed;p5=fd;}i0=(h5(p5.w)&~G3)|dd;}float e1;
#ifdef ENABLE_FEATHER
float n7;float v1;if((i0&a4)==y8&&m7==B8){uint gd=h5(p5.z);float e4=float(gd&0xffffu);float k2=float(gd>>16);Y G8=Y(-e4-1.,k2-e4+1.);if((i0&G3)!=0u)G8=-G8;D4 hd=q1(MC,o5(J4+G8.x));D4 Ba=q1(MC,o5(J4+G8.y));if((h5(Ba.w)&(G3|0xffffu))!=(h5(hd.w)&(G3|0xffffu))){Ba=q1(MC,o5(int(cd)));}n7=Y5(hd.z);float id=Y5(Ba.z);v1=id-n7;if(abs(v1)>D3)v1-=p8*sign(v1);float Ca=k2+1.-float(Ec);float jd=clamp(round(abs(v1)/D3*Ca),1.,Ca-1.);float o7=Ca-jd;if(e4<=o7){v1=-(D3*sign(v1)-v1);k2=o7;if(e4==o7)D1=-D1;}else if(e4==o7+1.){e4=.0;k2=.0;D1=.0;}else{e4-=o7+2.;k2=jd;}if(e4==k2){e1=id;}else{e1=n7+v1*(e4/k2);}}else
#endif
{e1=Y5(p5.z);}d Y2=d(sin(e1),-cos(e1));d kd=Y5(p5.xy);d H8=d(0,0);if(K2!=.0){K2=max(K2,(na/3.)/length(R0(U0,Y2)));}if(J2!=.0){D1*=sign(determinant(U0));if((i0&A8)!=0u)D1=min(D1,.0);if((i0&Jc)!=0u)D1=max(D1,.0);float L4=K2!=.0?K2:Zc(U0,Y2)*p4;c ld=1.;if(L4>J2&&K2==.0){ld=V4(J2)/V4(L4);J2=L4;}d q5=Y2*(J2+L4);
#ifndef RENDER_MODE_MSAA
float x=D1*(J2+L4);O1.xy=(1./(L4*2.))*(d(x,-x)+J2)+.5;O1.zw=M6(.0);
#endif
uint Da=i0&a4;if(Da>x8){int p7=2;if((i0&oa)==0u)p7=-p7;if((i0&G3)!=0u)p7=-p7;Y rg=o5(J4+p7);D4 sg=q1(MC,rg);float tg=Y5(sg.z);float q7=abs(tg-e1);if(q7>D3)q7=p8-q7;bool I8=(i0&oa)!=0u;bool ug=(i0&A8)!=0u;float md=q7*(I8==ug?-.5:.5)+e1;d J8=d(sin(md),-cos(md));float Ea=Zc(U0,J8);float r7=cos(q7*.5);float Fa;if((Da==Ef)||(Da==Ff&&r7>=.25)){float vg=(i0&z8)!=0u?1.:.25;Fa=J2*(1./max(r7,vg));}else{Fa=J2*r7+Ea*.5;}float Ga=Fa+Ea*p4;if((i0&Ic)!=0u){float nd=J2+L4;float wg=L4*.125;if(nd<=Ga*r7+wg){float xg=nd*(1./r7);q5=J8*xg;}else{d Ha=J8*Ga;d yg=d(dot(q5,q5),dot(Ha,Ha));q5=R0(yg,inverse(f0(q5,Ha)));}}d zg=abs(D1)*q5;float od=(Ga-dot(zg,J8))/(Ea*(p4*2.));
#ifndef RENDER_MODE_MSAA
if((i0&A8)!=0u)O1.y=od;else O1.x=od;
#endif
}
#ifndef RENDER_MODE_MSAA
O1.xy*=ld;O1.y=max(O1.y,1e-4);if(K2!=.0){O1.x=i7-O1.x;}
#endif
H8=R0(U0,D1*q5);if(m7!=B8)return false;}else{
#ifndef RENDER_MODE_MSAA
O1=g(ya,-1.,.0,.0);
#ifdef ENABLE_FEATHER
if(K2!=.0){O1.y=i7;O1.z=Vc;O1.w=ya;if((i0&a4)==y8&&m7==B8){if(v1<.0){n7+=v1;v1=-v1;}float f4=e1-n7;f4=mod(f4+W6,p8)-W6;f4=clamp(f4,.0,v1);if(f4>v1*.5){f4=v1-f4;}d D8=d(sin(f4),cos(f4));
#if 0
float P1=1.+.33*log2(W6/(D3-min(v1,D3-D3/16.)));g Ag=Wc(v1,D8,.5*(P1/3.));float Bg=d8(Ag d1);float Cg=oc(Bg);float Dg=(.5-Cg)*(na*2.);float Eg=P1/max(Dg,P1);D1*=Eg;
#endif
O1=Wc(v1,D8,D1);}H8=R0(U0,(D1*K2)*Y2);}else
#endif
{H8=sign(R0(D1*Y2,inverse(U0)))*p4;}if(bool(i0&G3)!=bool(i0&Gf)){O1*=g(-1.,+1.,+1.,+1.);}
#endif
if(m7==Mc)kd=bd;if((i0&Hc)!=0u&&m7!=Lc){return false;}}pg=R0(U0,kd)+H8+k3;
#ifdef RENDER_MODE_MSAA
G M4=J0(QB,e3*4u+2u);l7=X1(M4.x);
#else
O1.xy=mix(O1.xy,d(1.,-1.),of(m.Fg!=0u));
#endif
return true;}
#endif
#if defined(VERTEX)&&defined(DRAW_INTERIOR_TRIANGLES)
e d Hb(R m6,Z0(uint)e3
#ifdef RENDER_MODE_MSAA
,Z0(N)l7
#else
,Z0(c)Gg
#endif
l6){e3=floatBitsToUint(m6.z)&0xffffu;
#ifdef RENDER_MODE_MSAA
G M4=J0(QB,e3*4u+2u);l7=X1(M4.x);
#else
Gg=ba(floatBitsToInt(m6.z)>>16);
#endif
d n6=m6.xy;f0 U0=h2(uintBitsToFloat(J0(QB,e3*4u)));G K4=J0(QB,e3*4u+1u);d k3=uintBitsToFloat(K4.xy);n6=R0(U0,n6)+k3;return n6;}
#endif
#if defined(VERTEX)&&defined(FEATHER_ATLAS_BLIT)
e d Gb(R m6,Z0(uint)e3,
#ifdef RENDER_MODE_MSAA
Z0(N)l7,
#endif
Z0(d)Hg l6){e3=floatBitsToUint(m6.z)&0xffffu;G M4=J0(QB,e3*4u+2u);
#ifdef RENDER_MODE_MSAA
l7=X1(M4.x);
#endif
d n6=m6.xy;R v7=uintBitsToFloat(M4.yzw);Hg=(n6*v7.x+v7.yz)*m.Ig;return n6;}
#endif
e c K8(c a2,c E1,c v2){return(E1-a2)/max(1.-a2*v2,o9);}e uint L8(a1 o6,uint Jg){uint Ia=(o6.y>>h6)*(Jg<<h6)+((o6.x>>h6)<<(h6<<1));Ia+=((o6.x&0x1cu)<<h6)+((o6.y&0x1cu)<<2);Ia+=((o6.y&0x3u)<<2)+(o6.x&0x3u);return Ia;}
#ifdef RENDER_MODE_CLOCKWISE_ATOMIC
#ifdef FIXED_FUNCTION_COLOR_OUTPUT
#define f5 p2
#define Y3(r5) C1=r5;n3
#else
#define f5 L1
#define Y3(r5) y0(j0,r5);Z1;
#endif
e c Ja(uint Kg){return ba(int((Kg&sa)-l5))*qa;}e uint w7(c n){return uint(n*Qf+.5);}
#endif
