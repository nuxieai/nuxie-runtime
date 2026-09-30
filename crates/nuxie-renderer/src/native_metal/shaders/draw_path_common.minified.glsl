#define k7 -2.
#define ed -1.5
#define fd .25
#define D8 1e3
#define gd (D8*D8)
#ifdef VERTEX
V3 wc(d3,fg,KC);
#ifdef ENABLE_FEATHER
i6(d3,i7,YC);
#endif
W3 B4 J4(Yc,Dg,PB);M5(Sb,kf,DD);N5(Tb,lf,QB);J4(Zc,Eg,ID);C4
#endif
#if defined(ENABLE_FEATHER)||defined(FEATHER_ATLAS_BLIT)
d4(i7,fa)
#endif
#ifdef FRAGMENT
F3 a3(d3,ad,ED);
#if defined(ENABLE_FEATHER)||defined(FEATHER_ATLAS_BLIT)
i6(d3,i7,YC);
#endif
#ifdef FEATHER_ATLAS_BLIT
l5(d3,bd,FD);
#endif
a3(c5,X3,HC);
#if defined(RENDER_MODE_DEPTH_STENCIL)&&defined(ENABLE_ADVANCED_BLEND)&&!defined(FIXED_FUNCTION_COLOR_OUTPUT)
m5(YD);
#endif
G3 d4(ad,O9)
#ifdef FEATHER_ATLAS_BLIT
d4(bd,S9)
#endif
d5 Y3(V5)e5
#endif
#ifdef FRAGMENT
e bool U5(f N){return N.y>=.0;}e bool U5(E N){return N.y>=.0;}
#endif
#if defined(FRAGMENT)&&defined(ENABLE_FEATHER)
e bool ac(f N){return N.x<ed;}e bool bc(f N){return N.y<ed;}
#endif
#ifdef VERTEX
f hd(float Ca,c E8,float D1){c j6=(1.-E8*abs(D1))*.5;float e4,n5;if(abs(Ca-Y6)<1./D8){e4=.0;n5=.0;}else{float Da=tan(Ca);e4=sign(Y6-Ca)/max(abs(Da),1./gd);n5=e4>=.0?j6.y-(1.-j6.x)*Da:j6.y+j6.x*Da;}f N;N.x=max(j6.x,.0)+fd;N.y=-j6.y+k7;N.z=e4;N.w=n5;return N;}
#endif
#ifdef ENABLE_FEATHER
e d e8(f N I3){d e4=N.z;d n5=max(N.w,.0);d k6=e4>=.0?i5(n5):.0;if(abs(e4)<D8){d x=abs(N.x)-fd;d y=-N.y+k7;d Y2=(y-n5)*0.5984134206;i t=n5+Y2*D0(0.20888568955,0.62665706865,1.04442844776,1.46219982687);i u=t*-e4+(y*e4+x);i Fg=D0(i5(u[0]),i5(u[1]),i5(u[2]),i5(u[3]));i id=t*5.09593080173+-2.54796540086;i Gg=exp2(-id*id);k6+=dot(Fg,Gg)*Y2;}return k6*sign(N.x);}e d y4(f N I3){float k6=1.;float Hg=(1.-k7)+N.x;k6-=i5(Hg);float Ig=1.-N.y;k6-=i5(Ig);return k6;}
#endif
#if defined(VERTEX)&&defined(DRAW_PATH)
e Y o5(int jd){return Y(jd&((1<<Nc)-1),jd>>Nc);}e float kd(d0 U0,c Jg){c l2=N0(U0,Jg);return(abs(l2.x)+abs(l2.y))*(1./dot(l2,l2));}e bool w9(f l7,f Ea,int v,Z0(uint)e3,Z0(c)Kg
#ifndef RENDER_MODE_DEPTH_STENCIL
,Z0(f)P1
#else
,Z0(L)m7
#endif
l6){int F8=int(l7.x);float D1=l7.y;float Fa=l7.z;int ld=floatBitsToInt(l7.w)>>2;int n7=floatBitsToInt(l7.w)&3;int Ga=min(F8,ld-1);int K4=v*ld+Ga;D4 p5=p1(KC,o5(K4));uint i0=h5(p5.w);uint G8=max(i0&Uc,1u);X Ha=K0(ID,G8-1u);c md=uintBitsToFloat(Ha.xy);e3=Ha.z&0xffffu;uint nd=Ha.w;d0 U0=I1(uintBitsToFloat(K0(PB,e3*4u)));X L4=K0(PB,e3*4u+1u);c H2=uintBitsToFloat(L4.xy);float L2=uintBitsToFloat(L4.z);float M2=uintBitsToFloat(L4.w);uint od=i0&H3;if(od!=0u){F8=int(Ea.x);D1=Ea.y;Fa=Ea.z;}if(F8!=Ga){int pd=K4+F8-Ga;D4 qd=p1(KC,o5(pd));if((h5(qd.w)&(H3|0xffffu))!=(i0&(H3|0xffffu))){bool Lg=L2==.0||md.x!=.0;if(Lg){K4=int(nd);p5=p1(KC,o5(K4));}}else{K4=pd;p5=qd;}i0=(h5(p5.w)&~H3)|od;}float e1;
#ifdef ENABLE_FEATHER
float o7;float r1;if((i0&c4)==z8&&n7==C8){uint rd=h5(p5.z);float f4=float(rd&0xffffu);float m2=float(rd>>16);Y H8=Y(-f4-1.,m2-f4+1.);if((i0&H3)!=0u)H8=-H8;D4 sd=p1(KC,o5(K4+H8.x));D4 Ia=p1(KC,o5(K4+H8.y));if((h5(Ia.w)&(H3|0xffffu))!=(h5(sd.w)&(H3|0xffffu))){Ia=p1(KC,o5(int(nd)));}o7=Y5(sd.z);float td=Y5(Ia.z);r1=td-o7;if(abs(r1)>E3)r1-=q8*sign(r1);float Ja=m2+1.-float(Oc);float ud=clamp(round(abs(r1)/E3*Ja),1.,Ja-1.);float p7=Ja-ud;if(f4<=p7){r1=-(E3*sign(r1)-r1);m2=p7;if(f4==p7)D1=-D1;}else if(f4==p7+1.){f4=.0;m2=.0;D1=.0;}else{f4-=p7+2.;m2=ud;}if(f4==m2){e1=td;}else{e1=o7+r1*(f4/m2);}}else
#endif
{e1=Y5(p5.z);}c Z2=c(sin(e1),-cos(e1));c vd=Y5(p5.xy);c I8=c(0,0);if(M2!=.0){M2=max(M2,(ta/3.)/length(N0(U0,Z2)));}if(L2!=.0){D1*=sign(determinant(U0));if((i0&B8)!=0u)D1=min(D1,.0);if((i0&Tc)!=0u)D1=max(D1,.0);float M4=M2!=.0?M2:kd(U0,Z2)*r4;d wd=1.;if(M4>L2&&M2==.0){wd=W4(L2)/W4(M4);L2=M4;}c q5=Z2*(L2+M4);
#ifndef RENDER_MODE_DEPTH_STENCIL
float x=D1*(L2+M4);P1.xy=(1./(M4*2.))*(c(x,-x)+L2)+.5;P1.zw=O6(.0);
#endif
uint Ka=i0&c4;if(Ka>y8){int q7=2;if((i0&ua)==0u)q7=-q7;if((i0&H3)!=0u)q7=-q7;Y Mg=o5(K4+q7);D4 Ng=p1(KC,Mg);float Og=Y5(Ng.z);float r7=abs(Og-e1);if(r7>E3)r7=q8-r7;bool J8=(i0&ua)!=0u;bool Pg=(i0&B8)!=0u;float xd=r7*(J8==Pg?-.5:.5)+e1;c K8=c(sin(xd),-cos(xd));float La=kd(U0,K8);float v7=cos(r7*.5);float Ma;if((Ka==ag)||(Ka==bg&&v7>=.25)){float Qg=(i0&A8)!=0u?1.:.25;Ma=L2*(1./max(v7,Qg));}else{Ma=L2*v7+La*.5;}float Na=Ma+La*r4;if((i0&Sc)!=0u){float yd=L2+M4;float Rg=M4*.125;if(yd<=Na*v7+Rg){float Sg=yd*(1./v7);q5=K8*Sg;}else{c Oa=K8*Na;c Tg=c(dot(q5,q5),dot(Oa,Oa));q5=N0(Tg,inverse(d0(q5,Oa)));}}c Ug=abs(D1)*q5;float zd=(Na-dot(Ug,K8))/(La*(r4*2.));
#ifndef RENDER_MODE_DEPTH_STENCIL
if((i0&B8)!=0u)P1.y=zd;else P1.x=zd;
#endif
}
#ifndef RENDER_MODE_DEPTH_STENCIL
P1.xy*=wd;P1.y=max(P1.y,1e-4);if(M2!=.0){P1.x=k7-P1.x;}
#endif
I8=N0(U0,D1*q5);if(n7!=C8)return false;}else{
#ifndef RENDER_MODE_DEPTH_STENCIL
P1=f(Fa,-1.,.0,.0);
#ifdef ENABLE_FEATHER
if(M2!=.0){P1.y=k7;P1.z=gd;P1.w=Fa;if((i0&c4)==z8&&n7==C8){if(r1<.0){o7+=r1;r1=-r1;}float g4=e1-o7;g4=mod(g4+Y6,q8)-Y6;g4=clamp(g4,.0,r1);if(g4>r1*.5){g4=r1-g4;}c E8=c(sin(g4),cos(g4));
#if 0
float Q1=1.+.33*log2(Y6/(E3-min(r1,E3-E3/16.)));f Vg=hd(r1,E8,.5*(Q1/3.));float Wg=e8(Vg d1);float Xg=zc(Wg);float Yg=(.5-Xg)*(ta*2.);float Zg=Q1/max(Yg,Q1);D1*=Zg;
#endif
P1=hd(r1,E8,D1);}I8=N0(U0,(D1*M2)*Z2);}else
#endif
{I8=sign(N0(D1*Z2,inverse(U0)))*r4;}if(bool(i0&H3)!=bool(i0&cg)){P1*=f(-1.,+1.,+1.,+1.);}
#endif
if(n7==Wc)vd=md;if((i0&Rc)!=0u&&n7!=Vc){return false;}}Kg=N0(U0,vd)+I8+H2;
#ifdef RENDER_MODE_DEPTH_STENCIL
X N4=K0(PB,e3*4u+2u);m7=Y1(N4.x);
#else
P1.xy=mix(P1.xy,c(1.,-1.),Kf(l.ah!=0u));
#endif
return true;}
#endif
#if defined(VERTEX)&&defined(DRAW_INTERIOR_TRIANGLES)
e c Mb(Q m6,Z0(uint)e3
#ifdef RENDER_MODE_DEPTH_STENCIL
,Z0(L)m7
#else
,Z0(d)bh
#endif
l6){e3=floatBitsToUint(m6.z)&0xffffu;
#ifdef RENDER_MODE_DEPTH_STENCIL
X N4=K0(PB,e3*4u+2u);m7=Y1(N4.x);
#else
bh=ga(floatBitsToInt(m6.z)>>16);
#endif
c n6=m6.xy;d0 U0=I1(uintBitsToFloat(K0(PB,e3*4u)));X L4=K0(PB,e3*4u+1u);c H2=uintBitsToFloat(L4.xy);n6=N0(U0,n6)+H2;return n6;}
#endif
#if defined(VERTEX)&&defined(FEATHER_ATLAS_BLIT)
e c Lb(Q m6,Z0(uint)e3,
#ifdef RENDER_MODE_DEPTH_STENCIL
Z0(L)m7,
#endif
Z0(c)ch l6){e3=floatBitsToUint(m6.z)&0xffffu;X N4=K0(PB,e3*4u+2u);
#ifdef RENDER_MODE_DEPTH_STENCIL
m7=Y1(N4.x);
#endif
c n6=m6.xy;Q w7=uintBitsToFloat(N4.yzw);ch=(n6*w7.x+w7.yz)*l.dh;return n6;}
#endif
e d L8(d c2,d E1,d f3){return(E1-c2)/max(1.-c2*f3,r9);}
#if defined(RENDER_MODE_CLOCKWISE_ATOMIC)||defined(PLS_IMPL_STORAGE_BUFFER)
e uint M8(a1 h4,uint eh){uint Pa=(h4.y>>h6)*(eh<<h6)+((h4.x>>h6)<<(h6<<1));Pa+=((h4.x&0x1cu)<<h6)+((h4.y&0x1cu)<<2);Pa+=((h4.y&0x3u)<<2)+(h4.x&0x3u);return Pa;}
#endif
#ifdef RENDER_MODE_CLOCKWISE_ATOMIC
#ifdef FIXED_FUNCTION_COLOR_OUTPUT
#define f5 r2
#define Z3(r5) C1=r5;m3
#else
#define f5 M1
#define Z3(r5) z0(k0,r5);a2;
#endif
e d Qa(uint fh){return ga(int((fh&za)-k5))*xa;}e uint x7(d o){return uint(o*lg+.5);}
#endif
