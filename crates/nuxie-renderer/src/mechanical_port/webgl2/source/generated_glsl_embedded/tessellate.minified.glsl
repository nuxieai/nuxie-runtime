#define Dj 10
#ifdef BB
d1(e0) K(0,e,PD);K(1,e,QD);K(2,e,UC);
#ifdef cb
K(3,uint,NE);K(4,uint,OE);K(5,uint,PE);K(6,uint,QE);
#else
K(3,R,XB);
#endif
e1
#endif
n2 H0 W(0,e,d7);H0 W(1,e,e7);H0 W(2,e,o5);H0 W(3,M,p5);i3 W(4,uint,r8);N1
#ifdef BB
x4 K6(w3,L7,ZC);y4 d4(L7,fb) c5 m4(Ka,kd,JB);m4(me,qi,CD);d5 n1(KG,e0,A,C,r){L(r,A,PD,e);L(r,A,QD,e);L(r,A,UC,e);
#ifdef cb
L(r,A,NE,uint);L(r,A,OE,uint);L(r,A,PE,uint);L(r,A,QE,uint);R XB=R(NE,OE,PE,QE);
#else
L(r,A,XB,R);
#endif
V(d7,e);V(e7,e);V(o5,e);V(p5,M);V(r8,uint);c A0=PD.xy;c F0=PD.zw;c L0=QD.xy;c S0=QD.zw;bool rf=C<4;float y=rf?UC.z:UC.w;int Gc=int(rf?XB.x:XB.y);
#ifdef gb
int sf=Gc<<16;if(XB.z==0xffffffffu){--sf;}float ca=float(sf>>16);
#else
float ca=float(Gc<<16>>16);
#endif
float da=float(Gc>>16);c G2=c((C&1)==0?ca:da,(C&2)==0?y+1.:y);if((da-ca)*j.Qe<.0){G2.y=2.*y+1.-G2.y;}uint h3=XB.z&0x3ffu;uint tf=(XB.z>>10)&0x3ffu;uint E2=XB.z>>20;uint Z=XB.w;uint F6=Z&zb;uint g0=F6>0u?n0(CD,max(F6,1u)-1u).z:0u;R Q6=g0!=0u?n0(JB,g0*4u+2u):R(0u,0u,0u,0u);float e2=uintBitsToFloat(Q6.x);float P2=uintBitsToFloat(Q6.y);if(P2!=.0&&e2==.0){float uf;float Ej=Ng(A0,F0,L0,S0,uf);float Hc=P2*(1./ub);float Fj=Ig(A0,F0,L0,S0,uf,Hc);float v8=1.-Fj*(1./w4);float Gj=dot(S0-A0,S0-A0)/(Hc*Hc);float Hj=(Gj-1.)*.5;v8=min(v8,Hj);v8=min(v8,.99);float Ij=.5*v8;float x=Md(Ij)*-2.+1.;float vf=d9(x*P2,Ej);e wf=mix(A0.xyxy,S0.xyxy,e(1./3.,1./3.,2./3.,2./3.));F0=mix(F0,wf.xy,vf);L0=mix(L0,wf.zw,vf);}if((Z&yh)!=0u){X V9=p1(uintBitsToFloat(n0(JB,g0*4u)));c xf=D0(V9,-2.*F0+L0+A0);c yf=D0(V9,-2.*L0+S0+F0);float C1=max(dot(xf,xf),dot(yf,yf));float k4=max(ceil(sqrt(.75*4.*sqrt(C1))),1.);h3=min(uint(k4),h3);}uint ea=h3+tf+E2-1u;X C2=Za(A0,F0,L0,S0);float f1=acos(Y8(C2[0],C2[1]));float O4=f1/float(tf);float Ic=determinant(X(L0-A0,S0-F0));if(Ic==.0) Ic=determinant(C2);if(Ic<.0) O4=-O4;d7=e(A0,F0);e7=e(L0,S0);o5=e(float(ea)-abs(da-G2.x),float(ea),(E2<<10)|h3,O4);p5.xy=UC.xy;if(E2>1u){X Jc=X(C2[1],UC.xy);float Jj=acos(Y8(Jc[0],Jc[1]));float zf=float(E2);if((Z&(v3|q9))==(K7|q9)){zf-=2.;}float Kc=Jj/zf;if(determinant(Jc)<.0) Kc=-Kc;p5.z=Kc;}if(da<ca){Z|=Z2;}r8=Z;e I=h9(G2,2./lh,j.Qe);
#ifdef NC
I.y=-I.y;
#endif
a0(d7);a0(e7);a0(o5);a0(p5);a0(r8);A1(I);}
#endif
#ifdef DB
Y3 Z3 M2(R,LG){q(d7,e);q(e7,e);q(o5,e);q(p5,M);q(r8,uint);c A0=d7.xy;c F0=d7.zw;c L0=e7.xy;c S0=e7.zw;X C2=Za(A0,F0,L0,S0);float Kj=max(floor(o5.x),.0);float ea=o5.y;uint Af=uint(o5.z);float h3=float(Af&0x3ffu);float E2=float(Af>>10);float O4=o5.w;uint Z=r8;float A3=ea-E2;float y1=Kj;if(y1<=A3){Z&=~v3;}else{A0=F0=L0=S0;C2=X(C2[1],p5.xy);h3=1.;y1-=A3;A3=E2;O4=p5.z;bool Bf=(Z&q9)!=0u;if(Bf||(Z&v3)==p9){A3-=2.;--y1;}bool Lj=Bf&&(y1==0.||y1==A3);if(Lj){Z&=~v3;}else if((Z&v3)!=Bh){Z|=O4<.0?z6:yb;}if((Z&v3)>K7){float fa=A3*.5;if(y1<fa) Z|=wb;if(A3>3.&&y1>fa-1.&&y1<fa+1.) Z|=xb;y1=y1<fa?.0:A3;}}c c6;float f1=.0;if(y1==.0||y1==A3){bool J6=y1<A3*.5;c6=J6?A0:S0;f1=ib(J6?C2[0]:C2[1]);}else if((Z&ge)!=0u){c6=A0;if(y1>=float(n9/2u)) c6=F0;if(y1>=float(n9*3u/4u)) c6=L0;if(y1>=float(n9*7u/8u)) c6=p5.xy;}else{float J1,d6;if(h3==A3){J1=y1/h3;d6=.0;}else{c B,J,B2=F0-A0;c p7=S0-A0;c Z8=L0-F0;J=Z8-B2;B=-3.*Z8+p7;c Mj=J*(h3*2.);c r7=B2*(h3*h3);float ga=.0;float Nj=min(h3-1.,y1);c Lc=normalize(C2[0]);float Oj=-abs(O4);float Pj=(1.+y1)*abs(O4);for(int Mc=Dj-1;Mc>=0;--Mc){float w8=ga+exp2(float(Mc));if(w8<=Nj){c Nc=w8*B+Mj;Nc=w8*Nc+r7;float Qj=dot(normalize(Nc),Lc);float Oc=w8*Oj+Pj;Oc=min(Oc,w4);if(Qj>=cos(Oc)) ga=w8;}}float Rj=ga/h3;float Cf=y1-ga;float ha=acos(clamp(Lc.x,-1.,1.));ha=Lc.y>=.0?ha:-ha;f1=Cf*O4+ha;c K1=c(sin(f1),-cos(f1));float l=dot(K1,B),ia=dot(K1,J),W1=dot(K1,B2);float Sj=max(ia*ia-l*W1,.0);float L2=sqrt(Sj);if(ia>.0) L2=-L2;L2-=ia;float Df=-.5*L2*l;c Pc=(abs(L2*L2+Df)<abs(l*W1+Df))?c(L2,l):c(W1,L2);d6=(Pc.y!=.0)?Pc.x/Pc.y:.0;d6=clamp(d6,.0,1.);if(Cf==.0) d6=.0;J1=max(Rj,d6);}c Tj=v6(A0,F0,J1);c Ef=v6(F0,L0,J1);c Uj=v6(L0,S0,J1);c Ff=v6(Tj,Ef,J1);c Gf=v6(Ef,Uj,J1);c6=v6(Ff,Gf,J1);if(J1!=d6) f1=ib(Gf-Ff);}R x8;x8.xy=floatBitsToUint(c6);if((Z&v3)==p9){x8.z=(uint(A3)<<16)|uint(y1);}else{uint Vj=uint(int(round(f1*(65536./A7))))&0xffffu;uint Hf=0u;if((Z&v3)>K7){float Wj=clamp(Y8(C2[0],C2[1]),-1.,1.);Hf=uint(round(sqrt((1.+Wj)*.5)*65535.));}x8.z=(Vj<<16)|Hf;}x8.w=Z;N2(x8);}
#endif
