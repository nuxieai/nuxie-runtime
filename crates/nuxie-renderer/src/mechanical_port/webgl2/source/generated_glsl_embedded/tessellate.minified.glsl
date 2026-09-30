#define Bi 10
#ifdef BB
c1(d0) K(0,f,LD);K(1,f,MD);K(2,f,VC);
#ifdef ua
K(3,uint,IE);K(4,uint,JE);K(5,uint,KE);K(6,uint,LE);
#else
K(3,N,VB);
#endif
d1
#endif
l2 E0 W(0,f,M6);E0 W(1,f,N6);E0 W(2,f,a5);E0 W(3,P,c5);a3 W(4,uint,T7);e2
#ifdef BB
k4 r6(l3,w7,ZC);l4 p4(w7,xa) Q4 X4(yd,nh,LB);X4(zd,oh,AD);R4 v1(HG,d0,D,G,r){L(r,D,LD,f);L(r,D,MD,f);L(r,D,VC,f);
#ifdef ua
L(r,D,IE,uint);L(r,D,JE,uint);L(r,D,KE,uint);L(r,D,LE,uint);N VB=N(IE,JE,KE,LE);
#else
L(r,D,VB,N);
#endif
T(M6,f);T(N6,f);T(a5,f);T(c5,P);T(T7,uint);c z0=LD.xy;c C0=LD.zw;c J0=MD.xy;c P0=MD.zw;bool ve=G<4;float y=ve?VC.z:VC.w;int Gb=int(ve?VB.x:VB.y);
#ifdef ya
int we=Gb<<16;if(VB.z==0xffffffffu){--we;}float v9=float(we>>16);
#else
float v9=float(Gb<<16>>16);
#endif
float w9=float(Gb>>16);c z2=c((G&1)==0?v9:w9,(G&2)==0?y+1.:y);if((w9-v9)*j.Ud<.0){z2.y=2.*y+1.-z2.y;}uint Z2=VB.z&0x3ffu;uint xe=(VB.z>>10)&0x3ffu;uint w2=VB.z>>20;uint i0=VB.w;uint y6=i0&Na;uint a0=y6>0u?p0(AD,max(y6,1u)-1u).z:0u;N V3=a0!=0u?p0(LB,a0*4u+1u):N(0u,0u,0u,0u);float S2=uintBitsToFloat(V3.z);float T2=uintBitsToFloat(V3.w);if(T2!=.0&&S2==.0){float ye;float Ci=Xf(z0,C0,J0,P0,ye);float Hb=T2*(1./Ma);float Di=Sf(z0,C0,J0,P0,ye,Hb);float U7=1.-Di*(1./j4);float Ei=dot(P0-z0,P0-z0)/(Hb*Hb);float Fi=(Ei-1.)*.5;U7=min(U7,Fi);U7=min(U7,.99);float Gi=.5*U7;float x=Xc(Gi)*-2.+1.;float ze=C8(x*T2,Ci);f Ae=mix(z0.xyxy,P0.xyxy,f(1./3.,1./3.,2./3.,2./3.));C0=mix(C0,Ae.xy,ze);J0=mix(J0,Ae.zw,ze);}if((i0&Eg)!=0u){Y l9=n1(uintBitsToFloat(p0(LB,a0*4u)));c Be=M0(l9,-2.*C0+J0+z0);c Ce=M0(l9,-2.*J0+P0+C0);float y1=max(dot(Be,Be),dot(Ce,Ce));float f4=max(ceil(sqrt(.75*4.*sqrt(y1))),1.);Z2=min(uint(f4),Z2);}uint x9=Z2+xe+w2-1u;Y r2=ra(z0,C0,J0,P0);float x1=acos(y8(r2[0],r2[1]));float E4=x1/float(xe);float Ib=determinant(Y(J0-z0,P0-C0));if(Ib==.0) Ib=determinant(r2);if(Ib<.0) E4=-E4;M6=f(z0,C0);N6=f(J0,P0);a5=f(float(x9)-abs(w9-z2.x),float(x9),(w2<<10)|Z2,E4);c5.xy=VC.xy;if(w2>1u){Y Jb=Y(r2[1],VC.xy);float Hi=acos(y8(Jb[0],Jb[1]));float De=float(w2);if((i0&(R3|O8))==(v7|O8)){De-=2.;}float Kb=Hi/De;if(determinant(Jb)<.0) Kb=-Kb;c5.z=Kb;}if(w9<v9){i0|=R2;}T7=i0;f I=G8(z2,2./vg,j.Ud);
#ifdef NC
I.y=-I.y;
#endif
Z(M6);Z(N6);Z(a5);Z(c5);Z(T7);w1(I);}
#endif
#ifdef EB
O3 P3 j3(N,IG){q(M6,f);q(N6,f);q(a5,f);q(c5,P);q(T7,uint);c z0=M6.xy;c C0=M6.zw;c J0=N6.xy;c P0=N6.zw;Y r2=ra(z0,C0,J0,P0);float Ii=max(floor(a5.x),.0);float x9=a5.y;uint Ee=uint(a5.z);float Z2=float(Ee&0x3ffu);float w2=float(Ee>>10);float E4=a5.w;uint i0=T7;float d5=x9-w2;float c2=Ii;if(c2<=d5){i0&=~R3;}else{z0=C0=J0=P0;r2=Y(r2[1],c5.xy);Z2=1.;c2-=d5;d5=w2;E4=c5.z;if((i0&R3)>v7){if(c2<2.5) i0|=rd;if(c2>1.5&&c2<3.5) i0|=sd;}else if((i0&O8)!=0u||(i0&R3)==N8){d5-=2.;--c2;}i0|=E4<.0?P8:td;}c U5;float x1=.0;if(c2==.0||c2==d5||(i0&R3)>v7){bool Z8=c2<d5*.5;U5=Z8?z0:P0;x1=Zc(Z8?r2[0]:r2[1]);}else if((i0&qd)!=0u){U5=z0;if(c2>=float(M8/2u)) U5=C0;if(c2>=float(M8*3u/4u)) U5=J0;if(c2>=float(M8*7u/8u)) U5=c5.xy;}else{float C1,V5;if(Z2==d5){C1=c2/Z2;V5=.0;}else{c B,J,q2=C0-z0;c a7=P0-z0;c z8=J0-C0;J=z8-q2;B=-3.*z8+a7;c Ji=J*(Z2*2.);c d7=q2*(Z2*Z2);float y9=.0;float Ki=min(Z2-1.,c2);c Lb=normalize(r2[0]);float Li=-abs(E4);float Mi=(1.+c2)*abs(E4);for(int Mb=Bi-1;Mb>=0;--Mb){float V7=y9+exp2(float(Mb));if(V7<=Ki){c Nb=V7*B+Ji;Nb=V7*Nb+d7;float Ni=dot(normalize(Nb),Lb);float Ob=V7*Li+Mi;Ob=min(Ob,j4);if(Ni>=cos(Ob)) y9=V7;}}float Oi=y9/Z2;float Fe=c2-y9;float z9=acos(clamp(Lb.x,-1.,1.));z9=Lb.y>=.0?z9:-z9;x1=Fe*E4+z9;c P2=c(sin(x1),-cos(x1));float k=dot(P2,B),A9=dot(P2,J),O1=dot(P2,q2);float Pi=max(A9*A9-k*O1,.0);float C2=sqrt(Pi);if(A9>.0) C2=-C2;C2-=A9;float Ge=-.5*C2*k;c Pb=(abs(C2*C2+Ge)<abs(k*O1+Ge))?c(C2,k):c(O1,C2);V5=(Pb.y!=.0)?Pb.x/Pb.y:.0;V5=clamp(V5,.0,1.);if(Fe==.0) V5=.0;C1=max(Oi,V5);}c Qi=l6(z0,C0,C1);c He=l6(C0,J0,C1);c Ri=l6(J0,P0,C1);c Ie=l6(Qi,He,C1);c Je=l6(He,Ri,C1);U5=l6(Ie,Je,C1);if(C1!=V5) x1=Zc(Je-Ie);}N W7;W7.xy=floatBitsToUint(U5);if((i0&R3)==N8){W7.z=(uint(d5)<<16)|uint(c2);}else{uint Si=uint(int(round(x1*(65536./H8))))&0xffffu;uint Ke=0u;if((i0&R3)>v7){float Ti=clamp(y8(r2[0],r2[1]),-1.,1.);Ke=uint(round(sqrt((1.+Ti)*.5)*65535.));}W7.z=(Si<<16)|Ke;}W7.w=i0;Q2(W7);}
#endif
