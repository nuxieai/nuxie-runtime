#define Cj 10
#ifdef VERTEX
d1(e0) K(0,e,PD);K(1,e,QD);K(2,e,UC);
#ifdef cb
K(3,uint,NE);K(4,uint,OE);K(5,uint,PE);K(6,uint,QE);
#else
K(3,R,XB);
#endif
e1
#endif
n2 H0 W(0,e,a7);H0 W(1,e,c7);H0 W(2,e,o5);H0 W(3,M,p5);i3 W(4,uint,q8);M1
#ifdef VERTEX
x4 K6(w3,J7,ZC);y4 d4(J7,fb) c5 m4(Ka,jd,JB);m4(me,pi,CD);d5 n1(KG,e0,A,C,r){L(r,A,PD,e);L(r,A,QD,e);L(r,A,UC,e);
#ifdef cb
L(r,A,NE,uint);L(r,A,OE,uint);L(r,A,PE,uint);L(r,A,QE,uint);R XB=R(NE,OE,PE,QE);
#else
L(r,A,XB,R);
#endif
V(a7,e);V(c7,e);V(o5,e);V(p5,M);V(q8,uint);c A0=PD.xy;c F0=PD.zw;c L0=QD.xy;c S0=QD.zw;bool qf=C<4;float y=qf?UC.z:UC.w;int Fc=int(qf?XB.x:XB.y);
#ifdef gb
int rf=Fc<<16;if(XB.z==0xffffffffu){--rf;}float ca=float(rf>>16);
#else
float ca=float(Fc<<16>>16);
#endif
float da=float(Fc>>16);c G2=c((C&1)==0?ca:da,(C&2)==0?y+1.:y);if((da-ca)*j.Pe<.0){G2.y=2.*y+1.-G2.y;}uint h3=XB.z&0x3ffu;uint sf=(XB.z>>10)&0x3ffu;uint E2=XB.z>>20;uint Z=XB.w;uint F6=Z&zb;uint g0=F6>0u?n0(CD,max(F6,1u)-1u).z:0u;R Y7=g0!=0u?n0(JB,g0*4u+2u):R(0u,0u,0u,0u);float e2=uintBitsToFloat(Y7.x);float P2=uintBitsToFloat(Y7.y);if(P2!=.0&&e2==.0){float tf;float Dj=Mg(A0,F0,L0,S0,tf);float Gc=P2*(1./ub);float Ej=Hg(A0,F0,L0,S0,tf,Gc);float r8=1.-Ej*(1./w4);float Fj=dot(S0-A0,S0-A0)/(Gc*Gc);float Gj=(Fj-1.)*.5;r8=min(r8,Gj);r8=min(r8,.99);float Hj=.5*r8;float x=Ld(Hj)*-2.+1.;float uf=c9(x*P2,Dj);e vf=mix(A0.xyxy,S0.xyxy,e(1./3.,1./3.,2./3.,2./3.));F0=mix(F0,vf.xy,uf);L0=mix(L0,vf.zw,uf);}if((Z&xh)!=0u){X V9=p1(uintBitsToFloat(n0(JB,g0*4u)));c wf=D0(V9,-2.*F0+L0+A0);c xf=D0(V9,-2.*L0+S0+F0);float C1=max(dot(wf,wf),dot(xf,xf));float k4=max(ceil(sqrt(.75*4.*sqrt(C1))),1.);h3=min(uint(k4),h3);}uint ea=h3+sf+E2-1u;X C2=Za(A0,F0,L0,S0);float f1=acos(X8(C2[0],C2[1]));float O4=f1/float(sf);float Hc=determinant(X(L0-A0,S0-F0));if(Hc==.0) Hc=determinant(C2);if(Hc<.0) O4=-O4;a7=e(A0,F0);c7=e(L0,S0);o5=e(float(ea)-abs(da-G2.x),float(ea),(E2<<10)|h3,O4);p5.xy=UC.xy;if(E2>1u){X Ic=X(C2[1],UC.xy);float Ij=acos(X8(Ic[0],Ic[1]));float yf=float(E2);if((Z&(v3|p9))==(I7|p9)){yf-=2.;}float Jc=Ij/yf;if(determinant(Ic)<.0) Jc=-Jc;p5.z=Jc;}if(da<ca){Z|=Z2;}q8=Z;e I=g9(G2,2./kh,j.Pe);
#ifdef POST_INVERT_Y
I.y=-I.y;
#endif
a0(a7);a0(c7);a0(o5);a0(p5);a0(q8);A1(I);}
#endif
#ifdef FRAGMENT
Y3 Z3 M2(R,LG){q(a7,e);q(c7,e);q(o5,e);q(p5,M);q(q8,uint);c A0=a7.xy;c F0=a7.zw;c L0=c7.xy;c S0=c7.zw;X C2=Za(A0,F0,L0,S0);float Jj=max(floor(o5.x),.0);float ea=o5.y;uint zf=uint(o5.z);float h3=float(zf&0x3ffu);float E2=float(zf>>10);float O4=o5.w;uint Z=q8;float A3=ea-E2;float y1=Jj;if(y1<=A3){Z&=~v3;}else{A0=F0=L0=S0;C2=X(C2[1],p5.xy);h3=1.;y1-=A3;A3=E2;O4=p5.z;bool Af=(Z&p9)!=0u;if(Af||(Z&v3)==o9){A3-=2.;--y1;}bool Kj=Af&&(y1==0.||y1==A3);if(Kj){Z&=~v3;}else if((Z&v3)!=Ah){Z|=O4<.0?z6:yb;}if((Z&v3)>I7){float fa=A3*.5;if(y1<fa) Z|=wb;if(A3>3.&&y1>fa-1.&&y1<fa+1.) Z|=xb;y1=y1<fa?.0:A3;}}c c6;float f1=.0;if(y1==.0||y1==A3){bool J6=y1<A3*.5;c6=J6?A0:S0;f1=ib(J6?C2[0]:C2[1]);}else if((Z&fe)!=0u){c6=A0;if(y1>=float(m9/2u)) c6=F0;if(y1>=float(m9*3u/4u)) c6=L0;if(y1>=float(m9*7u/8u)) c6=p5.xy;}else{float J1,d6;if(h3==A3){J1=y1/h3;d6=.0;}else{c B,J,B2=F0-A0;c n7=S0-A0;c Y8=L0-F0;J=Y8-B2;B=-3.*Y8+n7;c Lj=J*(h3*2.);c p7=B2*(h3*h3);float ga=.0;float Mj=min(h3-1.,y1);c Kc=normalize(C2[0]);float Nj=-abs(O4);float Oj=(1.+y1)*abs(O4);for(int Lc=Cj-1;Lc>=0;--Lc){float v8=ga+exp2(float(Lc));if(v8<=Mj){c Mc=v8*B+Lj;Mc=v8*Mc+p7;float Pj=dot(normalize(Mc),Kc);float Nc=v8*Nj+Oj;Nc=min(Nc,w4);if(Pj>=cos(Nc)) ga=v8;}}float Qj=ga/h3;float Bf=y1-ga;float ha=acos(clamp(Kc.x,-1.,1.));ha=Kc.y>=.0?ha:-ha;f1=Bf*O4+ha;c S1=c(sin(f1),-cos(f1));float l=dot(S1,B),ia=dot(S1,J),W1=dot(S1,B2);float Rj=max(ia*ia-l*W1,.0);float L2=sqrt(Rj);if(ia>.0) L2=-L2;L2-=ia;float Cf=-.5*L2*l;c Oc=(abs(L2*L2+Cf)<abs(l*W1+Cf))?c(L2,l):c(W1,L2);d6=(Oc.y!=.0)?Oc.x/Oc.y:.0;d6=clamp(d6,.0,1.);if(Bf==.0) d6=.0;J1=max(Qj,d6);}c Sj=v6(A0,F0,J1);c Df=v6(F0,L0,J1);c Tj=v6(L0,S0,J1);c Ef=v6(Sj,Df,J1);c Ff=v6(Df,Tj,J1);c6=v6(Ef,Ff,J1);if(J1!=d6) f1=ib(Ff-Ef);}R w8;w8.xy=floatBitsToUint(c6);if((Z&v3)==o9){w8.z=(uint(A3)<<16)|uint(y1);}else{uint Uj=uint(int(round(f1*(65536./y7))))&0xffffu;uint Gf=0u;if((Z&v3)>I7){float Vj=clamp(X8(C2[0],C2[1]),-1.,1.);Gf=uint(round(sqrt((1.+Vj)*.5)*65535.));}w8.z=(Uj<<16)|Gf;}w8.w=Z;N2(w8);}
#endif
