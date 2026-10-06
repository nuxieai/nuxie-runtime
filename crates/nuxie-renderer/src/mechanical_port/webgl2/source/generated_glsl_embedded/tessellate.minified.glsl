#define Ci 10
#ifdef BB
c1(d0) K(0,e,KD);K(1,e,LD);K(2,e,TC);
#ifdef ua
K(3,uint,HE);K(4,uint,IE);K(5,uint,JE);K(6,uint,KE);
#else
K(3,M,VB);
#endif
d1
#endif
l2 E0 V(0,e,L6);E0 V(1,e,M6);E0 V(2,e,a5);E0 V(3,O,c5);Z2 V(4,uint,R7);e2
#ifdef BB
k4 q6(l3,r7,YC);l4 p4(r7,xa) Q4 X4(yd,oh,LB);X4(zd,ph,ZC);R4 w1(GG,d0,D,G,r){L(r,D,KD,e);L(r,D,LD,e);L(r,D,TC,e);
#ifdef ua
L(r,D,HE,uint);L(r,D,IE,uint);L(r,D,JE,uint);L(r,D,KE,uint);M VB=M(HE,IE,JE,KE);
#else
L(r,D,VB,M);
#endif
T(L6,e);T(M6,e);T(a5,e);T(c5,O);T(R7,uint);c z0=KD.xy;c C0=KD.zw;c J0=LD.xy;c P0=LD.zw;bool ve=G<4;float y=ve?TC.z:TC.w;int Gb=int(ve?VB.x:VB.y);
#ifdef ya
int we=Gb<<16;if(VB.z==0xffffffffu){--we;}float r9=float(we>>16);
#else
float r9=float(Gb<<16>>16);
#endif
float v9=float(Gb>>16);c y2=c((G&1)==0?r9:v9,(G&2)==0?y+1.:y);if((v9-r9)*j.Ud<.0){y2.y=2.*y+1.-y2.y;}uint Y2=VB.z&0x3ffu;uint xe=(VB.z>>10)&0x3ffu;uint v2=VB.z>>20;uint i0=VB.w;uint x6=i0&Na;uint a0=x6>0u?p0(ZC,max(x6,1u)-1u).z:0u;M V3=a0!=0u?p0(LB,a0*4u+1u):M(0u,0u,0u,0u);float R2=uintBitsToFloat(V3.z);float S2=uintBitsToFloat(V3.w);if(S2!=.0&&R2==.0){float ye;float Di=Yf(z0,C0,J0,P0,ye);float Hb=S2*(1./Ma);float Ei=Tf(z0,C0,J0,P0,ye,Hb);float S7=1.-Ei*(1./j4);float Fi=dot(P0-z0,P0-z0)/(Hb*Hb);float Gi=(Fi-1.)*.5;S7=min(S7,Gi);S7=min(S7,.99);float Hi=.5*S7;float x=Xc(Hi)*-2.+1.;float ze=A8(x*S2,Di);e Ae=mix(z0.xyxy,P0.xyxy,e(1./3.,1./3.,2./3.,2./3.));C0=mix(C0,Ae.xy,ze);J0=mix(J0,Ae.zw,ze);}if((i0&Fg)!=0u){Y k9=n1(uintBitsToFloat(p0(LB,a0*4u)));c Be=M0(k9,-2.*C0+J0+z0);c Ce=M0(k9,-2.*J0+P0+C0);float z1=max(dot(Be,Be),dot(Ce,Ce));float f4=max(ceil(sqrt(.75*4.*sqrt(z1))),1.);Y2=min(uint(f4),Y2);}uint w9=Y2+xe+v2-1u;Y q2=ra(z0,C0,J0,P0);float y1=acos(w8(q2[0],q2[1]));float E4=y1/float(xe);float Ib=determinant(Y(J0-z0,P0-C0));if(Ib==.0) Ib=determinant(q2);if(Ib<.0) E4=-E4;L6=e(z0,C0);M6=e(J0,P0);a5=e(float(w9)-abs(v9-y2.x),float(w9),(v2<<10)|Y2,E4);c5.xy=TC.xy;if(v2>1u){Y Jb=Y(q2[1],TC.xy);float Ii=acos(w8(Jb[0],Jb[1]));float De=float(v2);if((i0&(R3|N8))==(L8|N8)){De-=2.;}float Kb=Ii/De;if(determinant(Jb)<.0) Kb=-Kb;c5.z=Kb;}if(v9<r9){i0|=Q2;}R7=i0;e I=E8(y2,2./wg,j.Ud);
#ifdef MC
I.y=-I.y;
#endif
Z(L6);Z(M6);Z(a5);Z(c5);Z(R7);x1(I);}
#endif
#ifdef FB
O3 P3 j3(M,HG){q(L6,e);q(M6,e);q(a5,e);q(c5,O);q(R7,uint);c z0=L6.xy;c C0=L6.zw;c J0=M6.xy;c P0=M6.zw;Y q2=ra(z0,C0,J0,P0);float Ji=max(floor(a5.x),.0);float w9=a5.y;uint Ee=uint(a5.z);float Y2=float(Ee&0x3ffu);float v2=float(Ee>>10);float E4=a5.w;uint i0=R7;float q3=w9-v2;float r1=Ji;if(r1<=q3){i0&=~R3;}else{z0=C0=J0=P0;q2=Y(q2[1],c5.xy);Y2=1.;r1-=q3;q3=v2;E4=c5.z;bool Fe=(i0&N8)!=0u;if(Fe||(i0&R3)==M8){q3-=2.;--r1;}bool Ki=Fe&&(r1==0.||r1==q3);if(Ki){i0&=~R3;}else{i0|=E4<.0?O8:td;}if((i0&R3)>L8){float x9=q3*.5;if(r1<x9) i0|=rd;if(q3>3.&&r1>x9-1.&&r1<x9+1.) i0|=sd;r1=r1<x9?.0:q3;}}c T5;float y1=.0;if(r1==.0||r1==q3){bool Y8=r1<q3*.5;T5=Y8?z0:P0;y1=Zc(Y8?q2[0]:q2[1]);}else if((i0&qd)!=0u){T5=z0;if(r1>=float(K8/2u)) T5=C0;if(r1>=float(K8*3u/4u)) T5=J0;if(r1>=float(K8*7u/8u)) T5=c5.xy;}else{float D1,U5;if(Y2==q3){D1=r1/Y2;U5=.0;}else{c B,J,p2=C0-z0;c Z6=P0-z0;c x8=J0-C0;J=x8-p2;B=-3.*x8+Z6;c Li=J*(Y2*2.);c c7=p2*(Y2*Y2);float y9=.0;float Mi=min(Y2-1.,r1);c Lb=normalize(q2[0]);float Ni=-abs(E4);float Oi=(1.+r1)*abs(E4);for(int Mb=Ci-1;Mb>=0;--Mb){float T7=y9+exp2(float(Mb));if(T7<=Mi){c Nb=T7*B+Li;Nb=T7*Nb+c7;float Pi=dot(normalize(Nb),Lb);float Ob=T7*Ni+Oi;Ob=min(Ob,j4);if(Pi>=cos(Ob)) y9=T7;}}float Qi=y9/Y2;float Ge=r1-y9;float z9=acos(clamp(Lb.x,-1.,1.));z9=Lb.y>=.0?z9:-z9;y1=Ge*E4+z9;c O2=c(sin(y1),-cos(y1));float k=dot(O2,B),A9=dot(O2,J),P1=dot(O2,p2);float Ri=max(A9*A9-k*P1,.0);float B2=sqrt(Ri);if(A9>.0) B2=-B2;B2-=A9;float He=-.5*B2*k;c Pb=(abs(B2*B2+He)<abs(k*P1+He))?c(B2,k):c(P1,B2);U5=(Pb.y!=.0)?Pb.x/Pb.y:.0;U5=clamp(U5,.0,1.);if(Ge==.0) U5=.0;D1=max(Qi,U5);}c Si=k6(z0,C0,D1);c Ie=k6(C0,J0,D1);c Ti=k6(J0,P0,D1);c Je=k6(Si,Ie,D1);c Ke=k6(Ie,Ti,D1);T5=k6(Je,Ke,D1);if(D1!=U5) y1=Zc(Ke-Je);}M U7;U7.xy=floatBitsToUint(T5);if((i0&R3)==M8){U7.z=(uint(q3)<<16)|uint(r1);}else{uint Ui=uint(int(round(y1*(65536./F8))))&0xffffu;uint Le=0u;if((i0&R3)>L8){float Vi=clamp(w8(q2[0],q2[1]),-1.,1.);Le=uint(round(sqrt((1.+Vi)*.5)*65535.));}U7.z=(Ui<<16)|Le;}U7.w=i0;P2(U7);}
#endif
