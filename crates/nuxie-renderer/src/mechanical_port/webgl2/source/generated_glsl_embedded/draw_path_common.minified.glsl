#define Q7 -2.
#define ie -1.5
#define je .25
#define A9 1e3
#define ke (A9*A9)
#ifdef BB
o4 f5(q3,uh,UB);
#ifdef HB
F6(q3,F7,ZC);
#endif
p4 W4 g5(Xd,ci,KB);g6(Tc,gg,WC);h6(Uc,hg,JB);g5(Yd,di,BD);X4
#endif
#if defined(HB)||defined(FB)
y4(F7,Ta)
#endif
#ifdef EB
U3 p3(q3,Zd,YC);
#if defined(HB)||defined(FB)
F6(q3,F7,ZC);
#endif
#ifdef FB
K5(q3,ae,GD);
#endif
p3(x5,q4,TB);
#if defined(CB)&&defined(H)&&!defined(U)
L5(JD);
#endif
V3 y4(Zd,H8)
#ifdef FB
y4(ae,Ja)
#endif
y5 r4(S4) z5
#endif
#ifdef EB
f bool l6(e T){return T.y>=.0;}f bool l6(D T){return T.y>=.0;}
#endif
#if defined(EB)&&defined(HB)
f bool bd(e T){return T.x<ie;}f bool cd(e T){return T.y<ie;}
#endif
#ifdef BB
e le(float Eb,c B9,float Q0){c G6=(1.-B9*abs(Q0))*.5;float z4,M5;if(abs(Eb-r7)<1./A9){z4=.0;M5=.0;}else{float Fb=tan(Eb);z4=sign(r7-Eb)/max(abs(Fb),1./ke);M5=z4>=.0?G6.y-(1.-G6.x)*Fb:G6.y+G6.x*Fb;}e T;T.x=max(G6.x,.0)+je;T.y=-G6.y+Q7;T.z=z4;T.w=M5;return T;}
#endif
#ifdef HB
f d K8(e T a4){d z4=T.z;d M5=max(T.w,.0);d H6=z4>=.0?C5(M5):.0;if(abs(z4)<A9){d x=abs(T.x)-je;d y=-T.y+Q7;d o3=(y-M5)*0.5984134206;i t=M5+o3*H0(0.20888568955,0.62665706865,1.04442844776,1.46219982687);i u=t*-z4+(y*z4+x);i ei=H0(C5(u[0]),C5(u[1]),C5(u[2]),C5(u[3]));i me=t*5.09593080173+-2.54796540086;i fi=exp2(-me*me);H6+=dot(ei,fi)*o3;}return H6*sign(T.x);}f d R4(e T a4){float H6=1.;float gi=(1.-Q7)+T.x;H6-=C5(gi);float hi=1.-T.y;H6-=C5(hi);return H6;}
#endif
#ifdef BB
f g0 w4(int ne){return g0(ne&((1<<Ld)-1),ne>>Ld);}f float x9(uint z){return float(z)*(Y8/(65536.*65536.));}f float fe(uint z){return float(z&0xffffu)*(1./65535.);}
#endif
#if defined(BB)&&defined(OD)
f float oe(W N0,c ii){c A2=y0(N0,ii);return(abs(A2.x)+abs(A2.y))*(1./dot(A2,A2));}f bool ia(e R7,e Gb,int r,c1(uint) v3,c1(c) ji
#ifndef CB
,c1(e) e2
#else
,c1(P) S7
#endif
I6){int C9=int(R7.x);float Q0=R7.y;float Hb=R7.z;int pe=floatBitsToInt(R7.w)>>2;int T7=floatBitsToInt(R7.w)&3;int H5=min(C9,pe-1);int Y3=r*pe+H5;O c2=r1(UB,w4(Y3));uint a0=c2.w;uint B6=max(a0&mb,1u);O I5=p0(BD,B6-1u);c K7=uintBitsToFloat(I5.xy);v3=I5.z&0xffffu;uint v9=I5.w;W N0=p1(uintBitsToFloat(p0(KB,v3*4u)));O K3=p0(KB,v3*4u+1u);c L1=uintBitsToFloat(K3.xy);float B2=uintBitsToFloat(K3.z);float Z2=uintBitsToFloat(K3.w);uint L7=a0&Y2;if(L7!=0u){C9=int(Gb.x);Q0=Gb.y;Hb=Gb.z;}if(C9!=H5){int w9=Y3+C9-H5;O M7=r1(UB,w4(w9));if((M7.w&(Y2|0xffffu))!=(a0&(Y2|0xffffu))){bool N7=B2==.0||K7.x!=.0;if(N7){Y3=int(v9);c2=r1(UB,w4(Y3));}}else{Y3=w9;c2=M7;}a0=(c2.w&~Y2)|L7;}bool Ib=false;float h1;
#ifdef HB
float U7;float G1;if((a0&I3)==i9&&T7==k9){uint qe=c2.z;float A4=float(qe&0xffffu);float C2=float(qe>>16);g0 D9=g0(-A4-1.,C2-A4+1.);if((a0&Y2)!=0u) D9=-D9;O re=r1(UB,w4(Y3+D9.x));O Jb=r1(UB,w4(Y3+D9.y));if((Jb.w&(Y2|0xffffu))!=(re.w&(Y2|0xffffu))){Jb=r1(UB,w4(int(v9)));}U7=x9(re.z);float se=x9(Jb.z);G1=se-U7;if(abs(G1)>n4) G1-=Y8*sign(G1);float Kb=C2+1.-float(Pd);float te=clamp(round(abs(G1)/n4*Kb),1.,Kb-1.);float V7=Kb-te;if(A4<=V7){G1=-(n4*sign(G1)-G1);C2=V7;if(A4==V7) Q0=-Q0;}else if(A4==V7+1.){A4=.0;C2=.0;Q0=.0;}else{A4-=V7+2.;C2=te;}if(A4==C2){h1=se;}else{h1=U7+G1*(A4/C2);}}else
#endif
{h1=x9(c2.z);}c P1=c(sin(h1),-cos(h1));c D6=uintBitsToFloat(c2.xy);c E9=c(0,0);if(Z2!=.0){Z2=max(Z2,(hb/3.)/length(y0(N0,P1)));}if(B2!=.0){Q0*=sign(determinant(N0));if((a0&r6)!=0u) Q0=min(Q0,.0);if((a0&lb)!=0u) Q0=max(Q0,.0);float h5=Z2!=.0?Z2:oe(N0,P1)*M4;d ue=1.;if(h5>B2&&Z2==.0){ue=i4(B2)/i4(h5);B2=h5;}c L3=P1*(B2+h5);
#ifndef CB
float x=Q0*(B2+h5);e2.xy=(1./(h5*2.))*(c(x,-x)+B2)+.5;e2.zw=i7(.0);
#endif
uint c5=a0&I3;if(c5>E7){bool E6=(a0&jb)!=0u;bool xb=(a0&r6)!=0u;float d2=fe(c2.z);float x4=sqrt(max(1.-d2*d2,.0));if(E6==xb) x4=-x4;W yb=W(d2,x4,-x4,d2);c d5=y0(yb,P1);float Lb=oe(N0,d5);float Mb;if((c5==ib)||(c5==kh&&d2>=.25)){float ki=(a0&j9)!=0u?1.:.25;Mb=B2*(1./max(d2,ki));}else{Mb=B2*d2+Lb*.5;}float Nb=Mb+Lb*M4;if((a0&kb)!=0u){float ve=B2+h5;float li=h5*.125;if(ve<=Nb*d2+li){float mi=ve*(1./d2);L3=d5*mi;}else{c Ob=d5*Nb;c ni=c(dot(L3,L3),dot(Ob,Ob));L3=y0(ni,inverse(W(L3,Ob)));}}c oi=abs(Q0)*L3;float we=(Nb-dot(oi,d5))/(Lb*(M4*2.));
#ifndef CB
if((a0&r6)!=0u) e2.y=we;else e2.x=we;
#endif
}
#ifndef CB
e2.xy*=ue;e2.y=max(e2.y,1e-4);if(Z2!=.0){e2.x=Q7-e2.x;}
#endif
E9=y0(N0,Q0*L3);if(T7!=k9) Ib=true;}else{
#ifndef CB
e2=e(Hb,-1.,.0,.0);
#ifdef HB
if(Z2!=.0){e2.y=Q7;e2.z=ke;e2.w=Hb;if((a0&I3)==i9&&T7==k9){if(G1<.0){U7+=G1;G1=-G1;}float B4=h1-U7;B4=mod(B4+r7,Y8)-r7;B4=clamp(B4,.0,G1);if(B4>G1*.5){B4=G1-B4;}c B9=c(sin(B4),cos(B4));
#if 0
float f2=1.+.33*log2(r7/(n4-min(G1,n4-n4/16.)));e pi=le(G1,B9,.5*(f2/3.));float qi=K8(pi n1);float ri=yd(qi);float si=(.5-ri)*(hb*2.);float ti=f2/max(si,f2);Q0*=ti;
#endif
e2=le(G1,B9,Q0);}E9=y0(N0,(Q0*Z2)*P1);}else
#endif
{E9=sign(y0(Q0*P1,inverse(N0)))*M4;}if(bool(a0&Y2)!=bool(a0&mh)){e2*=e(-1.,+1.,+1.,+1.);}
#endif
if(T7==Ud) D6=K7;if((a0&Sd)!=0u&&T7!=Td){Ib=true;}}ji=y0(N0,D6)+E9+L1;
#ifdef CB
O Z3=p0(KB,v3*4u+2u);S7=T1(Z3.x);
#else
e2.xy=mix(e2.xy,c(1.,-1.),Hg(j.ui!=0u));
#endif
return!Ib;}
#endif
#if defined(BB)&&defined(DB)
f c Pc(M J6,c1(uint) v3
#ifdef CB
,c1(P) S7
#else
,c1(d) vi
#endif
I6){v3=floatBitsToUint(J6.z)&0xffffu;
#ifdef CB
O Z3=p0(KB,v3*4u+2u);S7=T1(Z3.x);
#else
vi=Va(floatBitsToInt(J6.z)>>16);
#endif
c K6=J6.xy;W N0=p1(uintBitsToFloat(p0(KB,v3*4u)));O K3=p0(KB,v3*4u+1u);c L1=uintBitsToFloat(K3.xy);K6=y0(N0,K6)+L1;return K6;}
#endif
#if defined(BB)&&defined(FB)
f c Oc(M J6,c1(uint) v3,
#ifdef CB
c1(P) S7,
#endif
c1(c) wi I6){v3=floatBitsToUint(J6.z)&0xffffu;O Z3=p0(KB,v3*4u+2u);
#ifdef CB
S7=T1(Z3.x);
#endif
c K6=J6.xy;M W7=uintBitsToFloat(Z3.yzw);wi=(K6*W7.x+W7.yz)*j.xi;return K6;}
#endif
f d F9(d q2,d Q1,d w3){return(Q1-q2)/max(1.-q2*w3,ha);}
#if defined(QB)||defined(KD)
f uint G9(S0 r3,uint yi){uint Pb=(r3.y>>x6)*(yi<<x6)+((r3.x>>x6)<<(x6<<1));Pb+=((r3.x&0x1cu)<<x6)+((r3.y&0x1cu)<<2);Pb+=((r3.y&0x3u)<<2)+(r3.x&0x3u);return Pb;}
#endif
#ifdef QB
#ifdef U
#define B5 G2
#define v4(N5) N1=N5;D3
#else
#define B5 Y1
#define v4(N5) z0(n0,N5);p2;
#endif
f d Qb(uint zi){return Va(int((zi&rb)-F5))*pb;}f uint X7(d l){return uint(l*Bh+.5);}
#endif
