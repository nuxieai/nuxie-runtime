#define j7 -2.
#define ed -1.5
#define fd .25
#define C8 1e3
#define gd (C8*C8)
#ifdef CB
Y3 L4(h3,jg,KC);
#ifdef GB
h6(h3,h7,YC);
#endif
Z3 F4 M4(Zc,Ig,OB);O5(Rb,kf,DD);P5(Sb,lf,PB);M4(ad,Jg,ID);G4
#endif
#if defined(GB)||defined(FB)
g4(h7,da)
#endif
#ifdef EB
I3 e3(h3,bd,ED);
#if defined(GB)||defined(FB)
h6(h3,h7,YC);
#endif
#ifdef FB
o5(h3,cd,FD);
#endif
e3(h5,a4,HC);
#if defined(BB)&&defined(T)&&!defined(Q)
p5(YD);
#endif
J3 g4(bd,N9)
#ifdef FB
g4(cd,S9)
#endif
i5 c4(W5) j5
#endif
#ifdef EB
e bool V5(f P){return P.y>=.0;}e bool V5(D P){return P.y>=.0;}
#endif
#if defined(EB)&&defined(GB)
e bool cc(f P){return P.x<ed;}e bool dc(f P){return P.y<ed;}
#endif
#ifdef CB
f hd(float Aa,c D8,float G1){c i6=(1.-D8*abs(G1))*.5;float h4,q5;if(abs(Aa-X6)<1./C8){h4=.0;q5=.0;}else{float Ba=tan(Aa);h4=sign(X6-Aa)/max(abs(Ba),1./gd);q5=h4>=.0?i6.y-(1.-i6.x)*Ba:i6.y+i6.x*Ba;}f P;P.x=max(i6.x,.0)+fd;P.y=-i6.y+j7;P.z=h4;P.w=q5;return P;}
#endif
#ifdef GB
e d d8(f P L3){d h4=P.z;d q5=max(P.w,.0);d j6=h4>=.0?l5(q5):.0;if(abs(h4)<C8){d x=abs(P.x)-fd;d y=-P.y+j7;d c3=(y-q5)*0.5984134206;i t=q5+c3*E0(0.20888568955,0.62665706865,1.04442844776,1.46219982687);i u=t*-h4+(y*h4+x);i Kg=E0(l5(u[0]),l5(u[1]),l5(u[2]),l5(u[3]));i id=t*5.09593080173+-2.54796540086;i Lg=exp2(-id*id);j6+=dot(Kg,Lg)*c3;}return j6*sign(P.x);}e d C4(f P L3){float j6=1.;float Mg=(1.-j7)+P.x;j6-=l5(Mg);float Ng=1.-P.y;j6-=l5(Ng);return j6;}
#endif
#if defined(CB)&&defined(OD)
e Y r5(int jd){return Y(jd&((1<<Oc)-1),jd>>Oc);}e float kd(e0 W0,c Og){c m2=P0(W0,Og);return(abs(m2.x)+abs(m2.y))*(1./dot(m2,m2));}e bool r9(f k7,f Ca,int q,c1(uint) i3,c1(c) Pg
#ifndef BB
,c1(f) S1
#else
,c1(N) l7
#endif
k6){int E8=int(k7.x);float G1=k7.y;float Da=k7.z;int ld=floatBitsToInt(k7.w)>>2;int m7=floatBitsToInt(k7.w)&3;int Ea=min(E8,ld-1);int N4=q*ld+Ea;R v5=v1(KC,r5(N4));uint j0=v5.w;uint F8=max(j0&Vc,1u);R Fa=L0(ID,F8-1u);c md=uintBitsToFloat(Fa.xy);i3=Fa.z&0xffffu;uint nd=Fa.w;e0 W0=L1(uintBitsToFloat(L0(OB,i3*4u)));R O4=L0(OB,i3*4u+1u);c I2=uintBitsToFloat(O4.xy);float N2=uintBitsToFloat(O4.z);float O2=uintBitsToFloat(O4.w);uint od=j0&K3;if(od!=0u){E8=int(Ca.x);G1=Ca.y;Da=Ca.z;}if(E8!=Ea){int pd=N4+E8-Ea;R qd=v1(KC,r5(pd));if((qd.w&(K3|0xffffu))!=(j0&(K3|0xffffu))){bool Qg=N2==.0||md.x!=.0;if(Qg){N4=int(nd);v5=v1(KC,r5(N4));}}else{N4=pd;v5=qd;}j0=(v5.w&~K3)|od;}bool Ga=false;float f1;
#ifdef GB
float n7;float x1;if((j0&f4)==y8&&m7==B8){uint rd=v5.z;float i4=float(rd&0xffffu);float n2=float(rd>>16);Y G8=Y(-i4-1.,n2-i4+1.);if((j0&K3)!=0u) G8=-G8;R sd=v1(KC,r5(N4+G8.x));R Ha=v1(KC,r5(N4+G8.y));if((Ha.w&(K3|0xffffu))!=(sd.w&(K3|0xffffu))){Ha=v1(KC,r5(int(nd)));}n7=uintBitsToFloat(sd.z);float td=uintBitsToFloat(Ha.z);x1=td-n7;if(abs(x1)>H3) x1-=p8*sign(x1);float Ia=n2+1.-float(Pc);float ud=clamp(round(abs(x1)/H3*Ia),1.,Ia-1.);float o7=Ia-ud;if(i4<=o7){x1=-(H3*sign(x1)-x1);n2=o7;if(i4==o7) G1=-G1;}else if(i4==o7+1.){i4=.0;n2=.0;G1=.0;}else{i4-=o7+2.;n2=ud;}if(i4==n2){f1=td;}else{f1=n7+x1*(i4/n2);}}else
#endif
{f1=uintBitsToFloat(v5.z);}c d3=c(sin(f1),-cos(f1));c vd=uintBitsToFloat(v5.xy);c H8=c(0,0);if(O2!=.0){O2=max(O2,(ra/3.)/length(P0(W0,d3)));}if(N2!=.0){G1*=sign(determinant(W0));if((j0&A8)!=0u) G1=min(G1,.0);if((j0&Uc)!=0u) G1=max(G1,.0);float P4=O2!=.0?O2:kd(W0,d3)*x4;d wd=1.;if(P4>N2&&O2==.0){wd=S3(N2)/S3(P4);N2=P4;}c w5=d3*(N2+P4);
#ifndef BB
float x=G1*(N2+P4);S1.xy=(1./(P4*2.))*(c(x,-x)+N2)+.5;S1.zw=N6(.0);
#endif
uint Ja=j0&f4;if(Ja>x8){int p7=2;if((j0&sa)==0u) p7=-p7;if((j0&K3)!=0u) p7=-p7;Y Rg=r5(N4+p7);R Sg=v1(KC,Rg);float Tg=uintBitsToFloat(Sg.z);float q7=abs(Tg-f1);if(q7>H3) q7=p8-q7;bool I8=(j0&sa)!=0u;bool Ug=(j0&A8)!=0u;float xd=q7*(I8==Ug?-.5:.5)+f1;c J8=c(sin(xd),-cos(xd));float Ka=kd(W0,J8);float r7=cos(q7*.5);float La;if((Ja==eg)||(Ja==fg&&r7>=.25)){float Vg=(j0&z8)!=0u?1.:.25;La=N2*(1./max(r7,Vg));}else{La=N2*r7+Ka*.5;}float Ma=La+Ka*x4;if((j0&Tc)!=0u){float yd=N2+P4;float Wg=P4*.125;if(yd<=Ma*r7+Wg){float Xg=yd*(1./r7);w5=J8*Xg;}else{c Na=J8*Ma;c Yg=c(dot(w5,w5),dot(Na,Na));w5=P0(Yg,inverse(e0(w5,Na)));}}c Zg=abs(G1)*w5;float zd=(Ma-dot(Zg,J8))/(Ka*(x4*2.));
#ifndef BB
if((j0&A8)!=0u) S1.y=zd;else S1.x=zd;
#endif
}
#ifndef BB
S1.xy*=wd;S1.y=max(S1.y,1e-4);if(O2!=.0){S1.x=j7-S1.x;}
#endif
H8=P0(W0,G1*w5);if(m7!=B8) Ga=true;}else{
#ifndef BB
S1=f(Da,-1.,.0,.0);
#ifdef GB
if(O2!=.0){S1.y=j7;S1.z=gd;S1.w=Da;if((j0&f4)==y8&&m7==B8){if(x1<.0){n7+=x1;x1=-x1;}float j4=f1-n7;j4=mod(j4+X6,p8)-X6;j4=clamp(j4,.0,x1);if(j4>x1*.5){j4=x1-j4;}c D8=c(sin(j4),cos(j4));
#if 0
float T1=1.+.33*log2(X6/(H3-min(x1,H3-H3/16.)));f ah=hd(x1,D8,.5*(T1/3.));float bh=d8(ah e1);float ch=Ac(bh);float dh=(.5-ch)*(ra*2.);float eh=T1/max(dh,T1);G1*=eh;
#endif
S1=hd(x1,D8,G1);}H8=P0(W0,(G1*O2)*d3);}else
#endif
{H8=sign(P0(G1*d3,inverse(W0)))*x4;}if(bool(j0&K3)!=bool(j0&gg)){S1*=f(-1.,+1.,+1.,+1.);}
#endif
if(m7==Xc) vd=md;if((j0&Sc)!=0u&&m7!=Wc){Ga=true;}}Pg=P0(W0,vd)+H8+I2;
#ifdef BB
R Q4=L0(OB,i3*4u+2u);l7=a2(Q4.x);
#else
S1.xy=mix(S1.xy,c(1.,-1.),Lf(j.fh!=0u));
#endif
return!Ga;}
#endif
#if defined(CB)&&defined(DB)
e c Lb(S l6,c1(uint) i3
#ifdef BB
,c1(N) l7
#else
,c1(d) gh
#endif
k6){i3=floatBitsToUint(l6.z)&0xffffu;
#ifdef BB
R Q4=L0(OB,i3*4u+2u);l7=a2(Q4.x);
#else
gh=fa(floatBitsToInt(l6.z)>>16);
#endif
c m6=l6.xy;e0 W0=L1(uintBitsToFloat(L0(OB,i3*4u)));R O4=L0(OB,i3*4u+1u);c I2=uintBitsToFloat(O4.xy);m6=P0(W0,m6)+I2;return m6;}
#endif
#if defined(CB)&&defined(FB)
e c Kb(S l6,c1(uint) i3,
#ifdef BB
c1(N) l7,
#endif
c1(c) hh k6){i3=floatBitsToUint(l6.z)&0xffffu;R Q4=L0(OB,i3*4u+2u);
#ifdef BB
l7=a2(Q4.x);
#endif
c m6=l6.xy;S v7=uintBitsToFloat(Q4.yzw);hh=(m6*v7.x+v7.yz)*j.ih;return m6;}
#endif
e d K8(d e2,d H1,d j3){return(H1-e2)/max(1.-e2*j3,p9);}
#if defined(QB)||defined(JD)
e uint L8(N0 k4,uint jh){uint Oa=(k4.y>>g6)*(jh<<g6)+((k4.x>>g6)<<(g6<<1));Oa+=((k4.x&0x1cu)<<g6)+((k4.y&0x1cu)<<2);Oa+=((k4.y&0x3u)<<2)+(k4.x&0x3u);return Oa;}
#endif
#ifdef QB
#ifdef Q
#define k5 v2
#define d4(x5) F1=x5;r3
#else
#define k5 P1
#define d4(x5) A0(m0,x5);d2;
#endif
e d Pa(uint kh){return fa(int((kh&xa)-n5))*va;}e uint w7(d o){return uint(o*qg+.5);}
#endif
