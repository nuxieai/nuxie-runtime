#define j7 -2.
#define cd -1.5
#define dd .25
#define D8 1e3
#define ed (D8*D8)
#ifdef DB
U3 uc(e3,dg,KC);
#ifdef HB
j6(e3,h7,YC);
#endif
V3 B4 J4(Wc,Bg,PB);N5(Qb,hf,DD);O5(Rb,jf,QB);J4(Xc,Cg,ID);C4
#endif
#if defined(HB)||defined(GB)
c4(h7,ea)
#endif
#ifdef FB
E3 c3(e3,Yc,ED);
#if defined(HB)||defined(GB)
j6(e3,h7,YC);
#endif
#ifdef GB
m5(e3,Zc,FD);
#endif
c3(d5,W3,HC);
#if defined(CB)&&defined(AB)&&!defined(O)
k6(YD);
#endif
F3 c4(Yc,N9)
#ifdef GB
c4(Zc,R9)
#endif
e5 X3(W5)f5
#endif
#ifdef FB
e bool V5(f N){return N.y>=.0;}e bool V5(E N){return N.y>=.0;}
#endif
#if defined(FB)&&defined(HB)
e bool Yb(f N){return N.x<cd;}e bool Zb(f N){return N.y<cd;}
#endif
#ifdef DB
f fd(float Ba,c E8,float D1){c l6=(1.-E8*abs(D1))*.5;float d4,n5;if(abs(Ba-X6)<1./D8){d4=.0;n5=.0;}else{float Ca=tan(Ba);d4=sign(X6-Ba)/max(abs(Ca),1./ed);n5=d4>=.0?l6.y-(1.-l6.x)*Ca:l6.y+l6.x*Ca;}f N;N.x=max(l6.x,.0)+dd;N.y=-l6.y+j7;N.z=d4;N.w=n5;return N;}
#endif
#ifdef HB
e d e8(f N I3){d d4=N.z;d n5=max(N.w,.0);d m6=d4>=.0?j5(n5):.0;if(abs(d4)<D8){d x=abs(N.x)-dd;d y=-N.y+j7;d Z2=(y-n5)*0.5984134206;i t=n5+Z2*D0(0.20888568955,0.62665706865,1.04442844776,1.46219982687);i u=t*-d4+(y*d4+x);i Dg=D0(j5(u[0]),j5(u[1]),j5(u[2]),j5(u[3]));i gd=t*5.09593080173+-2.54796540086;i Eg=exp2(-gd*gd);m6+=dot(Dg,Eg)*Z2;}return m6*sign(N.x);}e d y4(f N I3){float m6=1.;float Fg=(1.-j7)+N.x;m6-=j5(Fg);float Gg=1.-N.y;m6-=j5(Gg);return m6;}
#endif
#if defined(DB)&&defined(OD)
e Y o5(int hd){return Y(hd&((1<<Lc)-1),hd>>Lc);}e float id(d0 U0,c Hg){c l2=N0(U0,Hg);return(abs(l2.x)+abs(l2.y))*(1./dot(l2,l2));}e bool v9(f k7,f Da,int v,Z0(uint)f3,Z0(c)Ig
#ifndef CB
,Z0(f)P1
#else
,Z0(L)l7
#endif
n6){int F8=int(k7.x);float D1=k7.y;float Ea=k7.z;int jd=floatBitsToInt(k7.w)>>2;int m7=floatBitsToInt(k7.w)&3;int Fa=min(F8,jd-1);int K4=v*jd+Fa;D4 p5=p1(KC,o5(K4));uint i0=i5(p5.w);uint G8=max(i0&Sc,1u);X Ga=K0(ID,G8-1u);c kd=uintBitsToFloat(Ga.xy);f3=Ga.z&0xffffu;uint ld=Ga.w;d0 U0=I1(uintBitsToFloat(K0(PB,f3*4u)));X L4=K0(PB,f3*4u+1u);c H2=uintBitsToFloat(L4.xy);float L2=uintBitsToFloat(L4.z);float M2=uintBitsToFloat(L4.w);uint md=i0&G3;if(md!=0u){F8=int(Da.x);D1=Da.y;Ea=Da.z;}if(F8!=Fa){int nd=K4+F8-Fa;D4 od=p1(KC,o5(nd));if((i5(od.w)&(G3|0xffffu))!=(i0&(G3|0xffffu))){bool Jg=L2==.0||kd.x!=.0;if(Jg){K4=int(ld);p5=p1(KC,o5(K4));}}else{K4=nd;p5=od;}i0=(i5(p5.w)&~G3)|md;}float e1;
#ifdef HB
float n7;float r1;if((i0&a4)==z8&&m7==C8){uint pd=i5(p5.z);float e4=float(pd&0xffffu);float m2=float(pd>>16);Y H8=Y(-e4-1.,m2-e4+1.);if((i0&G3)!=0u)H8=-H8;D4 qd=p1(KC,o5(K4+H8.x));D4 Ha=p1(KC,o5(K4+H8.y));if((i5(Ha.w)&(G3|0xffffu))!=(i5(qd.w)&(G3|0xffffu))){Ha=p1(KC,o5(int(ld)));}n7=Z5(qd.z);float rd=Z5(Ha.z);r1=rd-n7;if(abs(r1)>D3)r1-=q8*sign(r1);float Ia=m2+1.-float(Mc);float sd=clamp(round(abs(r1)/D3*Ia),1.,Ia-1.);float o7=Ia-sd;if(e4<=o7){r1=-(D3*sign(r1)-r1);m2=o7;if(e4==o7)D1=-D1;}else if(e4==o7+1.){e4=.0;m2=.0;D1=.0;}else{e4-=o7+2.;m2=sd;}if(e4==m2){e1=rd;}else{e1=n7+r1*(e4/m2);}}else
#endif
{e1=Z5(p5.z);}c a3=c(sin(e1),-cos(e1));c td=Z5(p5.xy);c I8=c(0,0);if(M2!=.0){M2=max(M2,(sa/3.)/length(N0(U0,a3)));}if(L2!=.0){D1*=sign(determinant(U0));if((i0&B8)!=0u)D1=min(D1,.0);if((i0&Rc)!=0u)D1=max(D1,.0);float M4=M2!=.0?M2:id(U0,a3)*r4;d ud=1.;if(M4>L2&&M2==.0){ud=W4(L2)/W4(M4);L2=M4;}c q5=a3*(L2+M4);
#ifndef CB
float x=D1*(L2+M4);P1.xy=(1./(M4*2.))*(c(x,-x)+L2)+.5;P1.zw=N6(.0);
#endif
uint Ja=i0&a4;if(Ja>y8){int p7=2;if((i0&ta)==0u)p7=-p7;if((i0&G3)!=0u)p7=-p7;Y Kg=o5(K4+p7);D4 Lg=p1(KC,Kg);float Mg=Z5(Lg.z);float q7=abs(Mg-e1);if(q7>D3)q7=q8-q7;bool J8=(i0&ta)!=0u;bool Ng=(i0&B8)!=0u;float vd=q7*(J8==Ng?-.5:.5)+e1;c K8=c(sin(vd),-cos(vd));float Ka=id(U0,K8);float r7=cos(q7*.5);float La;if((Ja==Yf)||(Ja==Zf&&r7>=.25)){float Og=(i0&A8)!=0u?1.:.25;La=L2*(1./max(r7,Og));}else{La=L2*r7+Ka*.5;}float Ma=La+Ka*r4;if((i0&Qc)!=0u){float wd=L2+M4;float Pg=M4*.125;if(wd<=Ma*r7+Pg){float Qg=wd*(1./r7);q5=K8*Qg;}else{c Na=K8*Ma;c Rg=c(dot(q5,q5),dot(Na,Na));q5=N0(Rg,inverse(d0(q5,Na)));}}c Sg=abs(D1)*q5;float xd=(Ma-dot(Sg,K8))/(Ka*(r4*2.));
#ifndef CB
if((i0&B8)!=0u)P1.y=xd;else P1.x=xd;
#endif
}
#ifndef CB
P1.xy*=ud;P1.y=max(P1.y,1e-4);if(M2!=.0){P1.x=j7-P1.x;}
#endif
I8=N0(U0,D1*q5);if(m7!=C8)return false;}else{
#ifndef CB
P1=f(Ea,-1.,.0,.0);
#ifdef HB
if(M2!=.0){P1.y=j7;P1.z=ed;P1.w=Ea;if((i0&a4)==z8&&m7==C8){if(r1<.0){n7+=r1;r1=-r1;}float f4=e1-n7;f4=mod(f4+X6,q8)-X6;f4=clamp(f4,.0,r1);if(f4>r1*.5){f4=r1-f4;}c E8=c(sin(f4),cos(f4));
#if 0
float Q1=1.+.33*log2(X6/(D3-min(r1,D3-D3/16.)));f Tg=fd(r1,E8,.5*(Q1/3.));float Ug=e8(Tg d1);float Vg=xc(Ug);float Wg=(.5-Vg)*(sa*2.);float Xg=Q1/max(Wg,Q1);D1*=Xg;
#endif
P1=fd(r1,E8,D1);}I8=N0(U0,(D1*M2)*a3);}else
#endif
{I8=sign(N0(D1*a3,inverse(U0)))*r4;}if(bool(i0&G3)!=bool(i0&ag)){P1*=f(-1.,+1.,+1.,+1.);}
#endif
if(m7==Uc)td=kd;if((i0&Pc)!=0u&&m7!=Tc){return false;}}Ig=N0(U0,td)+I8+H2;
#ifdef CB
X N4=K0(PB,f3*4u+2u);l7=Y1(N4.x);
#else
P1.xy=mix(P1.xy,c(1.,-1.),If(n.Yg!=0u));
#endif
return true;}
#endif
#if defined(DB)&&defined(EB)
e c Lb(Q o6,Z0(uint)f3
#ifdef CB
,Z0(L)l7
#else
,Z0(d)Zg
#endif
n6){f3=floatBitsToUint(o6.z)&0xffffu;
#ifdef CB
X N4=K0(PB,f3*4u+2u);l7=Y1(N4.x);
#else
Zg=fa(floatBitsToInt(o6.z)>>16);
#endif
c p6=o6.xy;d0 U0=I1(uintBitsToFloat(K0(PB,f3*4u)));X L4=K0(PB,f3*4u+1u);c H2=uintBitsToFloat(L4.xy);p6=N0(U0,p6)+H2;return p6;}
#endif
#if defined(DB)&&defined(GB)
e c Kb(Q o6,Z0(uint)f3,
#ifdef CB
Z0(L)l7,
#endif
Z0(c)ah n6){f3=floatBitsToUint(o6.z)&0xffffu;X N4=K0(PB,f3*4u+2u);
#ifdef CB
l7=Y1(N4.x);
#endif
c p6=o6.xy;Q v7=uintBitsToFloat(N4.yzw);ah=(p6*v7.x+v7.yz)*n.bh;return p6;}
#endif
e d L8(d c2,d E1,d x2){return(E1-c2)/max(1.-c2*x2,q9);}
#if defined(RB)||defined(JD)
e uint M8(a1 g4,uint ch){uint Oa=(g4.y>>i6)*(ch<<i6)+((g4.x>>i6)<<(i6<<1));Oa+=((g4.x&0x1cu)<<i6)+((g4.y&0x1cu)<<2);Oa+=((g4.y&0x3u)<<2)+(g4.x&0x3u);return Oa;}
#endif
#ifdef RB
#ifdef O
#define g5 r2
#define Y3(r5) C1=r5;m3
#else
#define g5 M1
#define Y3(r5) z0(k0,r5);a2;
#endif
e d Pa(uint dh){return fa(int((dh&ya)-l5))*wa;}e uint w7(d o){return uint(o*jg+.5);}
#endif
