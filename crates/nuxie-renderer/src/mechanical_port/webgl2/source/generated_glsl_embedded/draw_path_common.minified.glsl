#define l7 -2.
#define ed -1.5
#define fd .25
#define B8 1e3
#define gd (B8*B8)
#ifdef CB
Y3 M4(h3,jg,MC);
#ifdef GB
h6(h3,j7,YC);
#endif
Z3 G4 N4(Zc,Ig,OB);O5(Qb,kf,DD);P5(Rb,lf,PB);N4(ad,Jg,ID);H4
#endif
#if defined(GB)||defined(FB)
g4(j7,ca)
#endif
#ifdef EB
H3 e3(h3,bd,ED);
#if defined(GB)||defined(FB)
h6(h3,j7,YC);
#endif
#ifdef FB
p5(h3,cd,FD);
#endif
e3(i5,a4,HC);
#if defined(BB)&&defined(T)&&!defined(Q)
q5(YD);
#endif
I3 g4(bd,N9)
#ifdef FB
g4(cd,S9)
#endif
j5 c4(W5) k5
#endif
#ifdef EB
e bool V5(f P){return P.y>=.0;}e bool V5(D P){return P.y>=.0;}
#endif
#if defined(EB)&&defined(GB)
e bool bc(f P){return P.x<ed;}e bool cc(f P){return P.y<ed;}
#endif
#ifdef CB
f hd(float ya,c C8,float G1){c i6=(1.-C8*abs(G1))*.5;float h4,r5;if(abs(ya-Y6)<1./B8){h4=.0;r5=.0;}else{float za=tan(ya);h4=sign(Y6-ya)/max(abs(za),1./gd);r5=h4>=.0?i6.y-(1.-i6.x)*za:i6.y+i6.x*za;}f P;P.x=max(i6.x,.0)+fd;P.y=-i6.y+l7;P.z=h4;P.w=r5;return P;}
#endif
#ifdef GB
e d c8(f P K3){d h4=P.z;d r5=max(P.w,.0);d j6=h4>=.0?m5(r5):.0;if(abs(h4)<B8){d x=abs(P.x)-fd;d y=-P.y+l7;d d3=(y-r5)*0.5984134206;i t=r5+d3*E0(0.20888568955,0.62665706865,1.04442844776,1.46219982687);i u=t*-h4+(y*h4+x);i Kg=E0(m5(u[0]),m5(u[1]),m5(u[2]),m5(u[3]));i id=t*5.09593080173+-2.54796540086;i Lg=exp2(-id*id);j6+=dot(Kg,Lg)*d3;}return j6*sign(P.x);}e d D4(f P K3){float j6=1.;float Mg=(1.-l7)+P.x;j6-=m5(Mg);float Ng=1.-P.y;j6-=m5(Ng);return j6;}
#endif
#if defined(CB)&&defined(OD)
e d0 k6(int jd){return d0(jd&((1<<Nc)-1),jd>>Nc);}e float Aa(uint z){return float(z)*(p8/(65536.*65536.));}e float Og(uint z){return float(z&0xffffu)*(1./65535.);}e float kd(Y W0,c Pg){c n2=N0(W0,Pg);return(abs(n2.x)+abs(n2.y))*(1./dot(n2,n2));}e bool r9(f m7,f Ba,int q,c1(uint) i3,c1(c) Qg
#ifndef BB
,c1(f) S1
#else
,c1(N) n7
#endif
l6){int D8=int(m7.x);float G1=m7.y;float Ca=m7.z;int ld=floatBitsToInt(m7.w)>>2;int o7=floatBitsToInt(m7.w)&3;int Da=min(D8,ld-1);int v5=q*ld+Da;R O4=F1(MC,k6(v5));uint m0=O4.w;uint E8=max(m0&Vc,1u);R Ea=L0(ID,E8-1u);c md=uintBitsToFloat(Ea.xy);i3=Ea.z&0xffffu;uint nd=Ea.w;Y W0=L1(uintBitsToFloat(L0(OB,i3*4u)));R P4=L0(OB,i3*4u+1u);c J2=uintBitsToFloat(P4.xy);float O2=uintBitsToFloat(P4.z);float P2=uintBitsToFloat(P4.w);uint od=m0&f4;if(od!=0u){D8=int(Ba.x);G1=Ba.y;Ca=Ba.z;}if(D8!=Da){int pd=v5+D8-Da;R qd=F1(MC,k6(pd));if((qd.w&(f4|0xffffu))!=(m0&(f4|0xffffu))){bool Rg=O2==.0||md.x!=.0;if(Rg){v5=int(nd);O4=F1(MC,k6(v5));}}else{v5=pd;O4=qd;}m0=(O4.w&~f4)|od;}bool Fa=false;float m1;
#ifdef GB
float p7;float w1;if((m0&J3)==x8&&o7==A8){uint rd=O4.z;float i4=float(rd&0xffffu);float o2=float(rd>>16);d0 F8=d0(-i4-1.,o2-i4+1.);if((m0&f4)!=0u) F8=-F8;R sd=F1(MC,k6(v5+F8.x));R Ga=F1(MC,k6(v5+F8.y));if((Ga.w&(f4|0xffffu))!=(sd.w&(f4|0xffffu))){Ga=F1(MC,k6(int(nd)));}p7=Aa(sd.z);float td=Aa(Ga.z);w1=td-p7;if(abs(w1)>X3) w1-=p8*sign(w1);float Ha=o2+1.-float(Oc);float ud=clamp(round(abs(w1)/X3*Ha),1.,Ha-1.);float q7=Ha-ud;if(i4<=q7){w1=-(X3*sign(w1)-w1);o2=q7;if(i4==q7) G1=-G1;}else if(i4==q7+1.){i4=.0;o2=.0;G1=.0;}else{i4-=q7+2.;o2=ud;}if(i4==o2){m1=td;}else{m1=p7+w1*(i4/o2);}}else
#endif
{m1=Aa(O4.z);}c M2=c(sin(m1),-cos(m1));c vd=uintBitsToFloat(O4.xy);c G8=c(0,0);if(P2!=.0){P2=max(P2,(qa/3.)/length(N0(W0,M2)));}if(O2!=.0){G1*=sign(determinant(W0));if((m0&z8)!=0u) G1=min(G1,.0);if((m0&Uc)!=0u) G1=max(G1,.0);float Q4=P2!=.0?P2:kd(W0,M2)*y4;d wd=1.;if(Q4>O2&&P2==.0){wd=R3(O2)/R3(Q4);O2=Q4;}c w5=M2*(O2+Q4);
#ifndef BB
float x=G1*(O2+Q4);S1.xy=(1./(Q4*2.))*(c(x,-x)+O2)+.5;S1.zw=O6(.0);
#endif
uint Ia=m0&J3;if(Ia>i7){bool H8=(m0&Sc)!=0u;bool Sg=(m0&z8)!=0u;float j4=Og(O4.z);float I8=sqrt(max(1.-j4*j4,.0));if(H8==Sg) I8=-I8;Y Tg=Y(j4,I8,-I8,j4);c J8=N0(Tg,M2);float Ja=kd(W0,J8);float Ka;if((Ia==eg)||(Ia==fg&&j4>=.25)){float Ug=(m0&y8)!=0u?1.:.25;Ka=O2*(1./max(j4,Ug));}else{Ka=O2*j4+Ja*.5;}float La=Ka+Ja*y4;if((m0&Tc)!=0u){float xd=O2+Q4;float Vg=Q4*.125;if(xd<=La*j4+Vg){float Wg=xd*(1./j4);w5=J8*Wg;}else{c Ma=J8*La;c Xg=c(dot(w5,w5),dot(Ma,Ma));w5=N0(Xg,inverse(Y(w5,Ma)));}}c Yg=abs(G1)*w5;float yd=(La-dot(Yg,J8))/(Ja*(y4*2.));
#ifndef BB
if((m0&z8)!=0u) S1.y=yd;else S1.x=yd;
#endif
}
#ifndef BB
S1.xy*=wd;S1.y=max(S1.y,1e-4);if(P2!=.0){S1.x=l7-S1.x;}
#endif
G8=N0(W0,G1*w5);if(o7!=A8) Fa=true;}else{
#ifndef BB
S1=f(Ca,-1.,.0,.0);
#ifdef GB
if(P2!=.0){S1.y=l7;S1.z=gd;S1.w=Ca;if((m0&J3)==x8&&o7==A8){if(w1<.0){p7+=w1;w1=-w1;}float k4=m1-p7;k4=mod(k4+Y6,p8)-Y6;k4=clamp(k4,.0,w1);if(k4>w1*.5){k4=w1-k4;}c C8=c(sin(k4),cos(k4));
#if 0
float T1=1.+.33*log2(Y6/(X3-min(w1,X3-X3/16.)));f Zg=hd(w1,C8,.5*(T1/3.));float ah=c8(Zg e1);float bh=zc(ah);float ch=(.5-bh)*(qa*2.);float dh=T1/max(ch,T1);G1*=dh;
#endif
S1=hd(w1,C8,G1);}G8=N0(W0,(G1*P2)*M2);}else
#endif
{G8=sign(N0(G1*M2,inverse(W0)))*y4;}if(bool(m0&f4)!=bool(m0&gg)){S1*=f(-1.,+1.,+1.,+1.);}
#endif
if(o7==Xc) vd=md;if((m0&Rc)!=0u&&o7!=Wc){Fa=true;}}Qg=N0(W0,vd)+G8+J2;
#ifdef BB
R R4=L0(OB,i3*4u+2u);n7=a2(R4.x);
#else
S1.xy=mix(S1.xy,c(1.,-1.),Lf(j.eh!=0u));
#endif
return!Fa;}
#endif
#if defined(CB)&&defined(DB)
e c Kb(S m6,c1(uint) i3
#ifdef BB
,c1(N) n7
#else
,c1(d) fh
#endif
l6){i3=floatBitsToUint(m6.z)&0xffffu;
#ifdef BB
R R4=L0(OB,i3*4u+2u);n7=a2(R4.x);
#else
fh=ea(floatBitsToInt(m6.z)>>16);
#endif
c n6=m6.xy;Y W0=L1(uintBitsToFloat(L0(OB,i3*4u)));R P4=L0(OB,i3*4u+1u);c J2=uintBitsToFloat(P4.xy);n6=N0(W0,n6)+J2;return n6;}
#endif
#if defined(CB)&&defined(FB)
e c Jb(S m6,c1(uint) i3,
#ifdef BB
c1(N) n7,
#endif
c1(c) gh l6){i3=floatBitsToUint(m6.z)&0xffffu;R R4=L0(OB,i3*4u+2u);
#ifdef BB
n7=a2(R4.x);
#endif
c n6=m6.xy;S r7=uintBitsToFloat(R4.yzw);gh=(n6*r7.x+r7.yz)*j.hh;return n6;}
#endif
e d K8(d e2,d H1,d j3){return(H1-e2)/max(1.-e2*j3,p9);}
#if defined(QB)||defined(JD)
e uint L8(O0 l4,uint ih){uint Na=(l4.y>>g6)*(ih<<g6)+((l4.x>>g6)<<(g6<<1));Na+=((l4.x&0x1cu)<<g6)+((l4.y&0x1cu)<<2);Na+=((l4.y&0x3u)<<2)+(l4.x&0x3u);return Na;}
#endif
#ifdef QB
#ifdef Q
#define l5 w2
#define d4(x5) E1=x5;r3
#else
#define l5 P1
#define d4(x5) A0(l0,x5);d2;
#endif
e d Oa(uint jh){return ea(int((jh&va)-o5))*ta;}e uint v7(d o){return uint(o*qg+.5);}
#endif
