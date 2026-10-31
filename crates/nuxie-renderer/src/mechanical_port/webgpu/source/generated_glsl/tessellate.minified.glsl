#define Nj 10
#ifdef VERTEX
f1(d0) K(0,e,QD);K(1,e,RD);K(2,e,UC);
#ifdef fb
K(3,uint,NE);K(4,uint,OE);K(5,uint,PE);K(6,uint,QE);
#else
K(3,R,XB);
#endif
g1
#endif
n2 I0 W(0,e,d7);I0 W(1,e,e7);I0 W(2,e,y5);I0 W(3,M,z5);m3 W(4,uint,y8);P1
#ifdef VERTEX
A4 M6(c3,M7,ZC);B4 f4(M7,ib) h5 o4(Na,nd,JB);o4(pe,xi,CD);i5 l1(NG,d0,A,C,r){L(r,A,QD,e);L(r,A,RD,e);L(r,A,UC,e);
#ifdef fb
L(r,A,NE,uint);L(r,A,OE,uint);L(r,A,PE,uint);L(r,A,QE,uint);R XB=R(NE,OE,PE,QE);
#else
L(r,A,XB,R);
#endif
V(d7,e);V(e7,e);V(y5,e);V(z5,M);V(y8,uint);c C0=QD.xy;c G0=QD.zw;c M0=RD.xy;c V0=RD.zw;bool yf=C<4;float y=yf?UC.z:UC.w;int Jc=int(yf?XB.x:XB.y);
#ifdef jb
int zf=Jc<<16;if(XB.z==0xffffffffu){--zf;}float fa=float(zf>>16);
#else
float fa=float(Jc<<16>>16);
#endif
float ga=float(Jc>>16);c I2=c((C&1)==0?fa:ga,(C&2)==0?y+1.:y);if((ga-fa)*j.Ue<.0){I2.y=2.*y+1.-I2.y;}uint l3=XB.z&0x3ffu;uint Af=(XB.z>>10)&0x3ffu;uint G2=XB.z>>20;uint c0=XB.w;uint H6=c0&Cb;uint g0=H6>0u?o0(CD,max(H6,1u)-1u).z:0u;R S6=g0!=0u?o0(JB,g0*4u+2u):R(0u,0u,0u,0u);float f2=uintBitsToFloat(S6.x);float P2=uintBitsToFloat(S6.y);if(P2!=.0&&f2==.0){float Bf;float Oj=Vg(C0,G0,M0,V0,Bf);float Kc=P2*(1./xb);float Pj=Qg(C0,G0,M0,V0,Bf,Kc);float z8=1.-Pj*(1./z4);float Qj=dot(V0-C0,V0-C0)/(Kc*Kc);float Rj=(Qj-1.)*.5;z8=min(z8,Rj);z8=min(z8,.99);float Sj=.5*z8;float x=Pd(Sj)*-2.+1.;float Cf=h9(x*P2,Oj);e Df=mix(C0.xyxy,V0.xyxy,e(1./3.,1./3.,2./3.,2./3.));G0=mix(G0,Df.xy,Cf);M0=mix(M0,Df.zw,Cf);}if((c0&Gh)!=0u){X Z9=v1(uintBitsToFloat(o0(JB,g0*4u)));c Ef=F0(Z9,-2.*G0+M0+C0);c Ff=F0(Z9,-2.*M0+V0+G0);float E1=max(dot(Ef,Ef),dot(Ff,Ff));float m4=max(ceil(sqrt(.75*4.*sqrt(E1))),1.);l3=min(uint(m4),l3);}uint ha=l3+Af+G2-1u;X E2=cb(C0,G0,M0,V0);float h1=acos(d9(E2[0],E2[1]));float U4=h1/float(Af);float Lc=determinant(X(M0-C0,V0-G0));if(Lc==.0) Lc=determinant(E2);if(Lc<.0) U4=-U4;d7=e(C0,G0);e7=e(M0,V0);y5=e(float(ha)-abs(ga-I2.x),float(ha),(G2<<10)|l3,U4);z5.xy=UC.xy;if(G2>1u){X Mc=X(E2[1],UC.xy);float Tj=acos(d9(Mc[0],Mc[1]));float Gf=float(G2);if((c0&(y3|x9))==(K7|x9)){Gf-=2.;}float Nc=Tj/Gf;if(determinant(Mc)<.0) Nc=-Nc;z5.z=Nc;}if(ga<fa){c0|=a3;}y8=c0;e I=l9(I2,2./th,j.Ue);
#ifdef POST_INVERT_Y
I.y=-I.y;
#endif
e0(d7);e0(e7);e0(y5);e0(z5);e0(y8);B1(I);}
#endif
#ifdef FRAGMENT
Y3 Z3 r2(R,OG){q(d7,e);q(e7,e);q(y5,e);q(z5,M);q(y8,uint);c C0=d7.xy;c G0=d7.zw;c M0=e7.xy;c V0=e7.zw;X E2=cb(C0,G0,M0,V0);float Uj=max(floor(y5.x),.0);float ha=y5.y;uint Hf=uint(y5.z);float l3=float(Hf&0x3ffu);float G2=float(Hf>>10);float U4=y5.w;uint c0=y8;float C3=ha-G2;float z1=Uj;if(z1<=C3){c0&=~y3;}else{C0=G0=M0=V0;E2=X(E2[1],z5.xy);l3=1.;z1-=C3;C3=G2;U4=z5.z;bool If=(c0&x9)!=0u;if(If||(c0&y3)==w9){C3-=2.;--z1;}bool Vj=If&&(z1==0.||z1==C3);if(Vj){c0&=~y3;}else if((c0&y3)!=Jh){c0|=U4<.0?C6:Bb;}if((c0&y3)>K7){float ia=C3*.5;if(z1<ia) c0|=zb;if(C3>3.&&z1>ia-1.&&z1<ia+1.) c0|=Ab;z1=z1<ia?.0:C3;}}c f6;float h1=.0;if(z1==.0||z1==C3){bool L6=z1<C3*.5;f6=L6?C0:V0;h1=lb(L6?E2[0]:E2[1]);}else if((c0&je)!=0u){f6=C0;if(z1>=float(r9/2u)) f6=G0;if(z1>=float(r9*3u/4u)) f6=M0;if(z1>=float(r9*7u/8u)) f6=z5.xy;}else{float L1,g6;if(l3==C3){L1=z1/l3;g6=.0;}else{c E,J,D2=G0-C0;c p7=V0-C0;c e9=M0-G0;J=e9-D2;E=-3.*e9+p7;c Wj=J*(l3*2.);c r7=D2*(l3*l3);float ja=.0;float Xj=min(l3-1.,z1);c Oc=normalize(E2[0]);float Yj=-abs(U4);float Zj=(1.+z1)*abs(U4);for(int Pc=Nj-1;Pc>=0;--Pc){float A8=ja+exp2(float(Pc));if(A8<=Xj){c Qc=A8*E+Wj;Qc=A8*Qc+r7;float ak=dot(normalize(Qc),Oc);float Rc=A8*Yj+Zj;Rc=min(Rc,z4);if(ak>=cos(Rc)) ja=A8;}}float bk=ja/l3;float Jf=z1-ja;float ka=acos(clamp(Oc.x,-1.,1.));ka=Oc.y>=.0?ka:-ka;h1=Jf*U4+ka;c M1=c(sin(h1),-cos(h1));float l=dot(M1,E),la=dot(M1,J),Y1=dot(M1,D2);float ck=max(la*la-l*Y1,.0);float N2=sqrt(ck);if(la>.0) N2=-N2;N2-=la;float Kf=-.5*N2*l;c Sc=(abs(N2*N2+Kf)<abs(l*Y1+Kf))?c(N2,l):c(Y1,N2);g6=(Sc.y!=.0)?Sc.x/Sc.y:.0;g6=clamp(g6,.0,1.);if(Jf==.0) g6=.0;L1=max(bk,g6);}c dk=y6(C0,G0,L1);c Lf=y6(G0,M0,L1);c ek=y6(M0,V0,L1);c Mf=y6(dk,Lf,L1);c Nf=y6(Lf,ek,L1);f6=y6(Mf,Nf,L1);if(L1!=g6) h1=lb(Nf-Mf);}R B8;B8.xy=floatBitsToUint(f6);if((c0&y3)==w9){B8.z=(uint(C3)<<16)|uint(z1);}else{uint fk=uint(int(round(h1*(65536./A7))))&0xffffu;uint Of=0u;if((c0&y3)>K7){float gk=clamp(d9(E2[0],E2[1]),-1.,1.);Of=uint(round(sqrt((1.+gk)*.5)*65535.));}B8.z=(fk<<16)|Of;}B8.w=c0;v2(B8);}
#endif
