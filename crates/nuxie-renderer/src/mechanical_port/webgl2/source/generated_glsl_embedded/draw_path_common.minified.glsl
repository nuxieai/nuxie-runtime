#define R7 -2.
#define le -1.5
#define me .25
#define C9 1e3
#define ne (C9*C9)
#ifdef BB
o4 f5(q3,xh,UB);
#ifdef HB
F6(q3,F7,ZC);
#endif
p4 W4 g5(Zd,gi,KB);g6(Vc,jg,WC);h6(Wc,kg,JB);g5(ae,hi,CD);X4
#endif
#if defined(HB)||defined(FB)
y4(F7,Va)
#endif
#ifdef EB
U3 p3(q3,be,YC);
#if defined(HB)||defined(FB)
F6(q3,F7,ZC);
#endif
#ifdef FB
K5(q3,ce,HD);
#endif
p3(x5,q4,TB);
#if defined(CB)&&defined(H)&&!defined(U)
L5(KD);
#endif
V3 y4(be,I8)
#ifdef FB
y4(ce,La)
#endif
y5 r4(S4) z5
#endif
#ifdef EB
f bool l6(e T){return T.y>=.0;}f bool l6(D T){return T.y>=.0;}
#endif
#if defined(EB)&&defined(HB)
f bool dd(e T){return T.x<le;}f bool ed(e T){return T.y<le;}
#endif
#ifdef BB
e oe(float Gb,c D9,float I0){c G6=(1.-D9*abs(I0))*.5;float z4,M5;if(abs(Gb-r7)<1./C9){z4=.0;M5=.0;}else{float Hb=tan(Gb);z4=sign(r7-Gb)/max(abs(Hb),1./ne);M5=z4>=.0?G6.y-(1.-G6.x)*Hb:G6.y+G6.x*Hb;}e T;T.x=max(G6.x,.0)+me;T.y=-G6.y+R7;T.z=z4;T.w=M5;return T;}
#endif
#ifdef HB
f d L8(e T a4){d z4=T.z;d M5=max(T.w,.0);d H6=z4>=.0?C5(M5):.0;if(abs(z4)<C9){d x=abs(T.x)-me;d y=-T.y+R7;d o3=(y-M5)*0.5984134206;i t=M5+o3*H0(0.20888568955,0.62665706865,1.04442844776,1.46219982687);i u=t*-z4+(y*z4+x);i ii=H0(C5(u[0]),C5(u[1]),C5(u[2]),C5(u[3]));i pe=t*5.09593080173+-2.54796540086;i ji=exp2(-pe*pe);H6+=dot(ii,ji)*o3;}return H6*sign(T.x);}f d R4(e T a4){float H6=1.;float ki=(1.-R7)+T.x;H6-=C5(ki);float li=1.-T.y;H6-=C5(li);return H6;}
#endif
#ifdef BB
f g0 w4(int qe){return g0(qe&((1<<Nd)-1),qe>>Nd);}f float z9(uint z){return float(z)*(Z8/(65536.*65536.));}f float ie(uint z){return float(z&0xffffu)*(1./65535.);}
#endif
#if defined(BB)&&defined(PD)
f float re(W O0,c mi){c A2=y0(O0,mi);return(abs(A2.x)+abs(A2.y))*(1./dot(A2,A2));}f bool ka(e S7,e Ib,int r,c1(uint) v3,c1(c) ni
#ifndef CB
,c1(e) e2
#else
,c1(P) T7
#endif
I6){int E9=int(S7.x);float I0=S7.y;float Jb=S7.z;int se=floatBitsToInt(S7.w)>>2;int U7=floatBitsToInt(S7.w)&3;int H5=min(E9,se-1);int Y3=r*se+H5;O c2=r1(UB,w4(Y3));uint a0=c2.w;uint A6=max(a0&ob,1u);O I5=p0(CD,A6-1u);c L7=uintBitsToFloat(I5.xy);v3=I5.z&0xffffu;uint x9=I5.w;W O0=p1(uintBitsToFloat(p0(KB,v3*4u)));O K3=p0(KB,v3*4u+1u);c L1=uintBitsToFloat(K3.xy);float B2=uintBitsToFloat(K3.z);float Z2=uintBitsToFloat(K3.w);uint C6=a0&Y2;if(C6!=0u){E9=int(Ib.x);I0=Ib.y;Jb=Ib.z;}if(E9!=H5){int y9=Y3+E9-H5;O N7=r1(UB,w4(y9));if((N7.w&(Y2|0xffffu))!=(a0&(Y2|0xffffu))){bool O7=B2==.0||L7.x!=.0;if(O7){Y3=int(x9);c2=r1(UB,w4(Y3));}}else{Y3=y9;c2=N7;}a0=(c2.w&~Y2)|C6;}bool Kb=false;float h1;
#ifdef HB
float V7;float G1;if((a0&I3)==j9&&U7==l9){uint te=c2.z;float A4=float(te&0xffffu);float C2=float(te>>16);g0 F9=g0(-A4-1.,C2-A4+1.);if((a0&Y2)!=0u) F9=-F9;O ue=r1(UB,w4(Y3+F9.x));O Lb=r1(UB,w4(Y3+F9.y));if((Lb.w&(Y2|0xffffu))!=(ue.w&(Y2|0xffffu))){Lb=r1(UB,w4(int(x9)));}V7=z9(ue.z);float ve=z9(Lb.z);G1=ve-V7;if(abs(G1)>n4) G1-=Z8*sign(G1);float Mb=C2+1.-float(Rd);float we=clamp(round(abs(G1)/n4*Mb),1.,Mb-1.);float W7=Mb-we;if(A4<=W7){G1=-(n4*sign(G1)-G1);C2=W7;if(A4==W7) I0=-I0;}else if(A4==W7+1.){A4=.0;C2=.0;I0=.0;}else{A4-=W7+2.;C2=we;}if(A4==C2){h1=ve;}else{h1=V7+G1*(A4/C2);}}else
#endif
{h1=z9(c2.z);}c P1=c(sin(h1),-cos(h1));c D6=uintBitsToFloat(c2.xy);c G9=c(0,0);if(Z2!=.0){Z2=max(Z2,(jb/3.)/length(y0(O0,P1)));}if(B2!=.0){I0*=sign(determinant(O0));if((a0&r6)!=0u) I0=min(I0,.0);if((a0&nb)!=0u) I0=max(I0,.0);float h5=Z2!=.0?Z2:re(O0,P1)*M4;d xe=1.;if(h5>B2&&Z2==.0){xe=i4(B2)/i4(h5);B2=h5;}c L3=P1*(B2+h5);
#ifndef CB
float x=I0*(B2+h5);e2.xy=(1./(h5*2.))*(c(x,-x)+B2)+.5;e2.zw=i7(.0);
#endif
uint c5=a0&I3;if(c5>E7){bool E6=(a0&lb)!=0u;bool zb=(a0&r6)!=0u;float d2=ie(c2.z);float x4=sqrt(max(1.-d2*d2,.0));if(E6==zb) x4=-x4;W Ab=W(d2,x4,-x4,d2);c d5=y0(Ab,P1);float Nb=re(O0,d5);float Ob;if((c5==kb)||(c5==nh&&d2>=.25)){float oi=(a0&k9)!=0u?1.:.25;Ob=B2*(1./max(d2,oi));}else{Ob=B2*d2+Nb*.5;}float Pb=Ob+Nb*M4;if((a0&mb)!=0u){float ye=B2+h5;float pi=h5*.125;if(ye<=Pb*d2+pi){float qi=ye*(1./d2);L3=d5*qi;}else{c Qb=d5*Pb;c ri=c(dot(L3,L3),dot(Qb,Qb));L3=y0(ri,inverse(W(L3,Qb)));}}c si=abs(I0)*L3;float ze=(Pb-dot(si,d5))/(Nb*(M4*2.));
#ifndef CB
if((a0&r6)!=0u) e2.y=ze;else e2.x=ze;
#endif
}
#ifndef CB
e2.xy*=xe;e2.y=max(e2.y,1e-4);if(Z2!=.0){e2.x=R7-e2.x;}
#endif
G9=y0(O0,I0*L3);if(U7!=l9) Kb=true;}else{
#ifndef CB
e2=e(Jb,-1.,.0,.0);
#ifdef HB
if(Z2!=.0){e2.y=R7;e2.z=ne;e2.w=Jb;if((a0&I3)==j9&&U7==l9){if(G1<.0){V7+=G1;G1=-G1;}float B4=h1-V7;B4=mod(B4+r7,Z8)-r7;B4=clamp(B4,.0,G1);if(B4>G1*.5){B4=G1-B4;}c D9=c(sin(B4),cos(B4));
#if 0
float f2=1.+.33*log2(r7/(n4-min(G1,n4-n4/16.)));e ti=oe(G1,D9,.5*(f2/3.));float ui=L8(ti n1);float vi=Ad(ui);float wi=(.5-vi)*(jb*2.);float xi=f2/max(wi,f2);I0*=xi;
#endif
e2=oe(G1,D9,I0);}G9=y0(O0,(I0*Z2)*P1);}else
#endif
{G9=sign(y0(I0*P1,inverse(O0)))*M4;}if(bool(a0&Y2)!=bool(a0&ph)){e2*=e(-1.,+1.,+1.,+1.);}
#endif
if(U7==Wd) D6=L7;if((a0&Ud)!=0u&&U7!=Vd){Kb=true;}}ni=y0(O0,D6)+G9+L1;
#ifdef CB
O Z3=p0(KB,v3*4u+2u);T7=T1(Z3.x);
#else
e2.xy=mix(e2.xy,c(1.,-1.),Kg(j.yi!=0u));
#endif
return!Kb;}
#endif
#if defined(BB)&&defined(DB)
f c Rc(M J6,c1(uint) v3
#ifdef CB
,c1(P) T7
#else
,c1(d) zi
#endif
I6){v3=floatBitsToUint(J6.z)&0xffffu;
#ifdef CB
O Z3=p0(KB,v3*4u+2u);T7=T1(Z3.x);
#else
zi=Xa(floatBitsToInt(J6.z)>>16);
#endif
c K6=J6.xy;W O0=p1(uintBitsToFloat(p0(KB,v3*4u)));O K3=p0(KB,v3*4u+1u);c L1=uintBitsToFloat(K3.xy);K6=y0(O0,K6)+L1;return K6;}
#endif
#if defined(BB)&&defined(FB)
f c Qc(M J6,c1(uint) v3,
#ifdef CB
c1(P) T7,
#endif
c1(c) Ai I6){v3=floatBitsToUint(J6.z)&0xffffu;O Z3=p0(KB,v3*4u+2u);
#ifdef CB
T7=T1(Z3.x);
#endif
c K6=J6.xy;M X7=uintBitsToFloat(Z3.yzw);Ai=(K6*X7.x+X7.yz)*j.Bi;return K6;}
#endif
f d H9(d q2,d Q1,d w3){return(Q1-q2)/max(1.-q2*w3,ja);}
#if defined(QB)||defined(LD)
f uint I9(S0 r3,uint Ci){uint Rb=(r3.y>>x6)*(Ci<<x6)+((r3.x>>x6)<<(x6<<1));Rb+=((r3.x&0x1cu)<<x6)+((r3.y&0x1cu)<<2);Rb+=((r3.y&0x3u)<<2)+(r3.x&0x3u);return Rb;}
#endif
#ifdef QB
#ifdef U
#define B5 G2
#define v4(N5) N1=N5;D3
#else
#define B5 Y1
#define v4(N5) z0(n0,N5);p2;
#endif
f d Sb(uint Di){return Xa(int((Di&tb)-F5))*rb;}f uint Y7(d l){return uint(l*Eh+.5);}
#endif
